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

  // États locaux avec Svelte 5 runes
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
      endCurrentCall();
    }
  });

  function handleIncomingCall(event: CustomEvent) {
    const { from_user_id, conversation_id } = event.detail;
    
    if (conversation_id === conversationId && !$callStore.isInCall) {
      incomingCallFrom = from_user_id;
      incomingCallConvId = conversation_id;
      showIncomingCallModal = true;
    }
  }

  async function acceptCall() {
    try {
      showIncomingCallModal = false;
      const participantIds = $participants.map(p => p.id);
      await startGroupCall(conversationId, participantIds, 'video');
    } catch (err) {
      error = (err instanceof Error ? err.message : String(err)) || 'Erreur lors de l\'acceptation de l\'appel';
      console.error('Erreur acceptation appel:', err);
    }
  }

  function declineCall() {
    showIncomingCallModal = false;
    callManager.sendDeclineSignal(incomingCallFrom, conversationId);
  }

  function toggleMute() {
    callManager.toggleMute();
  }

  function toggleVideo() {
    callManager.toggleVideo();
  }

  function endCall() {
    endCurrentCall();
    goto('/chat');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'm') {
      toggleMute();
    } else if (e.key === 'v') {
      toggleVideo();
    } else if (e.key === ' ') {
      e.preventDefault();
      endCall();
    }
  }
</script>

<svelte:head>
  <title>Appel - Nook</title>
</svelte:head>

