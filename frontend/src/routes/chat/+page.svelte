<script context="module">
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount, onDestroy } from 'svelte';
  import { page } from '@roxi/routify';
  import { goto } from '@roxi/routify';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
  import { availableUser } from '$lib/conversationStore';
  import { 
    messages, 
    currentUser, 
    connectionError, 
    sendMessage, 
    loadMessages,
    addReaction,
    searchGifs,
    sendGif,
    gifResults,
    showGifs,
    toggleGifs,
    decryptMessageContent
  } from '$lib/chatStore';
  import { 
    conversations,
    activeConversationId,
    participants,
    loadConversations,
    loadParticipants,
    createConversation,
    sortedConversations,
    conversationDisplayName
  } from '$lib/conversationStore';
  import { 
    recordingState, 
    startRecording, 
    stopRecording, 
    sendMediaMessage,
    formatDuration
  } from '$lib/mediaStore';
  import { decryptPrivateKey, getStoredKeys } from '$lib/crypto';
  import MediaRecorder from '$lib/components/MediaRecorder.svelte';
  import MediaPlayer from '$lib/components/MediaPlayer.svelte';
  import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';

  // États locaux
  let messageInput = '';
  let gifSearch = '';
  let myPrivateKey = null;
  let showConversationSidebar = false;
  let showNewConversationModal = false;
  let newConversationName = '';
  let selectedParticipants = [];
  let showMediaRecorder = false;
  let loading = true;
  let error = null;

  onMount(async () => {
    if (!$authStore.isAuthenticated) {
      goto('/login');
      return;
    }

    try {
      loading = true;
      error = null;
      
      // Charger l'utilisateur courant
      currentUser.set({
        id: $authStore.user.id,
        name: $authStore.user.name,
        username: $authStore.user.username
      });

      // Charger les clés de chiffrement
      const storedKeys = await getStoredKeys($authStore.user.id);
      if (storedKeys) {
        const password = $authStore.user.password || prompt('Entrez votre mot de passe pour déchiffrer vos messages:');
        if (password) {
          myPrivateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
        }
      }

      // Charger les conversations
      await loadConversations();
      
      // Charger la première conversation si aucune active
      if (!$activeConversationId && $conversations.length > 0) {
        activeConversationId.set($conversations[0].id);
      }
      
      loading = false;
    } catch (err) {
      error = err.message || 'Erreur de chargement du chat';
      loading = false;
      console.error('Erreur chat:', err);
    }
  });

  // Charger les messages quand la conversation active change
  $: if ($activeConversationId && $authStore.isAuthenticated) {
    loadMessages($activeConversationId);
    loadParticipants($activeConversationId);
  }

  // Gestion des websockets pour les messages en temps réel
  let ws;
  onMount(() => {
    ws = new WebSocket(`wss://${window.location.host}/ws/messages`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'new_message' && data.message.conversation_id === $activeConversationId) {
        messages.update(msgs => [...msgs, data.message]);
      }
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      connectionError.set('Connexion WebSocket perdue');
    };
    
    ws.onclose = () => {
      console.log('WebSocket closed');
      // On peut tenter de se reconnecter après un délai
    };
    
    return () => {
      if (ws) ws.close();
    };
  });

  async function handleSendMessage() {
    if (!messageInput.trim() || !myPrivateKey || !$activeConversationId) return;
    
    try {
      const convParticipants = $participants;
      const recipientPublicKeys = convParticipants
        .filter(p => p.id !== $currentUser?.id)
        .map(p => new Uint8Array(32)); // Placeholder - à remplacer par les vraies clés
      
      await sendMessage(
        messageInput.trim(), 
        $activeConversationId, 
        recipientPublicKeys, 
        myPrivateKey
      );
      messageInput = '';
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du message');
      console.error('Erreur envoi message:', err);
    }
  }

  async function handleSendGif(gifUrl) {
    if (!myPrivateKey || !$activeConversationId) return;
    
    try {
      const convParticipants = $participants;
      const recipientPublicKeys = convParticipants
        .filter(p => p.id !== $currentUser?.id)
        .map(p => new Uint8Array(32)); // Placeholder
      
      await sendGif(
        gifUrl, 
        $activeConversationId, 
        recipientPublicKeys, 
        myPrivateKey
      );
      showGifs.set(false);
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du GIF');
      console.error('Erreur envoi GIF:', err);
    }
  }

  async function handleSearchGifs() {
    if (gifSearch.trim()) {
      await searchGifs(gifSearch.trim());
    }
  }

  function handleKeyDown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if ($showGifs) {
        handleSearchGifs();
      } else {
        handleSendMessage();
      }
    }
  }

  function toggleSidebar() {
    showConversationSidebar = !showConversationSidebar;
  }

  function openNewConversationModal() {
    showNewConversationModal = true;
    newConversationName = '';
    selectedParticipants = [];
  }

  function closeNewConversationModal() {
    showNewConversationModal = false;
  }

  async function handleCreateConversation() {
    if ((!newConversationName.trim() && selectedParticipants.length > 2) || selectedParticipants.length === 0) {
      alert('Veuillez sélectionner au moins un participant et donner un nom au groupe');
      return;
    }
    
    try {
      await createConversation(
        newConversationName.trim() || null,
        selectedParticipants,
        selectedParticipants.length > 1
      );
      closeNewConversationModal();
    } catch (err) {
      console.error('Erreur création conversation:', err);
      alert('Erreur lors de la création de la conversation');
    }
  }

  function toggleParticipantSelection(userId) {
    if (selectedParticipants.includes(userId)) {
      selectedParticipants = selectedParticipants.filter(id => id !== userId);
    } else {
      selectedParticipants = [...selectedParticipants, userId];
    }
  }

  function handleStartVideoCall() {
    if (!$activeConversationId) return;
    goto(`/call/${$activeConversationId}?call=1&type=video`);
  }

  function handleStartAudioCall() {
    if (!$activeConversationId) return;
    goto(`/call/${$activeConversationId}?call=1&type=audio`);
  }
