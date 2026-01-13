<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores'; // <-- IMPORT AJOUTÉ
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  
  // -----------------------------------------------------------------
  // Import des runes (Svelte 5)
  // -----------------------------------------------------------------
  import { isAuthenticated } from '$lib/authStore';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import {
    startGroupCall,
    endCurrentCall,
    callStore,
    callManager,
  } from '$lib/webrtc-calls';
  import {
    participants,
    loadParticipants,
  } from '$lib/conversationStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showIncomingCallModal = $state(false);
  let incomingCallFrom = $state('');
  let incomingCallConvId = $state('');

  // -----------------------------------------------------------------
  // 2️⃣ Accès réactif aux paramètres d'URL (Svelte 5)
  // -----------------------------------------------------------------
  /** Id de la conversation (ex. /call/[id]) */
  const conversationId = $derived($page.params?.id ?? '');

  /** Type d'appel demandé dans l'URL (`?type=audio|video`) - défaut video */
  const callType = $derived($page.url?.searchParams?.get('type') ?? 'video');

  // -----------------------------------------------------------------
  // 3️⃣ Cycle de vie - chargement et écouteurs
  // -----------------------------------------------------------------
  onMount(async () => {
    // Rediriger si l'utilisateur n'est pas authentifié
    if (!isAuthenticated) {
      goto('/login');
      return;
    }

    try {
      loading = true;
      error = null;

      // Charger les participants de la conversation
      await loadParticipants(conversationId);

      // Si l'URL contient `?call` → démarrer immédiatement l'appel
      if ($page.url?.searchParams?.has('call')) {
        const ids = participants.map((p) => p.id);
        await startGroupCall(conversationId, ids, callType);
      }

      loading = false;

      // Écouteurs globaux (uniquement côté client)
      if (browser) {
        window.addEventListener('incoming-call', handleIncomingCall as EventListener);
        window.addEventListener('keydown', handleKeydown);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err) || "Erreur d'initialisation de l'appel";
      loading = false;
      console.error('Erreur appel :', err);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('incoming-call', handleIncomingCall as EventListener);
      window.removeEventListener('keydown', handleKeydown);
    }
  });

  // -----------------------------------------------------------------
  // 4️⃣ Gestion d'un appel entrant (custom event)
  // -----------------------------------------------------------------
  function handleIncomingCall(event: CustomEvent) {
    const { from_user_id, conversationId: convId } = event.detail;
    incomingCallFrom = from_user_id;
    incomingCallConvId = convId;
    showIncomingCallModal = true;
  }

  /** Ferme le modal d'appel entrant. */
  function closeIncomingCallModal() {
    showIncomingCallModal = false;
    incomingCallFrom = '';
    incomingCallConvId = '';
  }

  /** Gestion du `Esc` uniquement quand le modal est ouvert. */
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && showIncomingCallModal) {
      closeIncomingCallModal();
    }
  }

  // -----------------------------------------------------------------
  // 5️⃣ Acceptation / rejet d'un appel entrant
  // -----------------------------------------------------------------
  async function acceptCall() {
    if (!incomingCallConvId) return;

    try {
      const ids = participants.map((p) => p.id);
      await startGroupCall(incomingCallConvId, ids, 'audio');
      closeIncomingCallModal();
    } catch (err) {
      console.error('Erreur acceptation appel :', err);
    }
  }

  function rejectCall() {
    closeIncomingCallModal();
  }

  // -----------------------------------------------------------------
  // 6️⃣ Contrôles de l'appel en cours
  // -----------------------------------------------------------------
  function toggleMute() {
    callManager.toggleMute();
  }

  function toggleVideo() {
    callManager.toggleVideo();
  }

  async function endCall() {
    await endCurrentCall();
  }
</script>

<svelte:head>
  <title>Appel - Nook</title>
</svelte:head>

<div class="call-page">
  {#if loading}
    <!-- -----------------------------------------------------------------
         LOADING
         ----------------------------------------------------------------- -->
    <div class="loading-container">
      <div class="loading-spinner"></div>
      <p>Préparation de l'appel…</p>
    </div>

  {:else if error}
    <!-- -----------------------------------------------------------------
         ERREUR GLOBALE
         ----------------------------------------------------------------- -->
    <div class="error-container">
      <div class="error-content">
        <h1>❌ Erreur</h1>
        <p class="error-message">{error}</p>
        <button on:click={() => goto('/chat')} class="back-button">
          ← Retour au chat
        </button>
      </div>
    </div>

  {:else}
    <!-- -----------------------------------------------------------------
         CONTENU DE L'APPEL
         ----------------------------------------------------------------- -->
    <div class="call-container">
      <!-- HEADER -->
      <header class="call-header">
        <div class="header-theme">
          {#if currentTheme === 'jardin-secret'}
            🌿 Appel Jardin Secret
          {:else if currentTheme === 'space-hub'}
            🚀 Appel Space Hub
          {:else if currentTheme === 'maison-chaleureuse'}
            🏠 Appel Maison Chaleureuse
          {/if}
        </div>

        <h1 class="call-title">Appel avec {conversationId}</h1>

        {#if participants.length > 1}
          <p class="participants-count">{participants.length} participants</p>
        {/if}

        <button
          on:click={() => goto('/chat')}
          class="back-button"
          aria-label="Retour au chat"
        >
          ← Retour
        </button>
      </header>

      <!-- -----------------------------------------------------------------
           ÉTAT DE L'APPEL (en cours / en attente)
           ----------------------------------------------------------------- -->
      {#if callStore.isInCall}
        <!-- ------------------- CALL ACTIVE ------------------- -->
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
                  {participants.find((p) => p.id === userId)?.name || userId}
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
            on:click={toggleMute}
            class="control-button"
            aria-label={callStore.isMuted ? 'Activer le son' : 'Couper le son'}
          >
            {callStore.isMuted ? '🔇' : '🔊'}
          </button>

          <button
            on:click={toggleVideo}
            class="control-button"
            aria-label={callStore.isVideoOff ? 'Activer la vidéo' : 'Désactiver la vidéo'}
          >
            {callStore.isVideoOff ? '📹❌' : '📹'}
          </button>

          <button on:click={endCall} class="control-button hangup" aria-label="Raccrocher">
            📵
          </button>

          <div class="call-info">
            <span>💬 {callStore.remoteStreams.size + 1} participants</span>
            <span class="secure-badge">✅ Connexion sécurisée P2P</span>
          </div>
        </div>

      {:else if callStore.isCalling || callStore.isAnswering}
        <!-- ------------------- CALL EN COURS DE SETUP ------------------- -->
        <div class="call-status" role="status" aria-live="polite">
          <span class="icon large" aria-hidden="true">✆</span>

          {#if callStore.isCalling}
            <p>Appel en cours vers les participants…</p>
          {:else}
            <p>Appel entrant de {incomingCallFrom}</p>
          {/if}

          <div class="spinner"></div>
          <p class="secure-badge">Connexion sécurisée P2P</p>
        </div>

      {:else}
        <!-- ------------------- AUCUN APPEL EN COURS ------------------- -->
        <div class="no-call">
          <div class="theme-icon" aria-hidden="true">
            {#if currentTheme === 'jardin-secret'}
              🌸
            {:else if currentTheme === 'space-hub'}
              🌌
            {:else if currentTheme === 'maison-chaleureuse'}
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
              on:click={async () => {
                const ids = participants.map((p) => p.id);
                await startGroupCall(conversationId, ids, 'audio');
              }}
              aria-label="Démarrer un appel audio avec les participants"
            >
              🎤 Démarrer un appel audio
            </button>

            <button
              class="start-video-call"
              on:click={async () => {
                const ids = participants.map((p) => p.id);
                await startGroupCall(conversationId, ids, 'video');
              }}
              aria-label="Démarrer un appel vidéo avec les participants"
            >
              📹 Démarrer un appel vidéo
            </button>
          </div>
        </div>
      {/if}

      <!-- -----------------------------------------------------------------
           ERREUR CALL STORE (ex. problème WebRTC)
           ----------------------------------------------------------------- -->
      {#if callStore.error}
        <div class="error-modal" role="alertdialog" aria-label="Erreur">
          <p>{callStore.error}</p>
          <button
            on:click={() => callStore.error = null}
            aria-label="Fermer"
          >
            ✕
          </button>
        </div>
      {/if}

      <!-- -----------------------------------------------------------------
           MODAL APPEL ENTRANT
           ----------------------------------------------------------------- -->
      {#if showIncomingCallModal}
        <div
          class="modal-overlay"
          on:click={closeIncomingCallModal}
          role="dialog"
          aria-modal="true"
          aria-label="Appel entrant"
          tabindex="0"
          on:keydown={handleKeydown}
        >
          <div
            class="incoming-call-modal"
            on:click|stopPropagation
            tabindex="-1"
          >
            <div class="caller-avatar" aria-hidden="true">
              <span>✆</span>
            </div>

            <h2 class="caller-name">Appel entrant</h2>
            <p class="caller-from">De : {incomingCallFrom}</p>
            <p class="call-info-text">Vous avez un appel entrant</p>

            <div class="call-actions">
              <button
                on:click={acceptCall}
                class="accept-btn"
                aria-label="Accepter l'appel"
              >
                ✅ Accepter
              </button>

              <button
                on:click={rejectCall}
                class="reject-btn"
                aria-label="Rejeter l'appel"
              >
                ❌ Rejeter
              </button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  * { box-sizing: border-box; } /* ← Fix global overflow */

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

  /* -----------------------------------------------------------------
     HEADER
     ----------------------------------------------------------------- */
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

  .call-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .call-title {
    font-size: 1.25rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .call-status {
    font-size: 0.875rem;
    color: #64748b;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #2d5a27;
    animation: pulse 2s infinite;
  }

  .status-indicator.connecting {
    background: #f59e0b;
  }

  .status-indicator.disconnected {
    background: #dc2626;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  /* -----------------------------------------------------------------
     VIDEO SECTION
     ----------------------------------------------------------------- */
  .video-section {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  @media (min-width: 1024px) {
    .video-section {
      grid-template-columns: 1fr 1fr;
    }
  }

  .video-container {
    position: relative;
    background: #1e293b;
    border-radius: 16px;
    overflow: hidden;
    aspect-ratio: 16/9;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
    background: #0f172a;
  }

  .video-overlay {
    position: absolute;
    bottom: 1rem;
    left: 0;
    right: 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 1.5rem;
  }

  .user-info {
    background: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 2rem;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    backdrop-filter: blur(8px);
  }

  .user-avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: #2d5a27;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .video-stats {
    background: rgba(0, 0, 0, 0.7);
    color: #cbd5e1;
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.75rem;
    backdrop-filter: blur(8px);
  }

  .muted-indicator {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: rgba(220, 38, 38, 0.8);
    color: white;
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    backdrop-filter: blur(8px);
  }

  /* -----------------------------------------------------------------
     CONTROLS
     ----------------------------------------------------------------- */
  .controls-container {
    position: fixed;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 1rem;
    z-index: 100;
  }

  @media (max-width: 768px) {
    .controls-container {
      bottom: 1rem;
      gap: 0.75rem;
    }
  }

  .control-button {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    background: white;
    box-shadow: 0 4px 20px rgba(0,0,0,0.15);
  }

  @media (max-width: 768px) {
    .control-button {
      width: 48px;
      height: 48px;
    }
  }

  .control-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 24px rgba(0,0,0,0.2);
  }

  .control-button:active {
    transform: translateY(0);
  }

  .control-button.active {
    background: #2d5a27;
    color: white;
  }

  .control-button.active:hover {
    background: #3d7a37;
  }

  .control-button.muted {
    background: #dc2626;
    color: white;
  }

  .control-button.muted:hover {
    background: #b91c1c;
  }

  .control-icon {
    width: 24px;
    height: 24px;
  }

  @media (max-width: 768px) {
    .control-icon {
      width: 20px;
      height: 20px;
    }
  }

  .end-call-button {
    background: #dc2626;
    color: white;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 4px 20px rgba(220, 38, 38, 0.3);
  }

  @media (max-width: 768px) {
    .end-call-button {
      width: 48px;
      height: 48px;
    }
  }

  .end-call-button:hover {
    background: #b91c1c;
    transform: translateY(-2px);
    box-shadow: 0 6px 24px rgba(220, 38, 38, 0.4);
  }

  /* -----------------------------------------------------------------
     PARTICIPANTS
     ----------------------------------------------------------------- */
  .participants-container {
    background: white;
    border-radius: 16px;
    padding: 1.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    margin-bottom: 5rem; /* Pour éviter que les contrôles ne cachent */
  }

  .participants-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  .participants-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1e293b;
    margin: 0;
  }

  .participants-count {
    background: #f1f5f9;
    color: #64748b;
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .participants-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
  }

  @media (max-width: 768px) {
    .participants-list {
      grid-template-columns: 1fr;
    }
  }

  .participant-card {
    background: #f8fafc;
    border-radius: 12px;
    padding: 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    transition: all 0.2s;
  }

  .participant-card:hover {
    background: #f1f5f9;
    transform: translateY(-2px);
  }

  .participant-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #2d5a27;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    color: white;
    flex-shrink: 0;
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    font-weight: 600;
    color: #1e293b;
    margin: 0 0 0.25rem 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .participant-status {
    font-size: 0.75rem;
    color: #64748b;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #2d5a27;
  }

  .status-dot.muted {
    background: #dc2626;
  }

  /* -----------------------------------------------------------------
     CHAT TOGGLE (OPTIONNEL)
     ----------------------------------------------------------------- */
  .chat-toggle {
    position: fixed;
    right: 2rem;
    bottom: 2rem;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: white;
    border: none;
    box-shadow: 0 4px 20px rgba(0,0,0,0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 100;
  }

  @media (max-width: 768px) {
    .chat-toggle {
      right: 1rem;
      bottom: 1rem;
      width: 48px;
      height: 48px;
    }
  }

  .chat-toggle:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 24px rgba(0,0,0,0.2);
  }

  .chat-badge {
    position: absolute;
    top: -4px;
    right: -4px;
    background: #dc2626;
    color: white;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 768px) {
    .call-header {
      padding: 1rem;
      flex-direction: column;
      gap: 0.75rem;
      align-items: stretch;
    }

    .video-section {
      gap: 1rem;
    }

    .video-overlay {
      padding: 0 1rem;
      flex-direction: column;
      gap: 0.5rem;
      align-items: flex-start;
    }

    .participants-container {
      padding: 1rem;
      margin-bottom: 4rem;
    }
  }

  /* -----------------------------------------------------------------
     UTILITY CLASSES
     ----------------------------------------------------------------- */
  .hidden {
    display: none !important;
  }

  .text-success { color: #2d5a27; }
  .text-warning { color: #f59e0b; }
  .text-error { color: #dc2626; }

  .bg-success { background: #2d5a27; }
  .bg-warning { background: #f59e0b; }
  .bg-error { background: #dc2626; }

  .flex-center {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .flex-between {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
</style>