<div class="call-container theme-{$currentTheme}">
  {#if loading}
    <div class="loading-screen">
      <div class="spinner-large"></div>
      <p>Preparation de l'appel...</p>
    </div>
  {:else if error}
    <div class="error-screen">
      <h2>❌ {error}</h2>
      <button onclick={() => goto('/chat')} class="back-button">
        ← Retour au chat
      </button>
    </div>
  {:else}
    <header class="call-header">
      <div class="theme-indicator {$currentTheme}">
        {#if $currentTheme === 'jardin-secret'}
          🌿 Appel Jardin Secret
        {:else if $currentTheme === 'space-hub'}
          🚀 Appel Space Hub
        {:else}
          🏠 Appel Maison Chaleureuse
        {/if}
      </div>

      <div class="conversation-info">
        <h1>Appel avec {conversationId}</h1>
        {#if $participants.length > 1}
          <p class="participant-count">{$participants.length} participants</p>
        {/if}
      </div>

      <button class="back-button" onclick={() => goto('/chat')}>
        ← Retour
      </button>
    </header>

    <main class="call-area">
      {#if $callStore.isInCall}
        <div class="video-grid">
          {#if $callStore.localStream}
            <div class="video-container local">
              <video 
                autoplay 
                playsinline 
                muted 
                srcObject={$callStore.localStream} 
                class="video-element"
              ></video>
              <div class="local-overlay">
                <span>Vous</span>
                <div class="local-indicators">
                  {#if !$callStore.isMuted}
                    <span class="indicator mic-on">🎤</span>
                  {:else}
                    <span class="indicator mic-off">🔇</span>
                  {/if}
                  {#if !$callStore.isVideoOff}
                    <span class="indicator video-on">📹</span>
                  {:else}
                    <span class="indicator video-off">📹❌</span>
                  {/if}
                </div>
              </div>
            </div>
          {/if}

          {#each Array.from($callStore.remoteStreams.entries()) as [userId, stream]}
            <div class="video-container remote">
              <video 
                autoplay 
                playsinline 
                srcObject={stream} 
                class="video-element"
              ></video>
              <div class="remote-overlay">
                <span>{$participants.find(p => p.id === userId)?.name || userId}</span>
              </div>
            </div>
          {/each}

          {#if $callStore.remoteStreams.size === 0 && !$callStore.localStream}
            <div class="placeholder">
              <p>Connexion aux participants...</p>
              <div class="spinner"></div>
            </div>
          {/if}
        </div>

        <div class="call-controls">
          <button 
            class="control-button {$callStore.isMuted ? 'active' : ''}" 
            onclick={toggleMute}
            title={$callStore.isMuted ? 'Activer le micro' : 'Couper le micro'}
          >
            {$callStore.isMuted ? '🔇' : '🔊'}
          </button>

          <button 
            class="control-button {$callStore.isVideoOff ? 'active' : ''}" 
            onclick={toggleVideo}
            title={$callStore.isVideoOff ? 'Activer la caméra' : 'Désactiver la caméra'}
          >
            {$callStore.isVideoOff ? '📹❌' : '📹'}
          </button>

          <button 
            class="control-button hangup" 
            onclick={endCall}
            title="Terminer l'appel (Espace)"
          >
            📵
          </button>

          <div class="call-info">
            <span>💬 {$callStore.remoteStreams.size + 1} participants</span>
            <span class="connection-status">✅ Connexion sécurisée P2P</span>
          </div>
        </div>
      {:else if $callStore.isCalling || $callStore.isAnswering}
        <div class="calling-screen">
          <div class="calling-content">
            <div class="avatar-large">
              <span class="calling-icon">✆</span>
            </div>
            <p class="calling-to">
              {#if $callStore.isCalling}
                Appel en cours vers les participants...
              {:else}
                Appel entrant de {incomingCallFrom}
              {/if}
            </p>
            <div class="calling-indicators">
              <div class="pulse"></div>
              <div class="pulse"></div>
              <div class="pulse"></div>
            </div>
            <div class="connection-info">
              Connexion sécurisée P2P
            </div>
          </div>
        </div>
      {:else}
        <div class="no-call-screen">
          <div class="theme-icon">
            {#if $currentTheme === 'jardin-secret'}
              🌸
            {:else if $currentTheme === 'space-hub'}
              🌌
            {:else}
              🏡
            {/if}
          </div>
          <h2>Aucun appel en cours</h2>
          <p>Cette conversation n'a pas d'appel actif.</p>
          <div class="start-call-buttons">
            <button 
              class="start-call-button audio" 
              onclick={() => {
                const participantIds = $participants.map(p => p.id);
                startGroupCall(conversationId, participantIds, 'audio');
              }}
            >
              🎤 Démarrer un appel audio
            </button>
            <button 
              class="start-call-button video" 
              onclick={() => {
                const participantIds = $participants.map(p => p.id);
                startGroupCall(conversationId, participantIds, 'video');
              }}
            >
              📹 Démarrer un appel vidéo
            </button>
          </div>
        </div>
      {/if}
    </main>

    {#if $callStore.error}
      <div class="call-error">
        <p>{$callStore.error}</p>
        <button onclick={() => callStore.update(s => ({ ...s, error: null }))}>
          ✕
        </button>
      </div>
    {/if}

    {#if showIncomingCallModal}
      <div class="modal-overlay" onclick={() => showIncomingCallModal = false} role="button" tabindex="0" onkeydown={(e) => e.key === 'Escape' && (showIncomingCallModal = false)}>
        <div class="incoming-call-modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Appel entrant">
          <div class="caller-avatar">
            <span>✆</span>
          </div>
          <h3>Appel entrant</h3>
          <p>de {incomingCallFrom}</p>
          <p>dans la conversation {conversationId}</p>
          <div class="modal-actions">
            <button class="decline" onclick={declineCall}>❌ Refuser</button>
            <button class="accept" onclick={acceptCall}>✅ Accepter</button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .call-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }

  .loading-screen, .error-screen {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100vh;
    width: 100%;
    text-align: center;
    padding: 2rem;
    gap: 1rem;
  }

  .spinner-large {
    width: 60px;
    height: 60px;
    border: 4px solid var(--border, #e2e8f0);
    border-top-color: var(--accent, #4ade80);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-screen h2 {
    color: var(--error, #ef4444);
    margin-bottom: 1.5rem;
    font-size: 2rem;
  }

  .back-button {
    background: var(--accent, #4CAF50);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: var(--radius-lg, 0.75rem);
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .call-header {
    padding: 1rem;
    text-align: center;
    border-bottom: 1px solid var(--border, #e2e8f0);
    background: var(--header-bg, #f8fafc);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .theme-indicator {
    font-weight: 600;
    padding: 0.5rem 1rem;
    border-radius: var(--radius-full, 9999px);
    font-size: 0.9rem;
  }

  .theme-indicator.jardin-secret {
    background: rgba(74, 222, 128, 0.15);
    color: #4ade80;
  }

  .theme-indicator.space-hub {
    background: rgba(33, 150, 243, 0.15);
    color: #2196F3;
  }

  .theme-indicator.maison-chaleureuse {
    background: rgba(255, 152, 0, 0.15);
    color: #f97316;
  }

  .conversation-info {
    flex: 1;
    text-align: center;
    min-width: 200px;
  }

  .conversation-info h1 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary, #1e293b);
  }

  .participant-count {
    color: var(--text-secondary, #64748b);
    font-size: 0.85rem;
    margin-top: 0.25rem;
  }

  .call-area {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 1rem;
    background: var(--call-bg, #f8fafc);
  }

  .video-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
    width: 100%;
    height: 100%;
  }

  .video-container {
    position: relative;
    border-radius: var(--radius-xl, 1rem);
    overflow: hidden;
    background: #1a1a1a;
    box-shadow: var(--shadow-lg, 0 10px 15px -3px rgba(0, 0, 0, 0.1));
  }

  .video-element {
    width: 100%;
    height: 100%;
    display: block;
    object-fit: cover;
  }

  .local-overlay, .remote-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    padding: 0.75rem;
    text-align: center;
    font-weight: 500;
  }

  .local-indicators {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .indicator {
    font-size: 1rem;
  }

  .mic-on, .video-on { color: #4ade80; }
  .mic-off, .video-off { color: #ef4444; }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--text-secondary, #64748b);
    gap: 0.5rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border, #e2e8f0);
    border-top-color: var(--accent, #4ade80);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .call-controls {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--controls-bg, #ffffff);
    border-top: 1px solid var(--border, #e2e8f0);
    flex-wrap: wrap;
  }

  .control-button {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: none;
    background: var(--button-bg, #4ade80);
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.2s;
    box-shadow: var(--shadow-md);
  }

  .control-button:hover {
    transform: scale(1.1);
  }

  .control-button.active {
    background: #ef4444;
  }

  .hangup {
    background: #ef4444;
    width: 64px;
    height: 64px;
  }

  .call-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: var(--text-secondary, #64748b);
    margin-left: 1rem;
    font-size: 0.85rem;
  }

  .connection-status {
    color: #4ade80;
    font-weight: 500;
  }

  .calling-screen {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
  }

  .calling-content {
    text-align: center;
    padding: 3rem;
    background: var(--calling-bg, #f0fdf4);
    border-radius: var(--radius-2xl, 1.5rem);
    box-shadow: var(--shadow-xl);
    max-width: 400px;
  }

  .avatar-large {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--accent, #4ade80), var(--accent-dark, #22c55e));
    margin: 0 auto 1.5rem;
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
  }

  .calling-icon {
    font-size: 3rem;
  }

  .calling-to {
    font-size: 1.1rem;
    color: var(--text-primary, #1e293b);
    margin-bottom: 1.5rem;
  }

  .calling-indicators {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .pulse {
    width: 12px;
    height: 12px;
    background: var(--accent, #4ade80);
    border-radius: 50%;
    animation: pulse 1.5s infinite;
  }

  .pulse:nth-child(2) { animation-delay: 0.2s; }
  .pulse:nth-child(3) { animation-delay: 0.4s; }

  @keyframes pulse {
    0% { transform: scale(0.8); opacity: 0.5; }
    50% { transform: scale(1); opacity: 1; }
    100% { transform: scale(0.8); opacity: 0.5; }
  }

  .connection-info {
    color: #4ade80;
    font-weight: 500;
    margin-top: 0.5rem;
  }

  .no-call-screen {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    padding: 2rem;
    width: 100%;
    height: 100%;
  }

  .theme-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .no-call-screen h2 {
    font-size: 1.75rem;
    color: var(--text-primary, #1e293b);
    margin-bottom: 0.5rem;
  }

  .no-call-screen p {
    color: var(--text-secondary, #64748b);
  }

  .start-call-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  .start-call-button {
    background: var(--accent, #4ade80);
    color: white;
    border: none;
    padding: 0.875rem 2rem;
    border-radius: var(--radius-lg, 0.75rem);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: var(--shadow-md);
  }

  .start-call-button.audio {
    background: linear-gradient(135deg, #9c27b0, #7b1fa2);
  }

  .start-call-button:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg);
  }

  .call-error {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    background: var(--error, #ef4444);
    color: white;
    padding: 1rem 2rem;
    border-radius: var(--radius-lg, 0.75rem);
    box-shadow: var(--shadow-xl);
    display: flex;
    align-items: center;
    gap: 1rem;
    z-index: 1000;
    animation: slide-up 0.3s ease;
  }

  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }

  .call-error button {
    background: rgba(255, 255, 255, 0.2);
    border: none;
    color: white;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    animation: fade-in 0.2s ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .incoming-call-modal {
    background: white;
    border-radius: var(--radius-2xl, 1.5rem);
    padding: 2rem;
    text-align: center;
    max-width: 400px;
    width: 90%;
    animation: modal-slide-up 0.3s ease;
  }

  @keyframes modal-slide-up {
    from {
      transform: translateY(30px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .caller-avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: linear-gradient(135deg, #2196F3, #1976D2);
    margin: 0 auto 1rem;
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
    font-size: 2rem;
  }

  .incoming-call-modal h3 {
    font-size: 1.5rem;
    color: var(--text-primary, #1e293b);
    margin-bottom: 0.5rem;
  }

  .incoming-call-modal p {
    color: var(--text-secondary, #64748b);
    margin-bottom: 0.5rem;
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-top: 1.5rem;
  }

  .modal-actions button {
    flex: 1;
    padding: 0.75rem;
    border: none;
    border-radius: var(--radius-lg, 0.75rem);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept {
    background: #4ade80;
    color: white;
  }

  .decline {
    background: #ef4444;
    color: white;
  }

  .modal-actions button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .video-grid {
      grid-template-columns: 1fr;
    }

    .call-controls {
      gap: 1rem;
      padding: 1rem;
    }

    .call-info {
      margin-left: 0;
      margin-top: 0.5rem;
      width: 100%;
    }

    .control-button {
      width: 48px;
      height: 48px;
      font-size: 1.2rem;
    }

    .hangup {
      width: 56px;
      height: 56px;
    }

    .start-call-buttons {
      flex-direction: column;
    }

    .call-header {
      flex-direction: column;
      gap: 0.75rem;
    }

    .conversation-info {
      order: -1;
    }

    .back-button {
      width: 100%;
    }
  }
</style>