</script>

<svelte:head>
  <title>Chat • Nook</title>
</svelte:head>

<div class="chat-container theme-{$currentTheme}">
  {#if loading}
    <div class="loading-screen">
      <LoadingSpinner size="large" />
      <p>Chargement de vos conversations...</p>
    </div>
  {:else if error}
    <div class="error-screen">
      <h2>❌ {$error}</h2>
      <button onclick={() => window.location.reload()} class="retry-button">
        🔄 Recharger
      </button>
    </div>
  {:else}
    <!-- Sidebar des conversations -->
    <div class="sidebar {showConversationSidebar ? 'open' : ''}">
      <div class="sidebar-header">
        <h2>💬 Nook</h2>
        <button class="new-conversation-button" onclick={openNewConversationModal}>
          + Nouveau
        </button>
      </div>
      
      <div class="search-conversations">
        <input 
          type="text" 
          placeholder="Rechercher une conversation..." 
          class="search-input"
        />
      </div>
      
      <div class="conversations-list">
        {#if $sortedConversations.length === 0}
          <div class="empty-conversations">
            <p>aucune conversation</p>
            <button onclick={openNewConversationModal} class="create-first-button">
              + Créer votre première conversation
            </button>
          </div>
        {:else}
          {#each $sortedConversations as conv}
            <div 
              class="conversation-item {get(activeConversationId) === conv.id ? 'active' : ''}"
              onclick={() => activeConversationId.set(conv.id)}
            >
              <div class="conversation-info">
                <div class="conversation-avatar">
                  {#if conv.is_group}
                    👥
                  {:else}
                    {#each conv.participants as p}
                      {#if p.id !== $currentUser?.id}
                        {p.name.charAt(0)}
                      {/if}
                    {/each}
                  {/if}
                </div>
                <div class="conversation-details">
                  <div class="conversation-name">
                    {conv.name || conv.participants.map(p => p.name).join(', ')}
                    {#if conv.unread_count > 0}
                      <span class="unread-badge">{conv.unread_count}</span>
                    {/if}
                  </div>
                  <div class="conversation-preview">
                    {conv.last_message_preview || 'Aucun message'}
                  </div>
                </div>
              </div>
              <div class="conversation-time">
                {new Date(conv.last_message_at * 1000).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <!-- Zone principale de chat -->
    <div class="main-chat-area">
      <!-- Header du chat -->
      <header class="chat-header">
        <button class="sidebar-toggle" onclick={toggleSidebar}>
          ☰
        </button>
        
        {#if $activeConversationId}
          <div class="conversation-header">
            <h1>{$conversationDisplayName}</h1>
            <div class="call-buttons">
              <button class="call-button audio" onclick={handleStartAudioCall}>
                🎤
              </button>
              <button class="call-button video" onclick={handleStartVideoCall}>
                📹
              </button>
            </div>
          </div>
        {:else}
          <div class="welcome-message">
            <h1>🌱 Bienvenue dans Nook</h1>
            <p>Sélectionnez une conversation ou créez-en une nouvelle pour commencer à discuter.</p>
            <button class="start-button" onclick={openNewConversationModal}>
              + Nouvelle conversation
            </button>
          </div>
        {/if}
        
        <div class="theme-selector">
          {#if $currentTheme === 'jardin-secret'}
            <div class="theme-indicator jardin">🌿 Jardin Secret</div>
          {:else if $currentTheme === 'space-hub'}
            <div class="theme-indicator space">🚀 Space Hub</div>
          {:else}
            <div class="theme-indicator maison">🏠 Maison Chaleureuse</div>
          {/if}
        </div>
      </header>

      {#if $connectionError}
        <div class="error-banner">
          {$connectionError}
        </div>
      {/if}

      {#if $activeConversationId}
        <main class="messages-area">
          {#if $messages.length === 0}
            <div class="empty-chat">
              <p>C'est le début de cette conversation ! 🌱</p>
              <p>Écrivez un message pour commencer à discuter avec votre famille et vos amis.</p>
            </div>
          {:else}
            <div class="messages-list">
              {#each $messages as msg (msg.id)}
                <div class="message {msg.sender_id === $currentUser.id ? 'outgoing' : 'incoming'}">
                  <div class="message-header">
                    <span class="sender">{msg.sender_name}</span>
                    <span class="timestamp">{new Date(msg.timestamp * 1000).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}</span>
                  </div>
                  
                  <div class="message-content">
                    {#if msg.media_type === 'gif'}
                      <img 
                        src={msg.content} 
                        alt="GIF" 
                        class="gif-preview"
                        loading="lazy"
                        onclick={() => window.open(msg.content, '_blank')}
                      />
                    {:else if msg.media_type === 'audio' || msg.media_type === 'video'}
                      <MediaPlayer 
                        message={msg} 
                        isCurrentUser={msg.sender_id === $currentUser.id}
                      />
                    {:else}
                      {#await decryptMessageContent(msg, myPrivateKey) then decrypted}
                        {decrypted}
                      {:catch error}
                        <span class="decryption-error">[Message illisible]</span>
                      {/await}
                    {/if}
                  </div>
                  
                  <div class="reactions">
                    {#if msg.reactions && Object.keys(msg.reactions).length > 0}
                      {#each Object.entries(msg.reactions) as [emoji, count]}
                        <span class="reaction">{emoji} {count}</span>
                      {/each}
                    {/if}
                    
                    <button 
                      class="add-reaction" 
                      onclick={() => addReaction(msg.id, '❤️')}
                      title="Réagir avec ❤️"
                    >
                      ❤️
                    </button>
                    <button 
                      class="add-reaction" 
                      onclick={() => addReaction(msg.id, '👍')}
                      title="Réagir avec 👍"
                    >
                      👍
                    </button>
                    <button 
                      class="add-reaction" 
                      onclick={() => addReaction(msg.id, '😂')}
                      title="Réagir avec 😂"
                    >
                      😂
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </main>
      {/if}

      {#if $activeConversationId}
        {#if $recordingState.isRecording}
          <div class="recording-banner">
            <span class="recording-indicator"></span>
            <span>Enregistrement en cours... {$recordingState.duration}s</span>
            <button class="stop-recording" onclick={() => stopRecording(true)}>
              Arrêter
            </button>
            <button class="cancel-recording" onclick={() => stopRecording(false)}>
              Annuler
            </button>
          </div>
        {/if}

        {#if $showGifs}
          <div class="gif-search-area">
            <input
              type="text"
              bind:value={gifSearch}
              onkeydown={handleKeyDown}
              placeholder="Rechercher un GIF..."
              class="gif-input"
              autofocus
            />
            
            {#if $gifResults.length > 0}
              <div class="gif-grid">
                {#each $gifResults as gif}
                  <button 
                    class="gif-item" 
                    onclick={() => handleSendGif(gif.media[0].gif.url)}
                    title={gif.title}
                  >
                    <img 
                      src={gif.media[0].tinygif.url || gif.media[0].gif.url} 
                      alt={gif.title} 
                      loading="lazy"
                    />
                  </button>
                {/each}
              </div>
            {:else if gifSearch}
              <p class="no-gifs">Aucun GIF trouvé pour "{gifSearch}"</p>
            {/if}
            
            <button class="close-gifs" onclick={() => showGifs.set(false)}>
              Fermer la recherche
            </button>
          </div>
        {/if}

        <footer class="message-input-area">
          <div class="input-container">
            <div class="input-actions-left">
              <button 
                class="action-button emoji-toggle" 
                onclick={() => toggleGifs()}
                title={$showGifs ? 'Fermer les emojis' : 'Ajouter un GIF'}
              >
                {$showGifs ? '✕' : 'GIF'}
              </button>
              
              <button 
                class="action-button media-toggle" 
                onclick={() => showMediaRecorder = !showMediaRecorder}
                title={showMediaRecorder ? 'Fermer l\'enregistreur' : 'Messages audio/vidéo'}
              >
                {showMediaRecorder ? '✕' : '🎙️'}
              </button>
            </div>
            
            {#if showMediaRecorder}
              <div class="media-recorder-container">
                <MediaRecorder disabled={!$activeConversationId} />
              </div>
            {/if}
            
            <textarea
              bind:value={messageInput}
              onkeydown={handleKeyDown}
              placeholder="Écris un message..."
              class="message-input"
              rows="1"
              maxlength="1000"
              disabled={!$activeConversationId}
            ></textarea>
            
            <div class="input-actions-right">
              <button 
                class="action-button send" 
                onclick={handleSendMessage}
                disabled={!messageInput.trim() || !$activeConversationId}
                title="Envoyer le message"
              >
                Envoyer
              </button>
            </div>
          </div>
        </footer>
      {/if}
    </div>

    <!-- Modal pour nouvelle conversation -->
    {#if showNewConversationModal}
      <div class="modal-overlay">
        <div class="new-conversation-modal">
          <h2>✨ Nouvelle conversation</h2>
          
          <div class="modal-section">
            <label for="conv-name">Nom du groupe (optionnel)</label>
            <input
              id="conv-name"
              bind:value={newConversationName}
              placeholder="Ex: Famille Dupont"
            />
          </div>
          
          <div class="modal-section">
            <h3>Participants</h3>
            <p>Sélectionnez les membres à ajouter :</p>
            
            {#each $availableUsers as user}
              <label class="participant-checkbox">
                <input
                  type="checkbox"
                  checked={selectedParticipants.includes(user.id)}
                  onclick={() => toggleParticipantSelection(user.id)}
                />
                <span>{user.name} (@{user.username})</span>
              </label>
            {/each}
          </div>
          
          <div class="modal-actions">
            <button class="cancel-button" onclick={closeNewConversationModal}>
              Annuler
            </button>
            <button class="create-button" onclick={handleCreateConversation}>
              Créer la conversation
            </button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .chat-container {
    display: flex;
    height: 100vh;
    width: 100%;
    overflow: hidden;
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

  .retry-button {
    background: #4CAF50;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 18px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .sidebar {
    width: 300px;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transition: transform 0.3s ease;
    transform: translateX(-100%);
    position: fixed;
    left: 0;
    top: 0;
    bottom: 0;
    z-index: 100;
  }

  .sidebar.open {
    transform: translateX(0);
  }

  @media (min-width: 768px) {
    .sidebar {
      transform: translateX(0);
      position: relative;
      z-index: 1;
    }
    
    .sidebar.open {
      transform: translateX(0);
    }
  }

  .sidebar-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .new-conversation-button {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .new-conversation-button:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .search-conversations {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .search-input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid var(--border);
    border-radius: 12px;
    font-size: 1rem;
    background: var(--input-bg);
    color: var(--text);
  }

  .conversations-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .empty-conversations {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary);
  }

  .empty-conversations p {
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }

  .create-first-button {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 18px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .create-first-button:hover {
    background: var(--primary-dark);
  }

  .conversation-item {
    padding: 0.75rem;
    border-radius: 12px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .conversation-item:hover {
    background: var(--hover-bg);
  }

  .conversation-item.active {
    background: var(--primary-light);
    border-left: 3px solid var(--primary);
  }

  .conversation-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
  }

  .conversation-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--avatar-bg);
    display: flex;
    justify-content: center;
    align-items: center;
    font-weight: bold;
    color: var(--avatar-text);
  }

  .conversation-details {
    flex: 1;
  }

  .conversation-name {
    font-weight: 600;
    margin-bottom: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .unread-badge {
    background: var(--primary);
    color: white;
    font-size: 0.75rem;
    padding: 0.1rem 0.5rem;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
  }

  .conversation-preview {
    font-size: 0.85rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conversation-time {
    font-size: 0.75rem;
    color: var(--text-secondary);
    min-width: 50px;
    text-align: right;
  }

  .main-chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .chat-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--header-bg);
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
  }

  .sidebar-toggle {
    display: none;
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text);
    padding: 0.5rem;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .sidebar-toggle:hover {
    background: var(--hover-bg);
  }

  @media (max-width: 767px) {
    .sidebar-toggle {
      display: block;
    }
  }

  .conversation-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    flex: 1;
  }

  .conversation-header h1 {
    margin: 0 0 0.25rem 0;
    font-size: 1.25rem;
  }

  .call-buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .call-button {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.2s;
  }

  .call-button.audio {
    background: linear-gradient(135deg, #E91E63, #C2185B);
    color: white;
  }

  .call-button.video {
    background: linear-gradient(135deg, #2196F3, #1565C0);
    color: white;
  }

  .call-button:hover {
    transform: scale(1.1);
  }

  .welcome-message {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }

  .welcome-message h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .welcome-message p {
    margin-bottom: 1.5rem;
    font-size: 1.1rem;
  }

  .start-button {
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

  .start-button:hover {
    transform: scale(1.05);
    opacity: 0.9;
  }

  .theme-selector {
    position: absolute;
    right: 1rem;
    top: 50%;
    transform: translateY(-50%);
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

  .error-banner {
    background: #ffebee;
    color: #c62828;
    padding: 0.75rem;
    text-align: center;
    font-weight: 500;
    border-bottom: 1px solid #ef9a9a;
  }

  .messages-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .messages-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .message {
    max-width: 80%;
    padding: 0.75rem 1rem;
    border-radius: 18px;
    position: relative;
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .incoming {
    background: var(--incoming-bg);
    align-self: flex-start;
    border-bottom-left-radius: 5px;
  }

  .outgoing {
    background: var(--outgoing-bg);
    align-self: flex-end;
    border-bottom-right-radius: 5px;
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.25rem;
    font-size: 0.85rem;
  }

  .sender {
    font-weight: 600;
    color: var(--sender-color);
  }

  .timestamp {
    color: var(--timestamp-color);
    font-size: 0.75rem;
  }

  .message-content {
    word-wrap: break-word;
    line-height: 1.5;
  }

  .gif-preview {
    max-width: 200px;
    max-height: 200px;
    border-radius: 12px;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .gif-preview:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
  }

  .reactions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .reaction {
    background: rgba(0,0,0,0.05);
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .add-reaction {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0.25rem;
    border-radius: 50%;
    transition: transform 0.2s;
  }

  .add-reaction:hover {
    transform: scale(1.2);
    background: rgba(0,0,0,0.05);
  }

  .recording-banner {
    padding: 0.75rem;
    background: rgba(255, 87, 34, 0.1);
    border-top: 1px solid #ffab91;
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-content: center;
  }

  .recording-indicator {
    width: 12px;
    height: 12px;
    background: #ff5722;
    border-radius: 50%;
    animation: blink 1s infinite;
  }

  @keyframes blink {
    50% { opacity: 0.5; }
  }

  .stop-recording, .cancel-recording {
    padding: 0.25rem 0.75rem;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .stop-recording {
    background: #4CAF50;
    color: white;
    border: none;
  }

  .stop-recording:hover {
    background: #43a047;
  }

  .cancel-recording {
    background: #f44336;
    color: white;
    border: none;
  }

  .cancel-recording:hover {
    background: #d32f2f;
  }

  .gif-search-area {
    padding: 1rem;
    border-top: 1px solid var(--border);
    background: var(--search-bg);
    border-bottom: 1px solid var(--border);
  }

  .gif-input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid var(--border);
    border-radius: 12px;
    margin-bottom: 1rem;
    font-size: 1rem;
    background: var(--input-bg);
    color: var(--text);
  }

  .gif-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: 0.75rem;
    max-height: 300px;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .gif-item {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    border-radius: 8px;
    overflow: hidden;
    aspect-ratio: 1;
    transition: transform 0.2s;
  }

  .gif-item:hover {
    transform: scale(1.1);
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
  }

  .gif-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .no-gifs {
    text-align: center;
    color: var(--text-secondary);
    padding: 1rem 0;
  }

  .close-gifs {
    display: block;
    width: 100%;
    padding: 0.5rem;
    background: var(--button-bg);
    color: white;
    border: none;
    border-radius: 8px;
    margin-top: 0.5rem;
    cursor: pointer;
  }

  .message-input-area {
    padding: 1rem;
    border-top: 1px solid var(--border);
    background: var(--footer-bg);
  }

  .input-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    position: relative;
  }

  .input-actions-left {
    display: flex;
    gap: 0.5rem;
    margin-bottom: -2.5rem;
    z-index: 10;
    padding-left: 0.5rem;
  }

  .media-recorder-container {
    padding: 0.5rem;
    background: var(--media-recorder-bg);
    border-radius: 16px;
    border: 1px solid var(--border);
    margin-bottom: 0.5rem;
  }

  .message-input {
    width: 100%;
    padding: 0.75rem 1rem 0.75rem 4rem;
    border: 2px solid var(--border);
    border-radius: 18px;
    font-size: 1rem;
    resize: none;
    transition: border-color 0.2s;
    background: var(--input-bg);
    color: var(--text);
    min-height: 44px;
    max-height: 200px;
    overflow-y: auto;
  }

  .message-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .input-actions-right {
    position: absolute;
    right: 1rem;
    bottom: 1.25rem;
  }

  .action-button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .emoji-toggle {
    background: var(--secondary-bg);
    color: var(--text);
  }

  .emoji-toggle:hover {
    background: var(--secondary-hover);
  }

  .media-toggle {
    background: linear-gradient(135deg, #9C27B0, #6A1B9A);
    color: white;
  }

  .media-toggle:hover {
    opacity: 0.9;
  }

  .send {
    background: var(--primary);
    color: white;
  }

  .send:hover:not(:disabled) {
    background: var(--primary-dark);
  }

  .send:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .empty-chat {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: var(--text-secondary);
    text-align: center;
    padding: 2rem;
    gap: 1rem;
  }

  .empty-chat p {
    font-size: 1.1rem;
    margin: 0;
  }

  .decryption-error {
    color: #f44336;
    font-style: italic;
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

  .new-conversation-modal {
    background: white;
    border-radius: 20px;
    padding: 2rem;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
    animation: modalSlideUp 0.3s ease-out;
  }

  @keyframes modalSlideUp {
    from { transform: translateY(50px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .modal-section {
    margin-bottom: 1.5rem;
  }

  .modal-section label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  .modal-section input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid var(--border);
    border-radius: 12px;
    font-size: 1rem;
  }

  .modal-section h3 {
    margin-bottom: 0.5rem;
    font-size: 1.2rem;
  }

  .participant-checkbox {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .participant-checkbox:hover {
    background: var(--hover-bg);
  }

  .participant-checkbox input {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .cancel-button, .create-button {
    flex: 1;
    padding: 0.75rem;
    border-radius: 15px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .cancel-button {
    background: #e0e0e0;
    color: #333;
  }

  .create-button {
    background: var(--primary);
    color: white;
  }

  .cancel-button:hover, .create-button:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  /* Thèmes */
  .theme-jardin-secret {
    --primary: #4CAF50;
    --primary-dark: #388E3C;
    --primary-light: #E8F5E9;
    --secondary: #8BC34A;
    --secondary-bg: #F1F8E9;
    --secondary-hover: #DCEDC8;
    --sidebar-bg: #F8FDF8;
    --header-bg: #F0F7F0;
    --footer-bg: #F8FDF8;
    --search-bg: #FFFFFF;
    --media-recorder-bg: #E8F5E9;
    --incoming-bg: #E3F2FD;
    --outgoing-bg: #E8F5E9;
    --hover-bg: #F5F5F5;
    --border: #C8E6C9;
    --text: #333333;
    --text-secondary: #666666;
    --sender-color: #2E7D32;
    --timestamp-color: #78909C;
    --button-bg: #4CAF50;
    --input-bg: #FFFFFF;
    --avatar-bg: #4CAF50;
    --avatar-text: white;
    --call-bg: #F8FDF8;
    --controls-bg: #FFFFFF;
    --media-bg: #F1F8E9;
  }

  .theme-space-hub {
    --primary: #2196F3;
    --primary-dark: #1976D2;
    --primary-light: #E3F2FD;
    --secondary: #3F51B5;
    --secondary-bg: #E8EAF6;
    --secondary-hover: #C5CAE9;
    --sidebar-bg: #F5FAFF;
    --header-bg: #E3F2FD;
    --footer-bg: #F5FAFF;
    --search-bg: #FFFFFF;
    --media-recorder-bg: #E3F2FD;
    --incoming-bg: #E8EAF6;
    --outgoing-bg: #E3F2FD;
    --hover-bg: #F5F5F5;
    --border: #BBDEFB;
    --text: #333333;
    --text-secondary: #666666;
    --sender-color: #1565C0;
    --timestamp-color: #78909C;
    --button-bg: #2196F3;
    --input-bg: #FFFFFF;
    --avatar-bg: #2196F3;
    --avatar-text: white;
    --call-bg: #F5FAFF;
    --controls-bg: #FFFFFF;
    --media-bg: #E8EAF6;
  }

  .theme-maison-chaleureuse {
    --primary: #FF9800;
    --primary-dark: #E65100;
    --primary-light: #FFF3E0;
    --secondary: #FF5722;
    --secondary-bg: #FBE9E7;
    --secondary-hover: #FFCCBC;
    --sidebar-bg: #FFF9F5;
    --header-bg: #FFF3E0;
    --footer-bg: #FFF9F5;
    --search-bg: #FFFFFF;
    --media-recorder-bg: #FFF3E0;
    --incoming-bg: #FBE9E7;
    --outgoing-bg: #FFF3E0;
    --hover-bg: #F5F5F5;
    --border: #FFE0B2;
    --text: #333333;
    --text-secondary: #666666;
    --sender-color: #E65100;
    --timestamp-color: #78909C;
    --button-bg: #FF9800;
    --input-bg: #FFFFFF;
    --avatar-bg: #FF9800;
    --avatar-text: white;
    --call-bg: #FFF9F5;
    --controls-bg: #FFFFFF;
    --media-bg: #FBE9E7;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .chat-container {
      flex-direction: column;
    }
    
    .sidebar {
      width: 100%;
      max-height: 70vh;
      overflow-y: auto;
    }
    
    .conversations-list {
      max-height: 50vh;
      overflow-y: auto;
    }
    
    .main-chat-area {
      height: auto;
    }
    
    .message {
      max-width: 90%;
    }
    
    .gif-grid {
      grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
    }
    
    .input-container {
      flex-direction: row;
      flex-wrap: wrap;
      gap: 0.5rem;
    }
    
    .input-actions-left, .input-actions-right {
      position: static;
      margin-bottom: 0;
      width: auto;
    }
    
    .message-input {
      padding: 0.75rem;
      flex: 1;
      min-height: 44px;
      max-height: 100px;
    }
    
    .media-recorder-container {
      width: 100%;
    }
    
    .new-conversation-modal {
      width: 95%;
      max-width: 95%;
    }
  }
</style>