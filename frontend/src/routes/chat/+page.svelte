<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isAuthenticated, authUser } from '$lib/authStore';
  import {
    messages,
    loadMessages,
    sendMessage,
    formatTimestamp,
    chatStore,
    showGifs,
    gifResults,
    gifLoading,
    searchGifs,
  } from '$lib/chatStore';

  // -----------------------------------------------------------------
  // États locaux
  // -----------------------------------------------------------------
  let newMessage = $state('');
  let conversationId = $state('default_global'); // À rendre dynamique plus tard
  let chatContainer: HTMLElement;
  let gifSearchQuery = $state('');
  let fileInput: HTMLInputElement; // ref pour input file caché

  // -----------------------------------------------------------------
  // Fonctions utilitaires
  // -----------------------------------------------------------------
  function toggleGifs() {
    chatStore.toggleGifs();
  }

  async function handleSearchGifs() {
    if (gifSearchQuery.trim()) {
      await searchGifs(gifSearchQuery);
    }
  }

  function selectGif(gifUrl: string) {
    newMessage = `\( {newMessage}<img src=" \){gifUrl}" alt="GIF"/>`;
    toggleGifs();
  }

  async function handleSendMessage() {
    if (!newMessage.trim()) return;

    await sendMessage(newMessage, conversationId, [], new Uint8Array());
    newMessage = '';
  }

  // --------------------- UPLOAD DE FICHIERS ---------------------
  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;

    const file = input.files[0];
    const formData = new FormData();
    formData.append('file', file);
    formData.append('conversation_id', conversationId);
    formData.append('from_user_id', $authUser?.id || '');

    try {
      const response = await fetch('/api/upload/chat', {
        method: 'POST',
        body: formData,
        credentials: 'include', // si auth par cookies/sessions
      });

      if (!response.ok) throw new Error('Upload échoué');

      const data = await response.json();

      let content = '';
      if (file.type.startsWith('image/')) {
        content = `<img src="\( {data.url}" alt=" \){data.file_name}" class="uploaded-image" />`;
      } else {
        content = `<div class="file-attachment">
          <a href="\( {data.url}" download=" \){data.file_name}">📎 \( {data.file_name} ( \){formatFileSize(file.size)})</a>
        </div>`;
      }

      // Envoi d'un message dédié avec l'attachment rendu en HTML
      await sendMessage(content, conversationId, [], new Uint8Array());

      // Reset input file
      input.value = '';
    } catch (err) {
      console.error('[Upload] Erreur :', err);
      alert('Échec de l\'upload du fichier');
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // --------------------- Autres handlers ---------------------
  function handleGifKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleSearchGifs();
    }
  }

  function handleMessageKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSendMessage();
    }
  }

  function handleSubmit(event: Event) {
    event.preventDefault();
    handleSendMessage();
  }

  function isMyMessage(senderId: string): boolean {
    return $authUser?.id === senderId;
  }

  // -----------------------------------------------------------------
  // Cycle de vie
  // -----------------------------------------------------------------
  onMount(async () => {
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }

    await loadMessages(conversationId);
  });

  $effect(() => {
    if (chatContainer && messages.length > 0) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  });
</script>

<svelte:head>
  <title>Chat - Nook</title>
</svelte:head>

