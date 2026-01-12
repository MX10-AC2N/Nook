<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { state } from 'svelte'; // <-- Svelte 5 reactive state

  // -----------------------------------------------------------------
  // Stores
  // -----------------------------------------------------------------
  import { authStore } from '$lib/authStore';
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
    activeConversationId,
  } from '$lib/conversationStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let loading = state(true);
  let error = state<string | null>(null);
  let showIncomingCallModal = state(false);
  let incomingCallFrom = state('');
  let incomingCallConvId = state('');

  // -----------------------------------------------------------------
  // 2️⃣ Paramètres d’URL (reactif)
  // -----------------------------------------------------------------
  /** Id de la conversation (ex. /call/[id]) */
  $: conversationId = $page.params?.id ?? '';

  /** Type d’appel demandé dans l’URL (`?type=audio|video`) – défaut video */
  $: callType = $page.url?.searchParams?.get('type') ?? 'video';

  // -----------------------------------------------------------------
  // 3️⃣ Cycle de vie – chargement et écouteurs
  // -----------------------------------------------------------------
  onMount(async () => {
    // Rediriger si l’utilisateur n’est pas authentifié
    if (!$authStore.isAuthenticated) {
      goto('/login');
      return;
    }

    try {
      loading = true;
      error = null;

      // Charger les participants de la conversation
      await loadParticipants(conversationId);

      // Si l’URL contient `?call` → démarrer immédiatement l’appel
      if ($page.url?.searchParams?.has('call')) {
        const ids = $participants.map((p) => p.id);
        await startGroupCall(conversationId, ids, callType);
      }

      loading = false;

      // ----- Écouteurs globaux (uniquement côté client) -----
      if (browser) {
        window.addEventListener('incoming-call', handleIncomingCall as EventListener);
        window.addEventListener('keydown', handleKeydown);
      }
    } catch (err) {
      error =
        err instanceof Error ? err.message : String(err) || "Erreur d'initialisation de l'appel";
      loading = false;
      console.error('Erreur appel :', err);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('incoming-call', handleIncomingCall as EventListener);
      window.removeEventListener('keydown', handleKeydown);
    }
  });

  // -----------------------------------------------------------------
  // 4️⃣ Gestion d’un appel entrant (custom event)
  // -----------------------------------------------------------------
  /**
   * Payload attendu :
   * `{ from_user_id: string, conversationId: string }`
   */
  function handleIncomingCall(event: CustomEvent) {
    const { from_user_id, conversationId: convId } = event.detail;
    incomingCallFrom = from_user_id;
    incomingCallConvId = convId;
    showIncomingCallModal = true;
  }

  /** Ferme le modal d’appel entrant. */
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
  // 5️⃣ Acceptation / rejet d’un appel entrant
  // -----------------------------------------------------------------
  async function acceptCall() {
    if (!incomingCallConvId) return;

    try {
      const ids = $participants.map((p) => p.id);
      await startGroupCall(incomingCallConvId, ids, 'audio');
      closeIncomingCallModal();
    } catch (err) {
      console.error('Erreur acceptation appel :', err);
    }
  }

  function rejectCall() {
    closeIncomingCallModal();
  }

  // -----------------------------------------------------------------
  // 6️⃣ Contrôles de l’appel en cours
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
      <p>Préparation de l’appel…</p>
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
         CONTENU DE L’APPEL
         ----------------------------------------------------------------- -->
    <div class="call-container">
      <!-- HEADER -->
      <header class="call-header">
        <div class="header-theme">
          {#if $currentTheme === 'jardin-secret'}
            🌿 Appel Jardin Secret
          {:else if $currentTheme === 'space-hub'}
            🚀 Appel Space Hub
          {:else}
            🏠 Appel Maison Chaleureuse
          {/if}
        </div>

        <h1 class="call-title">Appel avec {conversationId}</h1>

        {#if $participants.length > 1}
          <p class="participants-count">{$participants.length} participants</p>
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
           ÉTAT DE L’APPEL (en cours / en attente)
           ----------------------------------------------------------------- -->
      {#if $callStore.isInCall}
        <!-- ------------------- CALL ACTIVE ------------------- -->
        <div class="video-grid" role="region" aria-label="Participants à l’appel">
          <!-- Local stream (self) -->
          {#if $callStore.localStream}
            <div class="video-participant local">
              <video
                bind:this={$callStore.localVideoElement}
                autoplay
                muted
                playsinline
                class="local-video"
              ></video>

              <div class="participant-info">
                <span class="participant-name">Vous</span>
                <span
                  class="icon"
                  aria-label={$callStore.isMuted ? 'Microphone coupé' : 'Microphone activé'}
                >
                  {$callStore.isMuted ? '🔇' : '🔊'}
                </span>
                <span
                  class="icon"
                  aria-label={$callStore.isVideoOff ? 'Vidéo désactivée' : 'Vidéo activée'}
                >
                  {$callStore.isVideoOff ? '📹❌' : '📹'}
                </span>
              </div>
            </div>
          {/if}

          <!-- Remote streams -->
          {#each Array.from($callStore.remoteStreams.entries()) as [userId, stream]}
            <div class="video-participant remote">
              <video srcObject={stream} autoplay playsinline class="remote-video"></video>
              <div class="participant-info">
                <span class="participant-name">
                  {$participants.find((p) => p.id === userId)?.name || userId}
                </span>
              </div>
            </div>
          {/each}

          {#if $callStore.remoteStreams.size === 0 && !$callStore.localStream}
            <div class="waiting-message">
              <p>Connexion aux participants…</p>
              <div class="spinner"></div>
            </div>
          {/if}
        </div>

        <!-- Controls -->
        <div class="call-controls" role="toolbar" aria-label="Contrôles de l’appel">
          <button
            on:click={toggleMute}
            class="control-button"
            aria-label={$callStore.isMuted ? 'Activer le son' : 'Couper le son'}
          >
            {$callStore.isMuted ? '🔇' : '🔊'}
          </button>

          <button
            on:click={toggleVideo}
            class="control-button"
            aria-label={$callStore.isVideoOff ? 'Activer la vidéo' : 'Désactiver la vidéo'}
          >
            {$callStore.isVideoOff ? '📹❌' : '📹'}
          </button>

          <button on:click={endCall} class="control-button hangup" aria-label="Raccrocher">
            📵
          </button>

          <div class="call-info">
            <span>💬 {$callStore.remoteStreams.size + 1} participants</span>
            <span class="secure-badge">✅ Connexion sécurisée P2P</span>
          </div>
        </div>

      {:else if $callStore.isCalling || $callStore.isAnswering}
        <!-- ------------------- CALL EN COURS DE SETUP ------------------- -->
        <div class="call-status" role="status" aria-live="polite">
          <span class="icon large" aria-hidden="true">✆</span>

          {#if $callStore.isCalling}
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
            {#if $currentTheme === 'jardin-secret'}
              🌸
            {:else if $currentTheme === 'space-hub'}
              🌌
            {:else}
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
                const ids = $participants.map((p) => p.id);
                await startGroupCall(conversationId, ids, 'audio');
              }}
              aria-label="Démarrer un appel audio avec les participants"
            >
              🎤 Démarrer un appel audio
            </button>

            <button
              class="start-video-call"
              on:click={async () => {
                const ids = $participants.map((p) => p.id);
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
      {#if $callStore.error}
        <div class="error-modal" role="alertdialog" aria-label="Erreur">
          <p>{$callStore.error}</p>
          <button
            on:click={() => callStore.update((s) => ({ ...s, error: null }))}
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
            <p class="caller-from">De : {incomingCallFrom}</p>
            <p class="call-info-text">Vous avez un appel entrant</p>

            <div class="call-actions">
              <button
                on:click={acceptCall}
                class="accept-btn"
                aria-label="Accepter l’appel"
              >
                ✅ Accepter
              </button>

              <button
                on:click={rejectCall}
                class="reject-btn"
                aria-label="Rejeter l’appel"
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
  /* -----------------------------------------------------------------
     PAGE LAYOUT
     ----------------------------------------------------------------- */
  .call-page {
    min-height: 100vh;
    background: var(--bg-primary, #f5f7fa);
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
    border: 4px solid #e2e8f0;
    border-top-color: #4ade80;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error-content {
    background: white;
    padding: 2.5rem;
    border-radius: 1rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    text-align: center;
    max-width: 400px;
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
    background: #4ade80;
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .call-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }