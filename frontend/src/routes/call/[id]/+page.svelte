<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  
  // -----------------------------------------------------------------
  // Import des runes (Svelte 5)
  // -----------------------------------------------------------------
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
  } from '$lib/conversationStore.svelte.ts';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showIncomingCallModal = $state(false);
  let incomingCallFrom     = $state('');     // user_id de l'appelant
  let incomingCallFromName = $state('');     // nom affiché dans la sonnerie
  let incomingCallConvId   = $state('');
  let incomingCallType     = $state<'audio'|'video'>('audio');

  // Ajout pour l'accessibilité : référence au contenu du modal
  let modalOverlay = $state<HTMLElement | undefined>(undefined);

  // Focus automatique sur le bouton Accepter quand le modal s'ouvre
  $effect(() => {
    if (showIncomingCallModal && modalOverlay) {
      const acceptBtn = modalOverlay.querySelector('.accept-btn') as HTMLButtonElement | null;
      if (acceptBtn) acceptBtn.focus();
    }
  });

  // -----------------------------------------------------------------
  // 2️⃣ Accès réactif aux paramètres d'URL (Svelte 5)
  // -----------------------------------------------------------------
  const conversationId = $derived($page.params?.id ?? '');
  const callType       = $derived(($page.url?.searchParams?.get('type') ?? 'audio') as 'audio'|'video');

  // BUG-CALL-6 FIX : nom de la conversation (pas l'UUID brut)
  // participants.value est le tableau des Participant[], chargé par loadParticipants()
  const convTitle = $derived(
    (() => {
      const parts = participants.value.filter((p) => p.id !== authStore.user?.id);
      if (parts.length === 0) return 'Appel';
      if (parts.length === 1) return parts[0].name ?? parts[0].username ?? 'Appel';
      return parts.map(p => p.name ?? p.username).join(', ');
    })()
  );

  // -----------------------------------------------------------------
  // 3️⃣ Cycle de vie
  // -----------------------------------------------------------------
  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    try {
      loading = true;
      error = null;
      await loadParticipants(conversationId);

      // BUG-CALL-2 FIX : participants.value (objet { value, subscribe }, pas un Array)
      if ($page.url?.searchParams?.has('call')) {
        const ids = participants.value.map((p) => p.id);
        await startGroupCall(conversationId, ids, callType);
      }

      loading = false;
      if (browser) {
        window.addEventListener('incoming-call', handleIncomingCall as EventListener);
        window.addEventListener('keydown', handleKeydown);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err) || "Erreur d'initialisation de l'appel";
      loading = false;
      console.error('Appel init error:', err);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('incoming-call', handleIncomingCall as EventListener);
      window.removeEventListener('keydown', handleKeydown);
    }
    // Stopper la sonnerie si la page est quittée
    callManager.stopRingtone();
  });

  // -----------------------------------------------------------------
  // 4️⃣ Gestion d'un appel entrant (custom event émis par webrtc-calls)
  // -----------------------------------------------------------------
  function handleIncomingCall(event: CustomEvent) {
    const { from_user_id, from_user_name, conversationId: convId, callType: ct } = event.detail;
    incomingCallFrom     = from_user_id;
    incomingCallFromName = from_user_name ?? from_user_id;
    incomingCallConvId   = convId;
    incomingCallType     = ct ?? 'audio';
    showIncomingCallModal = true;
  }

  function closeIncomingCallModal() {
    showIncomingCallModal = false;
    incomingCallFrom     = '';
    incomingCallFromName = '';
    incomingCallConvId   = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && showIncomingCallModal) {
      rejectCall();
    }
  }

  // -----------------------------------------------------------------
  // 5️⃣ Acceptation / rejet d'un appel entrant
  // -----------------------------------------------------------------
  async function acceptCall() {
    if (!incomingCallConvId) return;
    try {
      callManager.stopRingtone();
      // BUG-CALL-2 FIX : participants.value
      const ids = participants.value.map((p) => p.id);
      await startGroupCall(incomingCallConvId, ids, incomingCallType);
      closeIncomingCallModal();
    } catch (err) {
      console.error('Erreur acceptation appel :', err);
    }
  }

  function rejectCall() {
    callManager.stopRingtone();
    // Envoyer signal call_rejected à l'appelant
    callManager.sendReject(incomingCallConvId, incomingCallFrom);
    closeIncomingCallModal();
  }

  // -----------------------------------------------------------------
  // 6️⃣ Contrôles de l'appel en cours
  // -----------------------------------------------------------------
  function toggleMute()  { callManager.toggleMute(); }
  function toggleVideo() { callManager.toggleVideo(); }
  async function endCall() { await endCurrentCall(); }
