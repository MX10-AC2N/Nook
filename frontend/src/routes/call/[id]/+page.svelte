<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { authStore } from '$lib/authStore.svelte.js';
  import { getCurrentTheme } from '$lib/ui/ThemeStore.svelte.ts';
  import {
    startGroupCall,
    endCurrentCall,
    callStore,
    callManager,
  } from '$lib/webrtc-calls.svelte.ts';
  import {
    participants,
    loadParticipants,
    conversations,
  } from '$lib/conversationStore.svelte.ts';

  // ════════════════════════════════════════════════════
  // États locaux
  // ════════════════════════════════════════════════════
  let loading = $state(true);
  let error: string | null = $state(null);
  let callDuration = $state(0);
  let timerInterval: ReturnType<typeof setInterval> | null = null;
  let callStartedAt = $state(0);
  let showDebugPanel = $state(false);
  let qualityInterval: ReturnType<typeof setInterval> | null = null;

  // ════════════════════════════════════════════════════
  // Dérivés réactifs
  // ════════════════════════════════════════════════════
  const conversationId = $derived(($page.params.id as string) ?? '');
  const callType = $derived((($page.url?.searchParams?.get('type') ?? 'audio') as 'audio' | 'video'));

  // Titre de l'appel : nom de la conversation OU noms des participants
  // Helper fonctions pour $derived (Svelte 5 restriction)
  function _computeCallTitle(): string {
    const conv = conversations.find((c) => c.id === conversationId);
    if (conv?.name && conv.name !== 'Groupe Global') return conv.name;
    const others = participants.value.filter((p) => p.id !== authStore.user?.id);
    if (others.length === 0) return 'Appel';
    if (others.length === 1) return others[0].name ?? others[0].username ?? 'Appel';
    return `${others.length} participants`;
  }

  // Titre de l'appel
  const callTitle = $derived(_computeCallTitle());

  // Formatage de la durée d'appel MM:SS
  const formattedDuration = $derived(
    `${Math.floor(callDuration / 60).toString().padStart(2, '0')}:${(callDuration % 60).toString().padStart(2, '0')}`
  );

  // Participant connecté (pour affichage vidéo local)
  const localName = $derived(authStore.user?.name ?? authStore.user?.username ?? 'Moi');
  const isVideo = $derived(callType === 'video');

  // ════════════════════════════════════════════════════
  // Cycle de vie
  // ════════════════════════════════════════════════════
  onMount(async () => {
    if (!authStore.isAuthenticated) {
      goto('/login');
      return;
    }
    loading = true;
    error = null;
    try {
      await loadParticipants(conversationId);
      if (browser) {
        window.addEventListener('keydown', handleKeydown);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : "Erreur d'initialisation";
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('keydown', handleKeydown);
    }
    callManager.stopRingtone();
    if (timerInterval) {
      clearInterval(timerInterval);
    }
    if (qualityInterval) {
      clearInterval(qualityInterval);
      qualityInterval = null;
    }
  });

  // Redémarrer le timer quand on passe en isInCall
  $effect(() => {
    if (callStore.isInCall && callStartedAt === 0) {
      callStartedAt = Math.floor(Date.now() / 1000);
      timerInterval = setInterval(() => {
        callDuration = Math.floor(Date.now() / 1000) - callStartedAt;
      }, 1000);
    } else if (!callStore.isInCall) {
      if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
      }
      callStartedAt = 0;
      callDuration = 0;
    }
  });
  
  // Start quality monitoring when in call
  $effect(() => {
    if (callStore.isInCall) {
      qualityInterval = setInterval(() => {
        callManager.updateCallQuality();
      }, 2000);
      // Initial call
      callManager.updateCallQuality();
    } else {
      if (qualityInterval) {
        clearInterval(qualityInterval);
        qualityInterval = null;
      }
    }
  });
  
  // Keyboard shortcut: Ctrl+D to toggle debug panel
  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (e.ctrlKey && e.key === 'd') {
        e.preventDefault();
        showDebugPanel = !showDebugPanel;
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  // ════════════════════════════════════════════════════
  // Contrôles
  // ════════════════════════════════════════════════════
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (callStore.isInCall || callStore.isCalling) {
        endCall();
      }
    }
  }

  async function startCall(type: 'audio' | 'video') {
    try {
      error = null;
      const ids = participants.value.map((p) => p.id);
      await startGroupCall(conversationId, ids, type);
    } catch (err) {
      error = err instanceof Error ? err.message : "Erreur de démarrage de l'appel";
    }
  }

  async function endCall() {
    callManager.stopRingtone();
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    await endCurrentCall();
    goto('/chat');
  }

  function toggleMute() {
    callManager.toggleMute();
  }

  function toggleVideo() {
    callManager.toggleVideo();
  }

  function toggleScreenShare() {
    callManager.toggleScreenShare();
  }

  function goBack() {
    goto('/chat');
  }

  function toggleSfuMode() {
    if (callStore.useSfu) {
      callManager.stopSfuMode();
    } else {
      const nonSelf = participants.value.filter((p) => p.id !== authStore.user?.id);
      if (nonSelf.length >= 2) {
        callManager.startSfuCall(conversationId, nonSelf.map((p) => p.id), callType);
      } else {
        callStore.error = 'SFU requires at least 3 participants';
      }
    }
  }

