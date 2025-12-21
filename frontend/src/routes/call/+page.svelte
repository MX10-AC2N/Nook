<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '@roxi/routify';
  import { goto } from '@roxi/routify';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
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
  import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';

  // États locaux
  let conversationId = $page.params.id;
  let callType = $page.url.searchParams.get('type') || 'video';
  import { $state } from 'svelte';
  let loading = $state(true);
  let error = $state(null);
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
      
      // Charger les participants de la conversation
      await loadParticipants(conversationId);
      
      // Si l'URL contient ?call=1, initier un appel sortant
      if ($page.url.searchParams.has('call')) {
        const participantIds = $participants.map(p => p.id);
        await startGroupCall(conversationId, participantIds, callType);
      }
      
      loading = false;
      
      // Écouter les appels entrants
      window.addEventListener('incoming-call', handleIncomingCall);
      
      return () => {
        window.removeEventListener('incoming-call', handleIncomingCall);
        endCurrentCall();
      };
    } catch (err) {
      error = err.message || 'Erreur de chargement de l\'appel';
      loading = false;
      console.error('Erreur appel:', err);
    }
  });

  function handleIncomingCall(event) {
    const { from_user_id, conversation_id } = event.detail;
    
    // Vérifier si cet appel est pour cette conversation
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
      error = err.message || 'Erreur lors de l\'acceptation de l\'appel';
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

  // Gestion des touches raccourci
  function handleKeydown(e) {
    if (e.key === 'm') {
      toggleMute();
    } else if (e.key === 'v') {
      toggleVideo();
    } else if (e.key === ' ') {
      endCall();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>

<svelte:head>
  <title>Appel • Nook</title>
</svelte:head>

<div class="call-container theme-{$currentTheme}">
  {#if loading}
    <div class="loading-screen">
      <LoadingSpinner size="large" />
      <p>Préparation de l'appel...</p>
    </div>
  {:else if error}
    <div class="error-screen">
      <h2>❌ {$error}</h2>
      <button onclick={() => goto('/chat')} class="back-button">
        ← Retour au chat
      </button>
    </div>
  {:else}
    <!-- Header avec thème -->
    <header class="call-header">
      {#if $currentTheme === 'jardin-secret'}
        <div class="theme-indicator jardin">🌿 Appel Jardin Secret</div>
      {:else if $currentTheme === 'space-hub'}
        <div class="theme-indicator space">🚀 Appel Space Hub</div>
      {:else}
        <div class="theme-indicator maison">🏠 Appel Maison Chaleureuse</div>
      {/if}
      
      <div class="conversation-info">
        <h1>Appel avec {conversationId}</h1>
        <p class="participant-count">{#if $participants.length > 1}{$participants.length} participants{/if}</p>
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
                class="video-element local"
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
                class="video-element remote"
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
            class="control-button {get(callStore).isMuted ? 'active' : ''}" 
            onclick={toggleMute}
            title={get(callStore).isMuted ? 'Activer le micro' : 'Couper le micro'}
          >
            {get(callStore).isMuted ? '🔇' : '🔊'}
          </button>
          
          <button 
            class="control-button {get(callStore).isVideoOff ? 'active' : ''}" 
            onclick={toggleVideo}
            title={get(callStore).isVideoOff ? 'Activer la caméra' : 'Désactiver la caméra'}
          >
            {get(callStore).isVideoOff ? '📹❌' : '📹'}
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
              {#if $callStore.isCalling}
                <span>✆ Appel en cours...</span>
              {:else}
                <span>✆ Appel entrant...</span>
              {/if}
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
          ✕ Fermer
        </button>
      </div>
    {/if}
    
    {#if showIncomingCallModal}
      <div class="modal-overlay">
        <div class="incoming-call-modal">
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
  }

  .error-screen h2 {
    color: #f44336;
    margin-bottom: 1.5rem;
    font-size: 2rem;
  }

  .back-button {
    background: #4CAF50;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 18px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    margin-top: 1rem;
  }

  .back-button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .call-header {
    padding: 1rem;
    text-align: center;
    border-bottom: 1px solid var(--border);
    background: var(--header-bg);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .theme-indicator {
    font-weight: bold;
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    display: inline-block;
  }

  .jardin { background: rgba(76, 175, 80, 0.15); color: #4CAF50; }
  .space { background: rgba(33, 150, 243, 0.15); color: #2196F3; }
  .maison { background: rgba(255, 152, 0, 0.15); color: #FF9800; }

  .conversation-info {
    flex: 1;
    text-align: center;
  }

  .conversation-info h1 {
    margin: 0;
    font-size: 1.2rem;
  }

  .participant-count {
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-top: 0.25rem;
  }

  .call-area {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 1rem;
    background: var(--call-bg);
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
    border-radius: 16px;
    overflow: hidden;
    background: #000;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  }

  .local {
    grid-column: span 1;
  }

  .remote {
    grid-column: span 1;
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
    background: rgba(0,0,0,0.5);
    color: white;
    padding: 0.5rem;
    text-align: center;
    font-weight: bold;
  }

  .local-indicators {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .indicator {
    font-size: 0.9rem;
  }

  .mic-on, .video-on { color: #4CAF50; }
  .mic-off, .video-off { color: #f44336; }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(0,0,0,0.1);
    border-radius: 50%;
    border-top-color: var(--primary);
    animation: spin 1s linear infinite;
    margin-top: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .call-controls {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--controls-bg);
    border-top: 1px solid var(--border);
  }

  .control-button {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    border: none;
    background: var(--button-bg);
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.2s;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
  }

  .control-button:hover {
    transform: scale(1.1);
  }

  .control-button.active {
    background: #f44336;
  }

  .hangup {
    background: #f44336;
    width: 70px;
    height: 70px;
  }

  .call-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: var(--text-secondary);
    margin-left: 1rem;
  }

  .connection-status {
    color: #4CAF50;
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
    padding: 2rem;
    background: var(--calling-bg);
    border-radius: 24px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.15);
  }

  .avatar-large {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: var(--avatar-bg);
    display: flex;
    justify-content: center;
    align-items: center;
    margin: 0 auto 1.5rem;
    font-size: 2.5rem;
    color: var(--avatar-text);
  }

  .calling-to {
    font-size: 1.2rem;
    margin-bottom: 1.5rem;
    color: var(--text);
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
    background: var(--primary);
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
    color: #4CAF50;
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
    font-size: 2rem;
    margin-bottom: 0.5rem;
  }

  .start-call-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
  }

  .start-call-button {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.75rem 2rem;
    border-radius: 18px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .start-call-button.audio {
    background: #9c27b0;
  }

  .start-call-button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .call-error {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: #f44336;
    color: white;
    padding: 1rem 2rem;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
    display: flex;
    align-items: center;
    gap: 1rem;
    z-index: 1000;
  }

  .call-error button {
    background: rgba(255,255,255,0.2);
    border: none;
    color: white;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    cursor: pointer;
    font-weight: bold;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0,0,0,0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .incoming-call-modal {
    background: white;
    border-radius: 20px;
    padding: 2rem;
    text-align: center;
    max-width: 400px;
    width: 90%;
    animation: modalSlideUp 0.3s ease-out;
  }

  @keyframes modalSlideUp {
    from { transform: translateY(50px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .caller-avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: #2196F3;
    margin: 0 auto 1rem;
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
    font-size: 2rem;
  }

  .incoming-call-modal h3 {
    font-size: 1.8rem;
    margin-bottom: 0.5rem;
  }

  .incoming-call-modal p {
    color: #666;
    margin-bottom: 1.5rem;
    font-size: 1.1rem;
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  .modal-actions button {
    flex: 1;
    padding: 0.75rem;
    border: none;
    border-radius: 15px;
    font-size: 1.2rem;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept {
    background: #4CAF50;
    color: white;
  }

  .decline {
    background: #f44336;
    color: white;
  }

  .modal-actions button:hover {
    opacity: 0.9;
    transform: scale(1.05);
  }

  /* Thèmes */
  .theme-jardin-secret {
    --primary: #4CAF50;
    --primary-dark: #388E3C;
    --secondary: #8BC34A;
    --border: #C8E6C9;
    --text: #333333;
    --text-secondary: #666666;
    --header-bg: #F0F7F0;
    --call-bg: #F8FDF8;
    --controls-bg: #FFFFFF;
    --button-bg: #4CAF50;
    --avatar-bg: #4CAF50;
    --avatar-text: white;
    --calling-bg: #E8F5E9;
  }

  .theme-space-hub {
    --primary: #2196F3;
    --primary-dark: #1976D2;
    --secondary: #3F51B5;
    --border: #BBDEFB;
    --text: #333333;
    --text-secondary: #666666;
    --header-bg: #E3F2FD;
    --call-bg: #F5FAFF;
    --controls-bg: #FFFFFF;
    --button-bg: #2196F3;
    --avatar-bg: #2196F3;
    --avatar-text: white;
    --calling-bg: #E3F2FD;
  }

  .theme-maison-chaleureuse {
    --primary: #FF9800;
    --primary-dark: #E65100;
    --secondary: #FF5722;
    --border: #FFE0B2;
    --text: #333333;
    --text-secondary: #666666;
    --header-bg: #FFF3E0;
    --call-bg: #FFF9F5;
    --controls-bg: #FFFFFF;
    --button-bg: #FF9800;
    --avatar-bg: #FF9800;
    --avatar-text: white;
    --calling-bg: #FFF3E0;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .video-grid {
      grid-template-columns: 1fr;
    }
    
    .call-controls {
      flex-direction: column;
      gap: 1rem;
    }
    
    .call-info {
      margin-left: 0;
      margin-top: 0.5rem;
    }
    
    .control-button {
      width: 50px;
      height: 50px;
      font-size: 1.2rem;
    }
    
    .hangup {
      width: 60px;
      height: 60px;
    }
    
    .start-call-buttons {
      flex-direction: column;
      gap: 0.75rem;
    }
    
    .call-header {
      flex-direction: column;
      gap: 0.5rem;
    }
    
    .conversation-info {
      text-align: center;
    }
    
    .back-button {
      width: 100%;
      margin-top: 0.5rem;
    }
  }
</style>