</script>

<svelte:head>
  <title>Appel - Nook</title>
</svelte:head>

<div class="call-page">
  {#if loading}
    <!-- LOADING -->
    <div class="loading-container">
      <div class="loading-spinner"></div>
      <p>Préparation de l'appel…</p>
    </div>

  {:else if error}
    <!-- ERREUR GLOBALE -->
    <div class="error-container">
      <div class="error-content">
        <h1>❌ Erreur</h1>
        <p class="error-message">{error}</p>
        <button onclick={() => goto('/chat')} class="back-button">
          ← Retour au chat
        </button>
      </div>
    </div>

  {:else}
    <!-- CONTENU DE L'APPEL -->
    <div class="call-container">
      <!-- HEADER -->
      <header class="call-header">
        <div class="header-theme">
          {#if getCurrentTheme === 'jardin-secret'}
            🌿 Appel Jardin Secret
          {:else if getCurrentTheme === 'space-hub'}
            🚀 Appel Space Hub
          {:else if getCurrentTheme === 'maison-chaleureuse'}
            🏠 Appel Maison Chaleureuse
          {/if}
        </div>

        <h1 class="call-title">Appel avec {convTitle}</h1>

        {#if participants.value.length > 1}
          <p class="participants-count">{participants.value.length} participants</p>
        {/if}

        <button
          onclick={() => goto('/chat')}
          class="back-button"
          aria-label="Retour au chat"
        >
          ← Retour
        </button>
      </header>

      <!-- ÉTAT DE L'APPEL (en cours / en attente) -->
      {#if callStore.isInCall}
        <!-- CALL ACTIVE -->
        <div class="video-grid" role="region" aria-label="Participants à l'appel">
          <!-- Local stream (self) -->
          {#if callStore.localStream}
            <div class="video-participant local">
              <video
                bind:this={callStore.localVideoElement}
                autoplay
                muted
                playsinline
                class="local-video"
              ></video>

              <div class="participant-info">
                <span class="participant-name">Vous</span>
                <span
                  class="icon"
                  aria-label={callStore.isMuted ? 'Microphone coupé' : 'Microphone activé'}
                >
                  {callStore.isMuted ? '🔇' : '🔊'}
                </span>
                <span
                  class="icon"
                  aria-label={callStore.isVideoOff ? 'Vidéo désactivée' : 'Vidéo activée'}
                >
                  {callStore.isVideoOff ? '📹❌' : '📹'}
                </span>
              </div>
            </div>
          {/if}

          <!-- Remote streams -->
          {#each Array.from(callStore.remoteStreams.entries()) as [userId, stream]}
            <div class="video-participant remote">
              <video srcObject={stream} autoplay playsinline class="remote-video"></video>
              <div class="participant-info">
                <span class="participant-name">
                  {participants.value.find((p) => p.id === userId)?.name || userId}
                </span>
              </div>
            </div>
          {/each}

          {#if callStore.remoteStreams.size === 0 && !callStore.localStream}
            <div class="waiting-message">
              <p>Connexion aux participants…</p>
              <div class="spinner"></div>
            </div>
          {/if}
        </div>

        <!-- Controls -->
        <div class="call-controls" role="toolbar" aria-label="Contrôles de l'appel">
          <button
            onclick={toggleMute}
            class="control-button"
            aria-label={callStore.isMuted ? 'Activer le son' : 'Couper le son'}
          >
            {callStore.isMuted ? '🔇' : '🔊'}
          </button>

          <button
            onclick={toggleVideo}
            class="control-button"
            aria-label={callStore.isVideoOff ? 'Activer la vidéo' : 'Désactiver la vidéo'}
          >
            {callStore.isVideoOff ? '📹❌' : '📹'}
          </button>

          <button onclick={endCall} class="control-button hangup" aria-label="Raccrocher">
            📵
          </button>

          <div class="call-info">
            <span>💬 {callStore.remoteStreams.size + 1} participants</span>
            <span class="secure-badge">✅ Connexion sécurisée P2P</span>
          </div>
        </div>

      {:else if callStore.isCalling || callStore.isAnswering}
        <!-- CALL EN COURS DE SETUP -->
        <div class="call-status" role="status" aria-live="polite">
          <span class="icon large" aria-hidden="true">✆</span>

          {#if callStore.isCalling}
            <p>Appel en cours vers les participants…</p>
          {:else}
            <p>Appel entrant de {incomingCallFromName || incomingCallFrom}</p>
          {/if}

          <div class="spinner"></div>
          <p class="secure-badge">Connexion sécurisée P2P</p>
        </div>

      {:else}
        <!-- AUCUN APPEL EN COURS -->
        <div class="no-call">
          <div class="theme-icon" aria-hidden="true">
            {#if getCurrentTheme === 'jardin-secret'}
              🌸
            {:else if getCurrentTheme === 'space-hub'}
              🌌
            {:else if getCurrentTheme === 'maison-chaleureuse'}
              🏡
            {/if}
          </div>

          <h2 class="no-call-title">Aucun appel en cours</h2>
          <p class="no-call-description">
            Cette conversation n'a pas d'appel actif.
          </p>

          <div class="start-call-buttons">
            <button
              class="start-audio-call"
              onclick={async () => {
                const ids = participants.value.map((p) => p.id);
                await startGroupCall(conversationId, ids, 'audio');
              }}
              aria-label="Démarrer un appel audio avec les participants"
            >
              🎤 Démarrer un appel audio
            </button>

            <button
              class="start-video-call"
              onclick={async () => {
                const ids = participants.value.map((p) => p.id);
                await startGroupCall(conversationId, ids, 'video');
              }}
              aria-label="Démarrer un appel vidéo avec les participants"
            >
              📹 Démarrer un appel vidéo
            </button>
          </div>
        </div>
      {/if}

      <!-- ERREUR CALL STORE (ex. problème WebRTC) -->
      {#if callStore.error}
        <div class="error-modal" role="alertdialog" aria-label="Erreur">
          <p>{callStore.error}</p>
          <button
            onclick={() => (callStore.error = null)}
            aria-label="Fermer"
          >
            ✕
          </button>
        </div>
      {/if}
    </div>

    <!-- MODAL APPEL ENTRANT -->
    {#if showIncomingCallModal}
      <div
        bind:this={modalOverlay}
        class="modal-overlay"
        onclick={closeIncomingCallModal}
        role="dialog"
        aria-modal="true"
        aria-label="Appel entrant"
        tabindex="-1"
        onkeydown={handleKeydown}
      >
        <!-- eslint-disable-next-line svelte/a11y-click-events-have-key-events -->
        <!-- eslint-disable-next-line svelte/a11y-no-noninteractive-element-interactions -->
        <div
          class="incoming-call-modal"
          role="document"
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => e.stopPropagation()}
          tabindex="-1"
        >
          <div class="caller-avatar" aria-hidden="true">
            <span>✆</span>
          </div>

          <h2 class="caller-name">Appel entrant</h2>
          <p class="caller-from">De : {incomingCallFromName || incomingCallFrom}</p>
          <p class="call-info-text">Vous avez un appel entrant</p>

          <div class="call-actions">
            <button onclick={acceptCall} class="accept-btn" aria-label="Accepter l'appel">
              ✅ Accepter
            </button>

            <button onclick={rejectCall} class="reject-btn" aria-label="Rejeter l'appel">
              ❌ Rejeter
            </button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  * { box-sizing: border-box; }

  .call-page {
    min-height: 100vh;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
    padding: 1rem;
  }

  .loading-container,
  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    gap: 1rem;
    padding: 1.5rem;
  }

  .loading-spinner,
  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid #e0f2fe;
    border-top-color: #2d5a27;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-content {
    background: white;
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    text-align: center;
    max-width: 400px;
    width: 100%;
  }

  .error-content h1 {
    font-size: 1.5rem;
    margin: 0 0 0.5rem 0;
    color: #1e293b;
  }

  .error-message {
    color: #dc2626;
    margin: 0 0 1.5rem 0;
    line-height: 1.5;
  }

  .back-button {
    padding: 0.75rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    background: #3d7a37;
    transform: translateY(-1px);
  }

  .call-container {
    max-width: 1200px;
    margin: 0 auto;
  }

  .call-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    background: white;
    border-radius: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    margin-bottom: 1.5rem;
  }

  .header-theme {
    font-size: 1.125rem;
    font-weight: 600;
    color: #2d5a27;
  }

  .call-title {
    font-size: 1.25rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .participants-count {
    font-size: 0.875rem;
    color: #64748b;
  }

  .video-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .video-participant {
    position: relative;
    background: #1e293b;
    border-radius: 16px;
    overflow: hidden;
    aspect-ratio: 16/9;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .participant-info {
    position: absolute;
    bottom: 1rem;
    left: 1rem;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .participant-name {
    font-weight: 600;
  }

  .icon {
    font-size: 1.125rem;
  }

  .waiting-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    background: white;
    border-radius: 16px;
    color: #64748b;
  }

  .call-controls {
    position: fixed;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1rem;
    background: white;
    padding: 1rem 1.5rem;
    border-radius: 3rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  }

  .control-button {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: none;
    background: #f1f5f9;
    color: #1e293b;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .control-button:hover {
    background: #e2e8f0;
    transform: scale(1.05);
  }

  .control-button.hangup {
    background: #dc2626;
    color: white;
  }

  .control-button.hangup:hover {
    background: #b91c1c;
  }

  .call-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-left: 1rem;
    font-size: 0.875rem;
    color: #64748b;
  }

  .secure-badge {
    color: #2d5a27;
    font-weight: 500;
  }

  .call-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 60vh;
    gap: 1.5rem;
    text-align: center;
  }

  .icon.large {
    font-size: 4rem;
  }

  .no-call {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 60vh;
    gap: 2rem;
    text-align: center;
  }

  .theme-icon {
    font-size: 5rem;
  }

  .no-call-title {
    font-size: 2rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .no-call-description {
    color: #64748b;
    font-size: 1.125rem;
    margin: 0;
  }

  .start-call-buttons {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  .start-audio-call,
  .start-video-call {
    padding: 1rem 2rem;
    border: none;
    border-radius: 1rem;
    font-size: 1.125rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .start-audio-call {
    background: #2d5a27;
    color: white;
  }

  .start-video-call {
    background: #0ea5e9;
    color: white;
  }

  .start-audio-call:hover,
  .start-video-call:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }

  .error-modal {
    position: fixed;
    top: 2rem;
    right: 2rem;
    background: #fee2e2;
    color: #dc2626;
    padding: 1rem 1.5rem;
    border-radius: 0.75rem;
    box-shadow: 0 4px 12px rgba(220, 38, 38, 0.15);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    max-width: 400px;
    z-index: 1000;
  }

  .error-modal button {
    background: none;
    border: none;
    color: #dc2626;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .incoming-call-modal {
    background: white;
    padding: 2.5rem;
    border-radius: 1.5rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    max-width: 400px;
    width: 90%;
    text-align: center;
  }

  .caller-avatar {
    width: 80px;
    height: 80px;
    background: #2d5a27;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2.5rem;
    margin: 0 auto 1.5rem;
  }

  .caller-name {
    font-size: 1.5rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0 0 0.5rem 0;
  }

  .caller-from {
    color: #64748b;
    margin: 0 0 1rem 0;
  }

  .call-info-text {
    color: #64748b;
    margin: 0 0 2rem 0;
  }

  .call-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  .accept-btn,
  .reject-btn {
    padding: 1rem 2rem;
    border: none;
    border-radius: 0.75rem;
    font-size: 1.125rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept-btn {
    background: #2d5a27;
    color: white;
  }

  .reject-btn {
    background: #dc2626;
    color: white;
  }

  .accept-btn:hover {
    background: #3d7a37;
  }

  .reject-btn:hover {
    background: #b91c1c;
  }

  @media (max-width: 768px) {
    .call-header {
      padding: 1rem;
      flex-direction: column;
      gap: 0.75rem;
      text-align: center;
    }

    .video-grid {
      grid-template-columns: 1fr;
    }

    .call-controls {
      bottom: 1rem;
      padding: 0.75rem 1rem;
      gap: 0.5rem;
    }

    .control-button {
      width: 48px;
      height: 48px;
      font-size: 1.25rem;
    }

    .call-info {
      display: none;
    }

    .start-call-buttons {
      flex-direction: column;
      width: 100%;
      padding: 0 1rem;
    }

    .start-audio-call,
    .start-video-call {
      width: 100%;
    }
  }
</style>