<div class="chat-page">
  <aside class="conversations-sidebar">
    <h2>Conversations</h2>
    <div class="conversation-list">
      <button class="conversation-item active">
        <span class="avatar">👨‍👩‍👧‍👦</span>
        <div class="conversation-info">
          <span class="name">Groupe Global</span>
          <span class="preview">Bienvenue sur Nook !</span>
        </div>
      </button>
    </div>
  </aside>

  <main class="chat-area">
    <header class="chat-header">
      <h2>👨‍👩‍👧‍👦 Groupe Global</h2>
    </header>

    <div class="messages-container" bind:this={chatContainer}>
      {#each messages as message (message.id)}
        <div class="message" class:mine={isMyMessage(message.sender_id)}>
          <div class="message-sender">{message.sender_name}</div>
          <div class="message-content">{@html message.content}</div>
          <div class="message-time">{formatTimestamp(String(message.timestamp))}</div>
        </div>
      {/each}
    </div>

    {#if $showGifs}
      <div class="gif-panel">
        <div class="gif-search">
          <input
            type="text"
            placeholder="Rechercher des GIFs..."
            bind:value={gifSearchQuery}
            on:keydown={handleGifKeydown}
            class="gif-input"
          />
          <button on:click={handleSearchGifs} class="search-btn">🔍</button>
        </div>

        {#if $gifLoading}
          <div class="gif-loading">Chargement…</div>
        {:else if $gifResults.length > 0}
          <div class="gif-results">
            {#each $gifResults as gif}
              <button class="gif-item" on:click={() => selectGif(gif.media?.[0]?.tinygif?.url ?? '')}>
                <img src={gif.media?.[0]?.tinygif?.url ?? ''} alt={gif.title} />
              </button>
            {/each}
          </div>
        {:else}
          <div class="gif-empty">Recherchez des GIFs pour les envoyer</div>
        {/if}
      </div>
    {/if}

    <form class="message-input-area" on:submit={handleSubmit}>
      <button type="button" class="attach-btn" on:click={() => fileInput.click()}>📎</button>
      <input type="file" bind:this={fileInput} on:change={handleFileUpload} style="display:none;" />

      <button type="button" class="gif-toggle" on:click={toggleGifs}>🎬</button>

      <input
        type="text"
        placeholder="Envoyer un message..."
        bind:value={newMessage}
        class="message-input"
        on:keydown={handleMessageKeydown}
      />

      <button type="submit" class="send-btn" disabled={!newMessage.trim()}>Envoyer</button>
    </form>
  </main>
</div>

<style>
   
  /* -----------------------------------------------------------------
     LAYOUT GLOBAL
     ----------------------------------------------------------------- */
  .chat-page {
    display: flex;
    height: calc(100vh - 60px);
    max-height: calc(100vh - 60px);
  }

  /* -----------------------------------------------------------------
     SIDEBAR – CONVERSATIONS
     ----------------------------------------------------------------- */
  .conversations-sidebar {
    width: 280px;
    background-color: var(--bg-secondary, #f1f5f9);
    border-right: 1px solid var(--border, #e2e8f0);
    padding: 1rem;
    overflow-y: auto;
  }

  .conversations-sidebar h2 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
    color: var(--text-primary, #1e293b);
  }

  .conversation-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .conversation-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: none;
    border: none;
    border-radius: var(--radius-lg, 0.75rem);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.2s;
    width: 100%;
  }

  .conversation-item:hover,
  .conversation-item.active {
    background-color: var(--bg-tertiary, #e2e8f0);
  }

  .avatar {
    font-size: 1.5rem;
  }

  .conversation-info {
    flex: 1;
    min-width: 0;
  }

  .conversation-info .name {
    display: block;
    font-weight: 500;
    color: var(--text-primary, #1e293b);
  }

  .conversation-info .preview {
    display: block;
    font-size: 0.85rem;
    color: var(--text-secondary, #64748b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* -----------------------------------------------------------------
     ZONE DE CHAT
     ----------------------------------------------------------------- */
  .chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary, #ffffff);
  }

  .chat-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border, #e2e8f0);
    background-color: var(--bg-primary, #ffffff);
  }

  .chat-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--text-primary, #1e293b);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .message {
    max-width: 75%;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-xl, 1rem);
    background-color: var(--chat-theirs, #f1f5f9);
    align-self: flex-start;
    animation: slide-up 0.3s ease;
  }

  .message.mine {
    background-color: var(--chat-mine, #dcfce7);
    align-self: flex-end;
  }

  .message-sender {
    font-weight: 500;
    font-size: 0.85rem;
    color: var(--accent, #4ade80);
    margin-bottom: 0.25rem;
  }

  .message-content {
    color: var(--text-primary, #1e293b);
    line-height: 1.5;
  }

  .message-time {
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
    margin-top: 0.5rem;
    text-align: right;
  }

  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* -----------------------------------------------------------------
     PANEL GIF
     ----------------------------------------------------------------- */
  .gif-panel {
    border-top: 1px solid var(--border, #e2e8f0);
    padding: 1rem;
    background-color: var(--bg-secondary, #f8fafc);
  }

  .gif-search {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .gif-input {
    flex: 1;
    padding: 0.5rem 1rem;
    border: 2px solid var(--border, #e2e8f0);
    border-radius: var(--radius-lg, 0.75rem);
    font-size: 0.9rem;
    outline: none;
  }

  .gif-input:focus {
    border-color: var(--accent, #4ade80);
  }

  .search-btn {
    padding: 0.5rem 1rem;
    background-color: var(--accent, #4ade80);
    color: white;
    border: none;
    border-radius: var(--radius-lg, 0.75rem);
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .search-btn:hover {
    background-color: var(--button-hover, #22c55e);
  }

  .gif-loading,
  .gif-empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary, #64748b);
  }

  .gif-results {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .gif-item {
    width: 100px;
    height: 100px;
    border: none;
    border-radius: var(--radius-md, 0.5rem);
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s;
    padding: 0;
  }

  .gif-item:hover {
    transform: scale(1.05);
  }

  .gif-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* -----------------------------------------------------------------
     INPUT MESSAGE
     ----------------------------------------------------------------- */
  .message-input-area {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    border-top: 1px solid var(--border, #e2e8f0);
    background-color: var(--bg-primary, #ffffff);
  }

  .gif-toggle {
    padding: 0.5rem;
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    border-radius: var(--radius-full, 50%);
    transition: background-color 0.2s;
  }

  .gif-toggle:hover {
    background-color: var(--bg-secondary, #f1f5f9);
  }

  .message-input {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 2px solid var(--border, #e2e8f0);
    border-radius: var(--radius-full, 9999px);
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .message-input:focus {
    border-color: var(--accent, #4ade80);
  }

  .send-btn {
    padding: 0.75rem 1.5rem;
    background-color: var(--accent, #4ade80);
    color: white;
    border: none;
    border-radius: var(--radius-full, 9999px);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .send-btn:hover:not(:disabled) {
    background-color: var(--button-hover, #22c55e);
    transform: translateY(-1px);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /*
/* Styles spécifiques aux uploads */
  .uploaded-image {
    max-width: 300px;
    border-radius: 8px;
    margin: 0.5rem 0;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }

  .file-attachment {
    background: rgba(0,0,0,0.05);
    padding: 0.75rem;
    border-radius: 8px;
    margin: 0.5rem 0;
    display: inline-block;
  }

  .file-attachment a {
    color: var(--accent, #4ade80);
    text-decoration: none;
    font-weight: 500;
  }

  .file-attachment a:hover {
    text-decoration: underline;
  }

  .attach-btn {
    padding: 0.5rem;
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    border-radius: 50%;
    transition: background-color 0.2s;
  }

  .attach-btn:hover {
    background-color: var(--bg-secondary, #f1f5f9);
  }
 -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 768px) {
    .chat-page {
      flex-direction: column;
    }

    .conversations-sidebar {
      width: 100%;
      max-height: 150px;
      border-right: none;
      border-bottom: 1px solid var(--border, #e2e8f0);
    }

    .conversation-list {
      flex-direction: row;
      overflow-x: auto;
    }

    .conversation-item {
      flex-shrink: 0;
      width: 200px;
    }
  }
</style>