</script>

<svelte:head>
  <title>Appel - Nook</title>
</svelte:head>

<div class="call-page" data-theme={getCurrentTheme()}>
  {#if loading}
    <div class="loading">
      <div class="spinner" />
      <p>Préparation de l'appel…</p>
    </div>

  {:else if error}
    <div class="error">
      <p class="error-icon">⚠️</p>
      <p class="error-text">{error}</p>
      <button onclick={goBack} class="btn btn-secondary">← Retour au chat</button>
    </div>

  {:else}
    <!-- ═══ HEADER ═══ -->
    <header class="call-header">
      <button onclick={goBack} class="btn-back" aria-label="Retour">←</button>
      <div class="call-header-info">
        <h1 class="call-title">{callTitle}</h1>
        {#if callStore.isInCall || callStartedAt > 0}
          <span class="call-timer">{formattedDuration}</span>

          {#if callStore.useSfu}
            <span class="sfu-badge" title="Mode SFU (serveur relais)">
              🌐 SFU
              {#if callStore.sfuPeers.length > 0}
                · {callStore.sfuPeers.length} pairs
              {/if}
            </span>
          {/if}

          {#if callStore.isInCall}
            <span class="quality-badge" class:good={callStore.callQuality === 'good'} class:fair={callStore.callQuality === 'fair'} class:poor={callStore.callQuality === 'poor'}>
              {#if callStore.callQuality === 'good'}✅
              {:else if callStore.callQuality === 'fair'}⚠️
              {:else if callStore.callQuality === 'poor'}🔴
              {:else}⏳
              {/if}
            </span>
          {/if}
        {/if}
      </div>
      <div class="call-type-icon">
        {isVideo ? '📹' : '📞'}
      </div>
    </header>

    <!-- ═══ EN APPEL ═══ -->
    {#if callStore.isInCall}
      <div class="call-active">
        <!-- Grille vidéo / audio -->
        <div class="participants-grid" class:video-mode={isVideo} class:screen-share-mode={callStore.isScreenSharing}>
          <!-- Screen share -->
          {#if callStore.isScreenSharing}
            <div class="participant-card screen-share">
              <video
                bind:this={callStore.screenShareLocalVideoElement}
                autoplay
                muted
                playsinline
                class="video-stream"
              />
              <div class="participant-overlay">
                <span class="participant-name">🖥️ Partage d'ecran</span>
                <button onclick={toggleScreenShare} class="stop-share-btn" aria-label="Arreter le partage">✕</button>
              </div>
            </div>
          {/if}

          <!-- Local stream -->
          <div class="participant-card local" class:without-video={!isVideo} class:share-pip={callStore.isScreenSharing}>
            {#if isVideo}
              <video
                bind:this={callStore.localVideoElement}
                autoplay
                muted
                playsinline
                class="video-stream"
              />
            {/if}
            <div class="participant-overlay">
              <span class="participant-name">{localName} (vous)</span>
              <div class="participant-status">
                {#if callStore.isMuted}<span class="badge muted">🔇</span>{/if}
                {#if callStore.isVideoOff && isVideo}<span class="badge cam-off">📷❌</span>{/if}
              </div>
            </div>
          </div>

          <!-- Remote streams -->
          {#each Array.from(callStore.remoteStreams.entries()) as [userId, stream]}
            {@const participant = participants.value.find((p) => p.id === userId)}
              <div class="participant-card remote" class:without-video={!isVideo}>
                {#if isVideo && !callStore.isVideoOff}
                  <video
                    srcObject={stream}
                    autoplay
                    playsinline
                    class="video-stream"
                  />
                {/if}
                <div class="participant-overlay">
                  <span class="participant-name">{participant?.name ?? participant?.username ?? userId}</span>
                </div>
              </div>
            
          {/each}

          <!-- Waiting state -->
          {#if callStore.remoteStreams.size === 0 && !callStore.localStream}
            <div class="waiting">
              <div class="waiting-icon">⏳</div>
              <p>Connexion en cours…</p>
              <p class="waiting-hint">En attente des autres participants</p>
            </div>
          {/if}
        </div>

        <!-- Contrôles -->
        <div class="call-controls">
          <button
            onclick={toggleMute}
            class="ctrl-btn"
            class:active={callStore.isMuted}
            aria-label={callStore.isMuted ? 'Activer micro' : 'Couper micro'}
          >
            <span class="ctrl-icon">{callStore.isMuted ? '🔇' : '🎤'}</span>
            <span class="ctrl-label">{callStore.isMuted ? 'Micro coupé' : 'Micro'}</span>
          </button>

          <button
            onclick={toggleVideo}
            class="ctrl-btn"
            class:active={callStore.isVideoOff}
            aria-label={callStore.isVideoOff ? 'Activer vidéo' : 'Couper vidéo'}
          >
            <span class="ctrl-icon">{callStore.isVideoOff ? '📷❌' : '📹'}</span>
            <span class="ctrl-label">{callStore.isVideoOff ? 'Vidéo coupée' : 'Vidéo'}</span>
          </button>

          <button
            onclick={toggleScreenShare}
            class="ctrl-btn"
            class:active={callStore.isScreenSharing}
            aria-label={callStore.isScreenSharing ? 'Arreter partage ecran' : 'Partager ecran'}
          >
            <span class="ctrl-icon">{callStore.isScreenSharing ? '🖥️❌' : '🖥️'}</span>
            <span class="ctrl-label">Ecran</span>
          </button>


          {#if callStore.sfuPeers.length >= 2}
            <button
              onclick={toggleSfuMode}
              class="ctrl-btn"
              class:active={callStore.useSfu}
              aria-label={callStore.useSfu ? 'Mode P2P' : 'Mode SFU'}
            >
              <span class="ctrl-icon">{callStore.useSfu ? '🌐' : '🔗'}</span>
              <span class="ctrl-label">{callStore.useSfu ? 'P2P' : 'SFU'}</span>
            </button>
          {/if}

          <button onclick={endCall} class="ctrl-btn hangup" aria-label="Raccrocher">
            <span class="ctrl-icon">📵</span>
            <span class="ctrl-label">Raccrocher</span>
          </button>
        </div>
      </div>

    <!-- ═══ APPEL EN COURS DE SETUP ═══ -->
    {:else if callStore.isCalling}
      <div class="call-setup">
        <div class="setup-icon">📞</div>
        <p class="setup-text">Appel en cours…</p>
        <div class="spinner" />
        <button onclick={endCall} class="btn btn-danger">Annuler</button>
      </div>

    <!-- ═══ AUCUN APPEL ═══ -->
    {:else}
      <div class="call-idle">
        <div class="idle-icon">📞</div>
        <h2 class="idle-title">Appeler {callTitle}</h2>
        <p class="idle-subtitle">
          {participants.value.filter((p) => p.id !== authStore.user?.id).length} participant{#if participants.value.filter((p) => p.id !== authStore.user?.id).length !== 1}s{/if}
          dans cet appel
        </p>

        <div class="idle-buttons">
          <button onclick={() => startCall('audio')} class="btn btn-audio">
            🎤 Appel audio
          </button>
          {#if isVideo}
            <button onclick={() => startCall('video')} class="btn btn-video">
              📹 Appel vidéo
            </button>
          {:else}
            <button onclick={() => startCall('video')} class="btn btn-video">
              📹 Ajouter la vidéo
            </button>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ DEBUG PANEL ═══ -->
    {#if showDebugPanel && callStore.isInCall}
    <div class="debug-panel">
      <h3>Qualite d'appel</h3>
      <table>
        <tbody>
        <tr><td>Qualite:</td><td>{callStore.callQuality}</td></tr>
        <tr><td>RTT (latence):</td><td>{callStore.rtt} ms</td></tr>
        <tr><td>Jitter:</td><td>{callStore.jitter} ms</td></tr>
        <tr><td>Pertes paquets:</td><td>{callStore.packetsLost}</td></tr>
        <tr><td>Resolution distante:</td><td>{callStore.remoteResolution ?? '&mdash;'}</td></tr>
        <tr><td>FPS distant:</td><td>{callStore.remoteFps}</td></tr>
        <tr><td>Connexions paires:</td><td>{callStore.peerConnections.size}</td></tr>
        </tbody>
      </table>
      <p class="debug-hint">Ctrl+D pour fermer</p>
    </div>
    {/if}

    <!-- ═══ ERREUR STORE ═══ -->
    {#if callStore.error}
      <div class="error-banner">
        <span>{callStore.error}</span>
        <button onclick={() => (callStore.error = null)}>✕</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  /* ════════════════════════════════════════════════════
     LAYOUT
     ════════════════════════════════════════════════════ */
  .call-page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    color: var(--text-primary);
    transition: background 0.3s;
  }

  /* ════════════════════════════════════════════════
     HEADER
     ════════════════════════════════════════════════ */
  .call-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .btn-back {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--text-primary);
    cursor: pointer;
    padding: 0.25rem 0.75rem;
    border-radius: 0.5rem;
  }
  .btn-back:hover {
    background: var(--bg-hover);
  }

  .call-header-info {
    flex: 1;
    min-width: 0;
  }

  .call-title {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .call-timer {
    display: inline-block;
    font-size: 0.875rem;
    color: var(--text-muted);
    margin-top: 0.25rem;
    font-variant-numeric: tabular-nums;
  }

  .call-type-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  /* ════════════════════════════════════════════════
     QUALITY BADGE
     ════════════════════════════════════════════════ */
  .quality-badge {
    font-size: 0.875rem;
    margin-left: 0.5rem;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background: var(--bg-tertiary);
  }
  .quality-badge.good { color: #22c55e; }
  .quality-badge.fair { color: #f59e0b; }
  .quality-badge.poor { color: #ef4444; }

  /* ════════════════════════════════════════════════
     DEBUG PANEL
     ════════════════════════════════════════════════ */
  .debug-panel {
    position: fixed;
    bottom: 5rem;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 1rem;
    z-index: 1000;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    max-width: 90vw;
    min-width: 280px;
  }
  .debug-panel h3 {
    margin: 0 0 0.5rem;
    font-size: 0.875rem;
    color: var(--accent);
  }
  .debug-panel table {
    width: 100%;
    font-size: 0.75rem;
    border-collapse: collapse;
  }
  .debug-panel td {
    padding: 0.25rem 0;
    border-bottom: 1px solid var(--border, rgba(255,255,255,0.1));
  }
  .debug-panel td:first-child {
    color: var(--text-secondary);
    font-weight: 500;
    padding-right: 1rem;
  }
  .debug-panel td:last-child {
    color: var(--text-primary);
    font-family: monospace;
  }
  .debug-hint {
    margin: 0.5rem 0 0;
    font-size: 0.625rem;
    color: var(--text-muted);
    text-align: right;
  }

  /* ════════════════════════════════════════════════
     LOADING & ERROR
     ════════════════════════════════════════════════ */
  .loading, .error {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
  }

  .spinner {
    width: 3rem;
    height: 3rem;
    border: 0.25rem solid var(--bg-tertiary);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .error-icon { font-size: 3rem; }
  .error-text { color: var(--text-danger); text-align: center; }

  /* ════════════════════════════════════════════════
     CALL ACTIVE - GRILLE
     ════════════════════════════════════════════════ */
  .call-active {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .participants-grid {
    flex: 1;
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    align-content: center;
  }

  .participants-grid.video-mode {
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  }

  .participant-card {
    position: relative;
    border-radius: 1rem;
    overflow: hidden;
    background: var(--bg-tertiary);
    aspect-ratio: 16 / 9;
    min-height: 180px;
  }

  .participant-card.without-video {
    aspect-ratio: auto;
    min-height: 120px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .video-stream {
    width: 100%;
    height: 100%;
    object-fit: cover;
    background: #000;
  }

  .participant-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 0.75rem 1rem;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .participant-card.without-video .participant-overlay {
    position: relative;
    background: none;
    justify-content: center;
    gap: 0.5rem;
  }

  .participant-card.without-video .participant-name {
    font-size: 1.25rem;
  }

  .participant-name {
    font-weight: 600;
    font-size: 0.875rem;
    color: white;
  }

  .participant-status {
    display: flex;
    gap: 0.35rem;
  }

  .badge {
    font-size: 0.875rem;
  }

  .waiting {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    min-height: 300px;
  }

  .waiting-icon { font-size: 3rem; }
  .waiting-hint { font-size: 0.875rem; color: var(--text-muted); }

  /* ════════════════════════════════════════════════
     SCREEN SHARE MODE
     ════════════════════════════════════════════════ */
  .participants-grid.screen-share-mode {
    grid-template-columns: 1fr;
  }

  .participant-card.screen-share {
    grid-column: 1 / -1;
    aspect-ratio: 16 / 9;
    min-height: 300px;
  }

  .participant-card.screen-share .video-stream {
    object-fit: contain;
    background: #000;
  }

  .participant-card.local.share-pip {
    position: fixed;
    bottom: 80px;
    right: 1rem;
    width: 200px;
    aspect-ratio: 16 / 9;
    z-index: 50;
    border: 2px solid var(--accent);
    border-radius: 0.75rem;
    overflow: hidden;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }

  .participant-card.local .video-stream {
    width: 100%;
    height: 100%;
  }

  .stop-share-btn {
    background: rgba(220, 38, 38, 0.9);
    border: none;
    color: white;
    font-size: 1rem;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
  }

  .stop-share-btn:hover {
    background: #dc2626;
  }

  @media (max-width: 768px) {
    .participant-card.local.share-pip {
      width: 140px;
      bottom: 70px;
      right: 0.5rem;
    }
  }

  /* ════════════════════════════════════════════════
     CONTROLES
     ════════════════════════════════════════════════ */
  .call-controls {
    display: flex;
    justify-content: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
  }

  .ctrl-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding: 0.75rem 1rem;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 0.75rem;
    cursor: pointer;
    color: var(--text-primary);
    transition: all 0.15s;
    min-width: 70px;
  }
  .ctrl-btn:hover {
    background: var(--bg-hover);
    transform: translateY(-2px);
  }
  .ctrl-btn.active {
    background: var(--accent-danger);
    color: white;
  }
  .ctrl-btn.hangup {
    background: #dc2626;
    color: white;
  }
  .ctrl-btn.hangup:hover {
    background: #b91c1c;
  }

  .ctrl-icon { font-size: 1.5rem; }
  .ctrl-label { font-size: 0.7rem; font-weight: 500; }

  /* ════════════════════════════════════════════════
     CALL SETUP
     ════════════════════════════════════════════════ */
  .call-setup {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.5rem;
  }
  .setup-icon { font-size: 4rem; }
  .setup-text { font-size: 1.25rem; font-weight: 600; }

  /* ════════════════════════════════════════════════
     IDLE - Aucun appel
     ════════════════════════════════════════════════ */
  .call-idle {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.5rem;
    padding: 2rem;
    text-align: center;
  }
  .idle-icon { font-size: 4rem; }
  .idle-title { font-size: 1.5rem; font-weight: 700; margin: 0; }
  .idle-subtitle { color: var(--text-muted); margin: 0; }

  .idle-buttons {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  /* ════════════════════════════════════════════════
     BOUTONS
     ════════════════════════════════════════════════ */
  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .btn-danger {
    background: #dc2626;
    color: white;
  }
  .btn-audio {
    background: var(--accent);
    color: white;
    padding: 1rem 2rem;
    font-size: 1.125rem;
  }
  .btn-video {
    background: #0ea5e9;
    color: white;
    padding: 1rem 2rem;
    font-size: 1.125rem;
  }

  /* ════════════════════════════════════════════════
     ERROR BANNER
     ════════════════════════════════════════════════ */
  .error-banner {
    position: fixed;
    top: 1rem;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-danger);
    color: var(--text-danger);
    padding: 0.75rem 1.25rem;
    border-radius: 0.75rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    z-index: 100;
    box-shadow: 0 4px 12px rgba(220, 38, 38, 0.2);
  }
  .error-banner button {
    background: none;
    border: none;
    color: var(--text-danger);
    font-size: 1.25rem;
    cursor: pointer;
  }

  /* ════════════════════════════════════════════════
     RESPONSIVE
     ════════════════════════════════════════════════ */
  @media (max-width: 768px) {
    .call-header {
      padding: 0.75rem 1rem;
    }
    .participants-grid {
      grid-template-columns: 1fr;
      padding: 0.75rem;
      gap: 0.5rem;
    }
    .call-controls {
      gap: 0.5rem;
      padding: 0.75rem 0.5rem;
    }
    .ctrl-btn {
      min-width: 60px;
      padding: 0.5rem;
    }
    .ctrl-label { display: none; }
    .idle-buttons {
      flex-direction: column;
      width: 100%;
    }
    .idle-buttons .btn {
      width: 100%;
    }
  }

  .sfu-badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    background: var(--accent, #4ade80);
    color: #000;
    border-radius: 0.5rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

</style>