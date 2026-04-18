<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { authStore } from '$lib/authStore.svelte.js';
  import { getCurrentTheme } from '$lib/ui/ThemeStore.svelte.js';
  import {
    callManager,
    callStore,
    endCurrentCall,
    startGroupCall,
  } from '$lib/webrtc-calls.svelte.ts';
  import { loadParticipants } from '$lib/conversationStore.svelte.ts';

  // ── Route params ─────────────────────────────────────────────────────
  let conversationId = $derived($page.params.id);
  let isVideo        = $derived($page.url.searchParams.get('type') !== 'audio');

  // ── Local state ──────────────────────────────────────────────────────
  let loading        = $state(true);
  let error          = $state<string | null>(null);
  let callDuration   = $state(0);
  let timerInterval: ReturnType<typeof setInterval> | null = null;
  let showDebugPanel = $state(false);
  let callPhase      = $derived(
    callStore.isInCall   ? 'active' :
    callStore.isCalling  ? 'connecting' :
    error                ? 'error' :
    loading              ? 'loading' :
                           'idle'
  );

  // Participants (load async)
  const participants = $derived(loadParticipants(conversationId));

  const callTitle = $derived(
    participants.value.length > 0
      ? participants.value
          .filter((p: any) => p.id !== authStore.user?.id)
          .map((p: any) => p.name ?? p.username)
          .join(', ') || 'Conversation'
      : 'Appel'
  );

  // ── Lifecycle ────────────────────────────────────────────────────────
  onMount(async () => {
    try {
      await participants;
      // Check HTTPS / getUserMedia availability
      if (browser && !navigator.mediaDevices?.getUserMedia) {
        error = 'Les appels nécessitent un contexte sécurisé (HTTPS). Accédez à Nook via https://';
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur de chargement';
    } finally {
      loading = false;
    }

    // Keyboard shortcuts
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
    callManager.stopRingtone();
    window.removeEventListener('keydown', handleKeydown);
  });

  // ── Call actions ─────────────────────────────────────────────────────
  async function startCall(type: 'audio' | 'video') {
    try {
      error = null;
      if (browser && !navigator.mediaDevices?.getUserMedia) {
        error = 'Les appels nécessitent HTTPS. Vérifiez votre connexion sécurisée.';
        return;
      }
      const ids = participants.value.map((p: any) => p.id);
      // Timeout 15s to avoid infinite spinner
      await Promise.race([
        startGroupCall(conversationId, ids, type),
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error("Délai d'initialisation dépassé (15s) — vérifiez HTTPS et les permissions du navigateur")), 15000)
        ),
      ]);
      startTimer();
    } catch (err) {
      error = err instanceof Error ? err.message : "Erreur de démarrage de l'appel";
    }
  }

  function endCall() {
    callManager.stopRingtone();
    endCurrentCall();
    stopTimer();
    goto('/chat');
  }

  function toggleMute() {
    callStore.isMuted = !callStore.isMuted;
    const tracks = callStore.localStream?.getAudioTracks();
    tracks?.forEach((t: MediaStreamTrack) => (t.enabled = !callStore.isMuted));
  }

  function toggleVideo() {
    callStore.isVideoOff = !callStore.isVideoOff;
    const tracks = callStore.localStream?.getVideoTracks();
    tracks?.forEach((t: MediaStreamTrack) => (t.enabled = !callStore.isVideoOff));
  }

  function toggleScreenShare() {
    if (callStore.isScreenSharing) {
      callStore.screenShareLocalVideoElement?.srcObject
        ?.getTracks()
        .forEach((t: MediaStreamTrack) => t.stop());
      callStore.isScreenSharing = false;
      return;
    }
    navigator.mediaDevices
      .getDisplayMedia({ video: true })
      .then((stream) => {
        if (callStore.screenShareLocalVideoElement) {
          callStore.screenShareLocalVideoElement.srcObject = stream;
        }
        stream.getVideoTracks()[0].addEventListener('ended', () => {
          callStore.isScreenSharing = false;
        });
        callStore.isScreenSharing = true;
      })
      .catch((err) => {
        error = 'Impossible de partager l\'écran : ' + (err as Error).message;
      });
  }

  // ── Timer ────────────────────────────────────────────────────────────
  function startTimer() {
    callDuration = 0;
    timerInterval = setInterval(() => {
      callDuration++;
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function formatDuration(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }

  // ── Keyboard ─────────────────────────────────────────────────────────
  function handleKeydown(e: KeyboardEvent) {
    if (!callStore.isInCall) return;
    if (e.key === 'm' || e.key === 'M') toggleMute();
    if (e.key === 'v' || e.key === 'V') toggleVideo();
    if (e.key === 'Escape') endCall();
    if (e.ctrlKey && e.key === 'd') {
      e.preventDefault();
      showDebugPanel = !showDebugPanel;
    }
  }
</script>

<!-- ════════════════════════════════════════════════════════════════════════
     CALL PAGE TEMPLATE
     ════════════════════════════════════════════════════════════════════════ -->

<div class="call-page" class:dark={getCurrentTheme()?.isDark}>
  {#if loading}
    <!-- Loading -->
    <div class="call-center">
      <div class="spinner" />
      <p class="call-center-text">Chargement…</p>
    </div>

  {:else if error && !callStore.isInCall}
    <!-- Error state -->
    <div class="call-center error-state">
      <svg class="call-center-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="15" y1="9" x2="9" y2="15"/>
        <line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
      <p class="call-center-text error">{error}</p>
      <button class="btn btn-secondary" onclick={() => { error = null; }}>
        Réessayer
      </button>
    </div>

  {:else if callStore.isInCall}
    <!-- ═══ ACTIVE CALL ═══ -->
    <div class="call-active">
      <header class="call-header">
        <div class="call-header-info">
          <span class="call-header-title">{callTitle}</span>
          <span class="call-timer">{formatDuration(callDuration)}</span>
        </div>
        <div class="call-header-badges">
          {#if callStore.callQuality}
            <span class="quality-dot" class:good={callStore.callQuality === 'good'} class:fair={callStore.callQuality === 'fair'} class:poor={callStore.callQuality === 'poor'} />
          {/if}
          <span class="call-type-badge">
            {#if isVideo}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
            {/if}
          </span>
        </div>
      </header>

      <!-- Video / audio grid -->
      <div class="participants-grid" class:video-mode={isVideo}>
        <!-- Screen share -->
        {#if callStore.isScreenSharing}
          <div class="participant-card screen-share">
            <video bind:this={callStore.screenShareLocalVideoElement} autoplay muted playsinline class="video-stream" />
            <div class="participant-overlay">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
              <span>Partage d'écran</span>
            </div>
          </div>
        {/if}

        <!-- Local stream -->
        <div class="participant-card local" class:without-video={!isVideo || callStore.isVideoOff}>
          {#if isVideo && !callStore.isVideoOff}
            <video bind:this={callStore.localVideoElement} autoplay muted playsinline class="video-stream" />
          {/if}
          <div class="participant-overlay">
            <span class="participant-name">Vous</span>
            <div class="participant-badges">
              {#if callStore.isMuted}<span class="badge muted" aria-label="Micro coupé"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .41-.04.81-.1 1.2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/></svg></span>{/if}
              {#if callStore.isVideoOff && isVideo}<span class="badge cam-off" aria-label="Caméra coupée"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M21 7l-5.2 3.3M2 5h15a2 2 0 0 1 2 2v6m-2 0V5H2v14h13"/></svg></span>{/if}
            </div>
          </div>
        </div>

        <!-- Remote streams -->
        {#each Array.from(callStore.remoteStreams.entries()) as [userId, stream]}
          {@const participant = participants.value.find((p: any) => p.id === userId)}
          <div class="participant-card remote" class:without-video={!isVideo}>
            {#if isVideo}
              <video srcObject={stream} autoplay playsinline class="video-stream" />
            {/if}
            <div class="participant-overlay">
              <span class="participant-name">{participant?.name ?? participant?.username ?? 'Participant'}</span>
            </div>
          </div>
        {/each}

        <!-- Waiting -->
        {#if callStore.remoteStreams.size === 0}
          <div class="waiting">
            <div class="pulse-ring" />
            <p>En attente des participants…</p>
          </div>
        {/if}
      </div>

      <!-- Controls -->
      <div class="call-controls">
        <button onclick={toggleMute} class="ctrl-btn" class:active={callStore.isMuted} aria-label={callStore.isMuted ? 'Activer micro' : 'Couper micro'}>
          {#if callStore.isMuted}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .41-.04.81-.1 1.2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/></svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/></svg>
          {/if}
          <span>{callStore.isMuted ? 'Muté' : 'Micro'}</span>
        </button>

        <button onclick={toggleVideo} class="ctrl-btn" class:active={callStore.isVideoOff} aria-label={callStore.isVideoOff ? 'Activer vidéo' : 'Couper vidéo'}>
          {#if callStore.isVideoOff}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M21 7l-5.2 3.3M2 5h15a2 2 0 0 1 2 2v6m-2 0V5H2v14h13"/></svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
          {/if}
          <span>{callStore.isVideoOff ? 'Vidéo off' : 'Vidéo'}</span>
        </button>

        <button onclick={toggleScreenShare} class="ctrl-btn" class:active={callStore.isScreenSharing} aria-label={callStore.isScreenSharing ? 'Arrêter partage' : 'Partager écran'}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
          <span>Écran</span>
        </button>

        <button onclick={endCall} class="ctrl-btn hangup" aria-label="Raccrocher">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91"/><line x1="23" y1="1" x2="1" y2="23"/></svg>
          <span>Raccrocher</span>
        </button>
      </div>
    </div>

  {:else if callStore.isCalling}
    <!-- ═══ CONNECTING ═══ -->
    <div class="call-center connecting">
      <div class="pulse-ring large" />
      <div class="connecting-icon">
        {#if isVideo}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
        {/if}
      </div>
      <p class="call-center-text">Connexion en cours…</p>
      <p class="call-center-hint">Appel vers {callTitle}</p>
      <button class="btn btn-danger" onclick={endCall}>Annuler</button>
    </div>

  {:else}
    <!-- ═══ IDLE — Pre-call ═══ -->
    <div class="call-center idle">
      <div class="idle-avatar">
        {#if isVideo}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
        {/if}
      </div>
      <h2 class="call-center-title">{callTitle}</h2>
      <p class="call-center-hint">
        {participants.value.filter((p: any) => p.id !== authStore.user?.id).length} participant{participants.value.filter((p: any) => p.id !== authStore.user?.id).length !== 1 ? 's' : ''} dans la conversation
      </p>

      {#if error}
        <div class="call-error-inline">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          <span>{error}</span>
        </div>
      {/if}

      <div class="idle-actions">
        <button class="btn btn-primary btn-audio" onclick={() => startCall('audio')}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
          Appel audio
        </button>
        <button class="btn btn-secondary btn-video" onclick={() => startCall('video')}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
          {isVideo ? 'Appel vidéo' : 'Ajouter la vidéo'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Debug panel (Ctrl+D) -->
  {#if showDebugPanel && callStore.isInCall}
    <div class="debug-panel">
      <h3>Qualité d'appel</h3>
      <table>
        <tbody>
          <tr><td>Qualité</td><td>{callStore.callQuality}</td></tr>
          <tr><td>Latence</td><td>{callStore.rtt} ms</td></tr>
          <tr><td>Jitter</td><td>{callStore.jitter} ms</td></tr>
          <tr><td>Pertes</td><td>{callStore.packetsLost}</td></tr>
          <tr><td>Paires</td><td>{callStore.peerConnections.size}</td></tr>
        </tbody>
      </table>
      <p class="debug-hint">Ctrl+D pour fermer</p>
    </div>
  {/if}

  <!-- Error banner (during call) -->
  {#if callStore.error && callStore.isInCall}
    <div class="error-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      <span>{callStore.error}</span>
      <button onclick={() => (callStore.error = null)}>✕</button>
    </div>
  {/if}
</div>

<style>
  /* ═══════════════════════════════════════════════════════════════════
     CALL PAGE LAYOUT
     ═══════════════════════════════════════════════════════════════════ */
  .call-page {
    height: 100dvh;
    width: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary, #0f172a);
    color: var(--text-primary, #f1f5f9);
    overflow: hidden;
  }

  /* Center states (idle, loading, connecting, error) */
  .call-center {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
    text-align: center;
  }
  .call-center-icon { width: 4rem; height: 4rem; opacity: 0.6; }
  .call-center-text { font-size: 1.125rem; color: var(--text-secondary, #94a3b8); margin: 0; }
  .call-center-hint { font-size: 0.875rem; color: var(--text-muted, #64748b); margin: 0; }
  .call-center-title { font-size: 1.5rem; font-weight: 600; margin: 0; }

  /* Error */
  .error-state .call-center-icon { color: var(--danger, #ef4444); }
  .call-center-text.error { color: var(--danger, #ef4444); }

  .call-error-inline {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 0.75rem;
    color: var(--danger, #ef4444);
    font-size: 0.875rem;
    max-width: 400px;
  }
  .call-error-inline svg { width: 1.25rem; height: 1.25rem; flex-shrink: 0; }

  /* ═══════════════════════════════════════════════════════════════════
     IDLE / PRE-CALL
     ═══════════════════════════════════════════════════════════════════ */
  .idle .idle-avatar {
    width: 6rem; height: 6rem;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    background: var(--bg-secondary, #1e293b);
    border: 2px solid var(--border, #334155);
    margin-bottom: 0.5rem;
  }
  .idle .idle-avatar svg { width: 3rem; height: 3rem; color: var(--accent, #60a5fa); }

  .idle-actions {
    display: flex; gap: 1rem; margin-top: 1.5rem;
    flex-wrap: wrap; justify-content: center;
  }

  /* ═══════════════════════════════════════════════════════════════════
     CONNECTING
     ═══════════════════════════════════════════════════════════════════ */
  .connecting .connecting-icon {
    width: 5rem; height: 5rem;
    display: flex; align-items: center; justify-content: center;
    z-index: 2;
  }
  .connecting .connecting-icon svg { width: 2.5rem; height: 2.5rem; color: var(--accent, #60a5fa); }

  /* ═══════════════════════════════════════════════════════════════════
     ACTIVE CALL — HEADER
     ═══════════════════════════════════════════════════════════════════ */
  .call-active { flex: 1; display: flex; flex-direction: column; }

  .call-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary, #1e293b);
    border-bottom: 1px solid var(--border, #334155);
  }
  .call-header-info { display: flex; flex-direction: column; }
  .call-header-title { font-weight: 600; font-size: 0.95rem; }
  .call-timer {
    font-size: 0.8rem; color: var(--text-muted, #64748b);
    font-variant-numeric: tabular-nums;
  }
  .call-header-badges { display: flex; align-items: center; gap: 0.5rem; }

  .quality-dot {
    width: 0.5rem; height: 0.5rem; border-radius: 50%;
    background: var(--text-muted, #64748b);
  }
  .quality-dot.good { background: #22c55e; }
  .quality-dot.fair { background: #f59e0b; }
  .quality-dot.poor { background: #ef4444; }

  .call-type-badge {
    width: 1.5rem; height: 1.5rem;
    display: flex; align-items: center; justify-content: center;
    opacity: 0.7;
  }
  .call-type-badge svg { width: 1rem; height: 1rem; }

  /* ═══════════════════════════════════════════════════════════════════
     ACTIVE CALL — PARTICIPANT GRID
     ═══════════════════════════════════════════════════════════════════ */
  .participants-grid {
    flex: 1;
    display: grid;
    gap: 0.5rem;
    padding: 0.75rem;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    align-content: center;
  }

  .participant-card {
    position: relative;
    border-radius: 0.75rem;
    overflow: hidden;
    background: var(--bg-tertiary, #334155);
    aspect-ratio: 16/9;
    min-height: 150px;
  }
  .participant-card.without-video {
    aspect-ratio: auto;
    min-height: 100px;
    display: flex; align-items: center; justify-content: center;
  }
  .participant-card.screen-share { grid-column: 1 / -1; min-height: 300px; }

  .video-stream {
    width: 100%; height: 100%;
    object-fit: cover; background: #000;
  }

  .participant-overlay {
    position: absolute; bottom: 0; left: 0; right: 0;
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: linear-gradient(transparent, rgba(0,0,0,0.7));
    font-size: 0.8rem;
  }
  .participant-overlay svg { width: 1rem; height: 1rem; }
  .participant-name { font-weight: 500; }

  .participant-badges { display: flex; gap: 0.25rem; margin-left: auto; }
  .badge {
    display: flex; align-items: center; justify-content: center;
    width: 1.25rem; height: 1.25rem;
    background: rgba(0,0,0,0.5); border-radius: 50%;
  }
  .badge svg { width: 0.75rem; height: 0.75rem; }
  .badge.muted { color: #ef4444; }
  .badge.cam-off { color: #ef4444; }

  /* Waiting pulse */
  .waiting {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    grid-column: 1 / -1; padding: 3rem; color: var(--text-muted, #64748b);
    font-size: 0.9rem; position: relative;
  }

  /* ═══════════════════════════════════════════════════════════════════
     ACTIVE CALL — CONTROLS
     ═══════════════════════════════════════════════════════════════════ */
  .call-controls {
    display: flex; justify-content: center; gap: 0.75rem;
    padding: 1rem;
    background: var(--bg-secondary, #1e293b);
    border-top: 1px solid var(--border, #334155);
  }

  .ctrl-btn {
    display: flex; flex-direction: column; align-items: center; gap: 0.25rem;
    padding: 0.75rem 1rem;
    border: none; border-radius: 0.75rem;
    background: var(--bg-tertiary, #334155);
    color: var(--text-primary, #f1f5f9);
    cursor: pointer;
    transition: all 0.15s;
    font-size: 0.75rem;
  }
  .ctrl-btn svg { width: 1.5rem; height: 1.5rem; }
  .ctrl-btn:hover { background: var(--bg-hover, #475569); }
  .ctrl-btn.active { background: var(--danger, #ef4444); color: #fff; }
  .ctrl-btn.hangup { background: var(--danger, #ef4444); color: #fff; }
  .ctrl-btn.hangup:hover { background: #dc2626; }

  /* ═══════════════════════════════════════════════════════════════════
     BUTTONS
     ═══════════════════════════════════════════════════════════════════ */
  .btn {
    display: inline-flex; align-items: center; gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    border: none; border-radius: 0.75rem;
    font-size: 1rem; font-weight: 500;
    cursor: pointer; transition: all 0.15s;
  }
  .btn svg { width: 1.25rem; height: 1.25rem; }
  .btn-primary { background: var(--accent, #3b82f6); color: #fff; }
  .btn-primary:hover { background: var(--accent-hover, #2563eb); }
  .btn-secondary { background: var(--bg-tertiary, #334155); color: var(--text-primary, #f1f5f9); }
  .btn-secondary:hover { background: var(--bg-hover, #475569); }
  .btn-danger { background: var(--danger, #ef4444); color: #fff; }
  .btn-danger:hover { background: #dc2626; }

  /* ═══════════════════════════════════════════════════════════════════
     SPINNER & PULSE
     ═══════════════════════════════════════════════════════════════════ */
  .spinner {
    width: 3rem; height: 3rem;
    border: 0.25rem solid var(--bg-tertiary, #334155);
    border-top-color: var(--accent, #3b82f6);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .pulse-ring {
    position: absolute;
    width: 5rem; height: 5rem;
    border: 2px solid var(--accent, #3b82f6);
    border-radius: 50%;
    animation: pulse-out 1.5s ease-out infinite;
    z-index: 1;
  }
  .pulse-ring.large { width: 8rem; height: 8rem; }
  @keyframes pulse-out {
    0% { transform: scale(0.5); opacity: 1; }
    100% { transform: scale(2); opacity: 0; }
  }

  /* ═══════════════════════════════════════════════════════════════════
     ERROR BANNER
     ═══════════════════════════════════════════════════════════════════ */
  .error-banner {
    position: fixed; bottom: 5rem; left: 50%; transform: translateX(-50%);
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.9);
    color: #fff; border-radius: 0.75rem;
    z-index: 1000; max-width: 90vw;
    box-shadow: 0 4px 20px rgba(0,0,0,0.3);
    font-size: 0.875rem;
  }
  .error-banner svg { width: 1.25rem; height: 1.25rem; flex-shrink: 0; }
  .error-banner button {
    background: none; border: none; color: #fff;
    cursor: pointer; font-size: 1rem; margin-left: 0.5rem;
  }

  /* ═══════════════════════════════════════════════════════════════════
     DEBUG PANEL
     ═══════════════════════════════════════════════════════════════════ */
  .debug-panel {
    position: fixed; bottom: 5rem; left: 50%; transform: translateX(-50%);
    background: var(--bg-secondary, #1e293b);
    border: 1px solid var(--border, #334155);
    border-radius: 0.75rem;
    padding: 1rem; z-index: 1000;
    box-shadow: 0 4px 20px rgba(0,0,0,0.3);
    min-width: 260px;
  }
  .debug-panel h3 { margin: 0 0 0.5rem; font-size: 0.8rem; color: var(--accent, #60a5fa); }
  .debug-panel table { width: 100%; font-size: 0.75rem; border-collapse: collapse; }
  .debug-panel td { padding: 0.2rem 0; }
  .debug-panel td:first-child { color: var(--text-muted, #64748b); padding-right: 1rem; }
  .debug-panel td:last-child { font-family: monospace; }
  .debug-hint { margin: 0.5rem 0 0; font-size: 0.625rem; color: var(--text-muted, #64748b); text-align: right; }
</style>