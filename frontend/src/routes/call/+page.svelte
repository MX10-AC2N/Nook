<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import {
    startGroupCall,
    endCurrentCall,
    callStore,
    callManager
  } from '$lib/webrtc-calls';
  import {
    participants,
    loadParticipants,
    activeConversationId
  } from '$lib/conversationStore';

  let conversationId = $derived($page.params?.id || '');
  let callType = $derived($page.url?.searchParams?.get('type') || 'video');
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showIncomingCallModal = $state(false);
  let incomingCallFrom = $state('');
  let incomingCallConvId = $state('');

  onMount(async () => {
    if (!$authStore.isAuthenticated) {
      goto('/login');
      return;
    }
    try {
      loading = true;
      error = null;
      await loadParticipants(conversationId);
      if ($page.url?.searchParams?.has('call')) {
        const participantIds = $participants.map(p => p.id);
        await startGroupCall(conversationId, participantIds, callType);
      }
      loading = false;
      if (browser) {
        window.addEventListener('incoming-call', handleIncomingCall);
        window.addEventListener('keydown', handleKeydown);
      }
    } catch (err) {
      error = (err instanceof Error ? err.message : String(err)) || 'Erreur de chargement de l\'appel';
      loading = false;
      console.error('Erreur appel:', err);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('incoming-call', handleIncomingCall);
      window.removeEventListener('keydown', handleKeydown);
    }
  });

  function handleIncomingCall(event: CustomEvent) {
    const { from, conversationId: convId } = event.detail;
    incomingCallFrom = from;
    incomingCallConvId = convId;
    showIncomingCallModal = true;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && showIncomingCallModal) {
      closeIncomingCallModal();
    }
  }

  function closeIncomingCallModal() {
    showIncomingCallModal = false;
    incomingCallFrom = '';
    incomingCallConvId = '';
  }

  async function acceptCall() {
    if (!incomingCallConvId) return;
    try {
      const participantIds = $participants.map(p => p.id);
      await startGroupCall(incomingCallConvId, participantIds, 'audio');
      closeIncomingCallModal();
    } catch (err) {
      console.error('Erreur acceptation appel:', err);
    }
  }

  function rejectCall() {
    closeIncomingCallModal();
  }

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
    <div class="loading-container">
      <div class="loading-spinner"></div>
      <p>Preparation de l'appel...</p>
    </div>
  {:else if error}
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
    <div class="call-container">
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
        <button onclick={() => goto('/chat')} class="back-button" aria-label="Retour au chat">
          ← Retour
        </button>
      </header>

      {#if $callStore.isInCall}
        <div class="video-grid" role="region" aria-label="Participants à l'appel">
          {#if $callStore.localStream}
            <div class="video-participant local">
              <video bind:this={$callStore.localVideoElement} autoplay muted playsinline class="local-video"></video>
              <div class="participant-info">
                <span class="participant-name">Vous</span>
                <span class="icon" aria-label={$callStore.isMuted ? 'Microphone coupé' : 'Microphone activé'}>
                  {$callStore.isMuted ? '🔇' : '🔊'}
                </span>
                <span class="icon" aria-label={$callStore.isVideoOff ? 'Vidéo désactivée' : 'Vidéo activée'}>
                  {$callStore.isVideoOff ? '📹❌' : '📹'}
                </span>
              </div>
            </div>
          {/if}
          {#each Array.from($callStore.remoteStreams.entries()) as [userId, stream]}
            <div class="video-participant remote">
              <video srcObject={stream} autoplay playsinline class="remote-video"></video>
              <div class="participant-info">
                <span class="participant-name">{$participants.find(p => p.id === userId)?.name || userId}</span>
              </div>
            </div>
          {/each}
          {#if $callStore.remoteStreams.size === 0 && !$callStore.localStream}
            <div class="waiting-message">
              <p>Connexion aux participants...</p>
              <div class="spinner"></div>
            </div>
          {/if}
        </div>

        <div class="call-controls" role="toolbar" aria-label="Contrôles de l'appel">
          <button onclick={toggleMute} class="control-button" aria-label={$callStore.isMuted ? 'Activer le son' : 'Couper le son'}>
            {$callStore.isMuted ? '🔇' : '🔊'}
          </button>
          <button onclick={toggleVideo} class="control-button" aria-label={$callStore.isVideoOff ? 'Activer la vidéo' : 'Désactiver la vidéo'}>
            {$callStore.isVideoOff ? '📹❌' : '📹'}
          </button>
          <button onclick={endCall} class="control-button hangup" aria-label="Raccrocher">
            📵
          </button>
          <div class="call-info">
            <span>💬 {$callStore.remoteStreams.size + 1} participants</span>
            <span class="secure-badge">✅ Connexion sécurisée P2P</span>
          </div>
        </div>

      {:else if $callStore.isCalling || $callStore.isAnswering}
        <div class="call-status" role="status" aria-live="polite">
          <span class="icon large" aria-hidden="true">✆</span>
          {#if $callStore.isCalling}
            <p>Appel en cours vers les participants...</p>
          {:else}
            <p>Appel entrant de {incomingCallFrom}</p>
          {/if}
          <div class="spinner"></div>
          <p class="secure-badge">Connexion sécurisée P2P</p>
        </div>

      {:else}
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
          <p class="no-call-description">Cette conversation n'a pas d'appel actif.</p>
          <div class="start-call-buttons">
            <button 
              class="start-audio-call" 
              onclick={async () => {
                const participantIds = $participants.map(p => p.id);
                await startGroupCall(conversationId, participantIds, 'audio');
              }}
              aria-label="Démarrer un appel audio avec les participants"
            >
              🎤 Démarrer un appel audio
            </button>
            <button 
              class="start-video-call" 
              onclick={async () => {
                const participantIds = $participants.map(p => p.id);
                await startGroupCall(conversationId, participantIds, 'video');
              }}
              aria-label="Démarrer un appel vidéo avec les participants"
            >
              📹 Démarrer un appel vidéo
            </button>
          </div>
        </div>
      {/if}

      {#if $callStore.error}
        <div class="error-modal" role="alertdialog" aria-label="Erreur">
          <p>{$callStore.error}</p>
          <button onclick={() => callStore.update(s => ({ ...s, error: null }))} aria-label="Fermer">
            ✕
          </button>
        </div>
      {/if}

      {#if showIncomingCallModal}
        <div 
          class="modal-overlay" 
          onclick={closeIncomingCallModal}
          role="button"
          tabindex="0"
          onkeydown={handleKeydown}
        >
          <div 
            class="incoming-call-modal" 
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-label="Appel entrant"
            tabindex="-1"
          >
            <div class="caller-avatar" aria-hidden="true">
              <span>✆</span>
            </div>
            <h2 class="caller-name">Appel entrant</h2>
            <p class="caller-from">De: {incomingCallFrom}</p>
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
    </div>
  {/if}
</div>

<style>
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
    to { transform: rotate(360deg); }
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

  .call-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .header-theme {
    font-size: 0.9rem;
    color: var(--text-secondary, #64748b);
  }

  .call-title {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0;
    color: #1e293b;
  }

  .participants-count {
    font-size: 0.9rem;
    color: #64748b;
    margin: 0;
  }

  .video-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .video-participant {
    position: relative;
    background: #1e293b;
    border-radius: 0.75rem;
    overflow: hidden;
    aspect-ratio: 16 / 9;
  }

  .video-participant.local {
    max-width: 300px;
    margin: 0 auto;
  }

  .local-video,
  .remote-video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .participant-info {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 0.75rem;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: white;
  }

  .participant-name {
    font-weight: 500;
    flex: 1;
  }

  .icon {
    font-size: 1.1rem;
  }

  .icon.large {
    font-size: 4rem;
  }

  .waiting-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 300px;
    color: white;
    gap: 1rem;
  }

  .call-controls {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: white;
    border-radius: 0.75rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    flex-wrap: wrap;
  }

  .control-button {
    width: 56px;
    height: 56px;
    border: none;
    border-radius: 50%;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
    background: #f1f5f9;
  }

  .control-button:hover {
    background: #e2e8f0;
    transform: scale(1.05);
  }

  .control-button:focus {
    outline: 2px solid #4ade80;
    outline-offset: 2px;
  }

  .control-button.hangup {
    background: #ef4444;
    color: white;
  }

  .control-button.hangup:hover {
    background: #dc2626;
  }

  .call-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #64748b;
    margin-left: 1rem;
  }

  .secure-badge {
    font-size: 0.8rem;
    color: #16a34a;
  }

  .call-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    gap: 1rem;
    color: #1e293b;
  }

  .call-status p {
    margin: 0;
    font-size: 1.1rem;
  }

  .no-call {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 500px;
    text-align: center;
    gap: 1rem;
    padding: 2rem;
  }

  .theme-icon {
    font-size: 5rem;
  }

  .no-call-title {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
    color: #1e293b;
  }

  .no-call-description {
    color: #64748b;
    margin: 0;
    max-width: 400px;
  }

  .start-call-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  .start-audio-call,
  .start-video-call {
    padding: 1rem 1.5rem;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .start-audio-call {
    background: #4ade80;
    color: white;
  }

  .start-video-call {
    background: #3b82f6;
    color: white;
  }

  .start-audio-call:hover,
  .start-video-call:hover {
    filter: brightness(1.1);
    transform: translateY(-2px);
  }

  .start-audio-call:focus,
  .start-video-call:focus {
    outline: 2px solid #4ade80;
    outline-offset: 2px;
  }

  .error-modal {
    position: fixed;
    top: 1rem;
    right: 1rem;
    background: white;
    padding: 1rem 1.5rem;
    border-radius: 0.75rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    display: flex;
    align-items: center;
    gap: 1rem;
    animation: slideIn 0.2s ease-out;
    z-index: 100;
  }

  .error-modal p {
    margin: 0;
    color: #dc2626;
  }

  .error-modal button {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: #64748b;
    padding: 0.25rem;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .incoming-call-modal {
    background: white;
    border-radius: 1.25rem;
    padding: 2rem;
    max-width: 360px;
    width: 100%;
    text-align: center;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    animation: popIn 0.3s ease-out;
  }

  @keyframes popIn {
    from {
      opacity: 0;
      transform: scale(0.9);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .caller-avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: linear-gradient(135deg, #4ade80, #22c55e);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1.25rem;
  }

  .caller-avatar span {
    font-size: 2.5rem;
  }

  .caller-name {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
    color: #1e293b;
  }

  .caller-from {
    font-size: 1rem;
    color: #64748b;
    margin: 0 0 0.5rem 0;
  }

  .call-info-text {
    font-size: 0.9rem;
    color: #94a3b8;
    margin: 0 0 1.5rem 0;
  }

  .call-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  .accept-btn,
  .reject-btn {
    flex: 1;
    padding: 0.85rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept-btn {
    background: #4ade80;
    color: white;
  }

  .accept-btn:hover {
    filter: brightness(1.1);
  }

  .reject-btn {
    background: #ef4444;
    color: white;
  }

  .reject-btn:hover {
    background: #dc2626;
  }

  .accept-btn:focus,
  .reject-btn:focus {
    outline: 2px solid #4ade80;
    outline-offset: 2px;
  }

  @media (max-width: 640px) {
    .call-controls {
      gap: 0.75rem;
    }

    .control-button {
      width: 48px;
      height: 48px;
      font-size: 1.25rem;
    }

    .start-call-buttons {
      flex-direction: column;
      width: 100%;
    }

    .start-audio-call,
    .start-video-call {
      width: 100%;
    }
  }
</style>
