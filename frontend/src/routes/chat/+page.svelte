<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { get } from 'svelte/store';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
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
  import { decryptPrivateKey, getStoredKeys } from '$lib/crypto';

  let messageInput = '';
  let gifSearch = '';
  let myPrivateKey: Uint8Array | null = null;

  onMount(async () => {
    const user = get(authStore).user;
    if (!user) return;

    currentUser.set({
      id: user.id,
      name: user.name,
      username: user.username
    });

    // Charger les clés de chiffrement
    const storedKeys = await getStoredKeys(user.id);
    if (storedKeys) {
      try {
        // Déchiffrer la clé privée avec le mot de passe utilisateur
        // (Dans une vraie implémentation, demander le mot de passe ou le récupérer du store sécurisé)
        myPrivateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, user.password);
      } catch (err) {
        connectionError.set('Erreur de déchiffrement des clés');
      }
    }

    await loadMessages();
    
    // S'abonner aux nouveaux messages via WebSocket
    const ws = new WebSocket(`ws://${window.location.host}/ws/messages`);
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'new_message') {
        messages.update(msgs => [...msgs, data.message]);
      }
    };
    
    return () => ws.close();
  });

  async function handleSendMessage() {
    if (!messageInput.trim() || !myPrivateKey) return;
    
    try {
      await sendMessage(messageInput.trim(), myPrivateKey);
      messageInput = '';
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du message');
    }
  }

  async function handleSendGif(gifUrl: string) {
    if (!myPrivateKey) return;
    
    try {
      await sendGif(gifUrl, myPrivateKey);
      showGifs.set(false);
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du GIF');
    }
  }

  async function handleSearchGifs() {
    if (gifSearch.trim()) {
      await searchGifs(gifSearch.trim());
    }
  }

  async function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (get(showGifs)) {
        await handleSearchGifs();
      } else {
        await handleSendMessage();
      }
    }
  }
</script>

<svelte:head>
  <title>Chat • Nook</title>
</svelte:head>

<div class="chat-container theme-{$currentTheme}">
  <!-- Header avec thème -->
  <header class="chat-header">
    {#if $currentTheme === 'jardin-secret'}
      <div class="theme-indicator jardin">🌿 Jardin Secret</div>
    {:else if $currentTheme === 'space-hub'}
      <div class="theme-indicator space">🚀 Space Hub</div>
    {:else}
      <div class="theme-indicator maison">🏠 Maison Chaleureuse</div>
    {/if}
    
    <div class="user-info">
      {#if $currentUser.name}
        <span>Connecté en tant que <strong>{$currentUser.name}</strong></span>
      {/if}
    </div>
  </header>

  {#if $connectionError}
    <div class="error-banner">
      {$connectionError}
    </div>
  {/if}

  <main class="messages-area">
    {#if $messages.length === 0}
      <div class="empty-chat">
        <p>Commence une conversation avec ta famille et tes amis !</p>
      </div>
    {:else}
      <div class="messages-list">
        {#each $messages as msg (msg.id)}
          <div class="message {msg.sender_id === $currentUser.id ? 'outgoing' : 'incoming'}">
            <div class="message-header">
              <span class="sender">{msg.sender_name}</span>
              <span class="timestamp">{new Date(msg.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}</span>
            </div>
            
            <div class="message-content">
              {#if msg.media_type === 'gif'}
                <img 
                  src={msg.content} 
                  alt="GIF" 
                  class="gif-preview"
                  loading="lazy"
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
                on:click={() => addReaction(msg.id, '❤️')}
                title="Réagir avec ❤️"
              >
                ❤️
              </button>
              <button 
                class="add-reaction" 
                on:click={() => addReaction(msg.id, '👍')}
                title="Réagir avec 👍"
              >
                👍
              </button>
              <button 
                class="add-reaction" 
                on:click={() => addReaction(msg.id, '😂')}
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

  {#if $showGifs}
    <div class="gif-search-area">
      <input
        type="text"
        bind:value={gifSearch}
        on:keydown={handleKeyDown}
        placeholder="Rechercher un GIF..."
        class="gif-input"
      />
      
      {#if $gifResults.length > 0}
        <div class="gif-grid">
          {#each $gifResults as gif}
            <button 
              class="gif-item" 
              on:click={() => handleSendGif(gif.media[0].gif.url)}
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
      
      <button class="close-gifs" on:click={() => showGifs.set(false)}>
        Fermer la recherche
      </button>
    </div>
  {/if}

  <footer class="message-input-area">
    <div class="input-container">
      <textarea
        bind:value={messageInput}
        on:keydown={handleKeyDown}
        placeholder="Écris un message..."
        class="message-input"
        rows="1"
        maxlength="1000"
      ></textarea>
      
      <div class="input-actions">
        <button 
          class="action-button gif-toggle" 
          on:click={() => toggleGifs()}
          title={$showGifs ? 'Fermer les GIFs' : 'Ajouter un GIF'}
        >
          {$showGifs ? '✕' : 'GIF'}
        </button>
        
        <button 
          class="action-button send" 
          on:click={handleSendMessage}
          disabled={!messageInput.trim()}
          title="Envoyer le message"
        >
          Envoyer
        </button>
      </div>
    </div>
  </footer>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }

  .chat-header {
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
    margin-bottom: 0.5rem;
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
  }

  .messages-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
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
    margin-bottom: 0.5rem;
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
  }

  .reactions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
    flex-wrap: wrap;
  }

  .reaction {
    background: rgba(0,0,0,0.05);
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.85rem;
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

  .gif-search-area {
    padding: 1rem;
    border-top: 1px solid var(--border);
    background: var(--search-bg);
  }

  .gif-input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid var(--border);
    border-radius: 12px;
    margin-bottom: 1rem;
    font-size: 1rem;
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
  }

  .message-input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 2px solid var(--border);
    border-radius: 18px;
    font-size: 1rem;
    resize: none;
    transition: border-color 0.2s;
    background: var(--input-bg);
    color: var(--text);
  }

  .message-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .input-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .action-button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .gif-toggle {
    background: var(--secondary-bg);
    color: var(--text);
  }

  .gif-toggle:hover {
    background: var(--secondary-hover);
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
    justify-content: center;
    align-items: center;
    height: 100%;
    color: var(--text-secondary);
    text-align: center;
    padding: 2rem;
  }

  .decryption-error {
    color: #f44336;
    font-style: italic;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .message {
      max-width: 90%;
    }
    
    .gif-grid {
      grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
    }
  }
</style>