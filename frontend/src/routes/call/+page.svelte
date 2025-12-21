<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { get } from 'svelte/store';
  import { currentTheme } from '$lib/themeStore';
  import { callManager, callStore } from '$lib/webrtc-calls';
  import { participants, loadParticipants } from '$lib/conversationStore';

  let conversationId = $page.params.id;
  let showIncomingCallModal = false;
  let incomingCallFrom = '';
  let incomingCallConvId = '';

  onMount(async () => {
    // Charger les participants de la conversation
    await loadParticipants(conversationId);
    
    // S'abonner aux appels entrants
    window.addEventListener('incoming-call', handleIncomingCall);
    
    // Si l'URL contient ?call=1, initier un appel sortant
    if ($page.url.searchParams.has('call')) {
      const type = $page.url.searchParams.get('type') || 'video';
      const participantIds = get(participants).map(p => p.id);
      await callManager.startCall(conversationId, participantIds, type as 'audio' | 'video');
    }
    
    return () => {
      window.removeEventListener('incoming-call', handleIncomingCall);
      callManager.endCall();
    };
  });

  function handleIncomingCall(event: CustomEvent) {
    const { from_user_id, conversation_id } = event.detail;
    
    // Vérifier si cet appel est pour cette conversation
    if (conversation_id === conversationId && !$callStore.isInCall) {
      incomingCallFrom = from_user_id;
      incomingCallConvId = conversation_id;
      showIncomingCallModal = true;
    }
  }

  async function acceptCall() {
    showIncomingCallModal = false;
    const participantIds = get(participants).map(p => p.id);
    await callManager.startCall(conversationId, participantIds, 'video');
  }

  function declineCall() {
    showIncomingCallModal = false;
    // Envoyer un signal de refus (à implémenter dans callManager)
    callManager.sendDeclineSignal(incomingCallFrom, conversationId);
  }

  function toggleMute() {
    callManager.toggleMute();
  }

  function toggleVideo() {
    callManager.toggleVideo();
  }

  function endCall() {
    callManager.endCall();
  }
</script>

<svelte:head>
  <title>Appel • Nook</title>
</svelte:head>

<div class="call-container theme-{$currentTheme}">
  <!-- Header avec thème -->
  <header class="call-header">
    {#if $currentTheme === 'jardin-secret'}
      <div class="theme-indicator jardin">🌿 Appel Jardin Secret</div>
    {:else if $currentTheme === 'space-hub'}
      <div class="theme-indicator space">🚀 Appel Space Hub</div>
    {:else}
      <div class="theme-indicator maison">🏠 Appel Maison Chaleureuse</div>
    {/if}
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
            />
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
            />
            <div class="remote-overlay">
              <span>{get(participants).find(p => p.id === userId)?.name || userId}</span>
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
          on:click={toggleMute}
          title={get(callStore).isMuted ? 'Activer le micro' : 'Couper le micro'}
        >
          {get(callStore).isMuted ? '🔇' : '🔊'}
        </button>
        
        <button 
          class="control-button {get(callStore).isVideoOff ? 'active' : ''}" 
          on:click={toggleVideo}
          title={get(callStore).isVideoOff ? 'Activer la caméra' : 'Désactiver la caméra'}
        >
          {get(callStore).isVideoOff ? '📹❌' : '📹'}
        </button>
        
        <button 
          class="control-button hangup" 
          on:click={endCall}
          title="Terminer l'appel"
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
        <button 
          class="start-call-button" 
          on:click={() => {
            const participantIds = get(participants).map(p => p.id);
            callManager.startCall(conversationId, participantIds, 'video');
          }}
        >
          📞 Démarrer un appel vidéo
        </button>
        <button 
          class="start-call-button audio" 
          on:click={() => {
            const participantIds = get(participants).map(p => p.id);
            callManager.startCall(conversationId, participantIds, 'audio');
          }}
        >
          🎤 Démarrer un appel audio
        </button>
      </div>
    {/if}
  </main>
  
  {#if $callStore.error}
    <div class="call-error">
      <p>{$callStore.error}</p>
      <button on:click={() => callStore.update(s => ({ ...s, error: null }))}>
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
          <button class="decline" on:click={declineCall}>❌ Refuser</button>
          <button class="accept" on:click={acceptCall}>✅ Accepter</button>
        </div>
      </div>
    </div>
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

  .call-header {
    padding: 1rem;
    text-align: center;
    border-bottom: 1px solid var(--border);
    background: var(--header-bg);
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

  .start-call-button {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.75rem 2rem;
    border-radius: 18px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    margin: 0.5rem;
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
  }
</style>