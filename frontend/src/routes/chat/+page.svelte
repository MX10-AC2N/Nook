<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { browser } from '$app/environment';
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
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let newMessage = $state('');
  let conversationId = $state('default_global'); // identifiant de la conversation (global)
  let chatContainer: HTMLElement;                // ref du conteneur de messages
  let gifSearchQuery = $state('');

  // -----------------------------------------------------------------
  // 2️⃣ Fonctions utilitaires
  // -----------------------------------------------------------------
  /** Ouvre/ferme le panneau GIF. */
  function toggleGifs() {
    chatStore.toggleGifs();
  }

  /** Recherche des GIFs via l'API Tenor. */
  async function handleSearchGifs() {
    if (gifSearchQuery.trim()) {
      await searchGifs(gifSearchQuery);
    }
  }

  /** Sélection d'un GIF → insertion dans le champ texte. */
  function selectGif(gifUrl: string) {
    // On insère un `<img>` afin que le backend le traite comme média.
    newMessage = `${newMessage}<img src="${gifUrl}" alt="GIF"/>`;
    toggleGifs(); // refermer le panneau après sélection
  }

  /** Envoi du message (texte ou GIF). */
  async function handleSendMessage() {
    if (!newMessage.trim()) return;

    // Pas de destinataires ni de clé privée dans le chat global (vide)
    await sendMessage(newMessage, conversationId, [], new Uint8Array());

    newMessage = ''; // reset du champ
  }

  /** Gestion du `Enter` dans le champ de recherche GIF. */
  function handleGifKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleSearchGifs();
    }
  }

  /** Gestion du `Enter` dans le champ de texte du chat (envoi du message). */
  function handleMessageKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSendMessage();
    }
  }

  /** Soumission du formulaire (fallback au cas où le bouton serait utilisé). */
  function handleSubmit(event: Event) {
    event.preventDefault();
    handleSendMessage();
  }

  /** Détermine si le message provient de l'utilisateur courant. */
  function isMyMessage(senderId: string): boolean {
    return $authUser?.id === senderId;
  }

  // -----------------------------------------------------------------
  // 3️⃣ Cycle de vie
  // -----------------------------------------------------------------
  onMount(async () => {
    // Rediriger si l'utilisateur n'est pas authentifié
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }

    // Charger les messages de la conversation globale
    await loadMessages(conversationId);
  });

  // Scroll automatique vers le bas à chaque nouveau message
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
  <!-- -----------------------------------------------------------------
       SIDEBAR – LISTE DES CONVERSATIONS (pour le moment fixe)
       ----------------------------------------------------------------- -->
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

  <!-- -----------------------------------------------------------------
       ZONE DE CHAT
       ----------------------------------------------------------------- -->
  <main class="chat-area">
    <header class="chat-header">
      <h2>👨‍👩‍👧‍👦 Groupe Global</h2>
    </header>

    <!-- Messages -->
    <div class="messages-container" bind:this={chatContainer}>
      {#each messages as message (message.id)}
        <div class="message" class:mine={isMyMessage(message.sender_id)}>
          <div class="message-sender">{message.sender_name}</div>
          <div class="message-content">{@html message.content}</div>
          <div class="message-time">{formatTimestamp(String(message.timestamp))}</div>
        </div>
      {/each}
    </div>

    <!-- Panneau GIF (affiché uniquement si showGifs est true) -->
    {#if $showGifs}
      <div class="gif-panel">
        <div class="gif-search">
          <input
            type="text"
            placeholder="Rechercher des GIFs..."
            bind:value={gifSearchQuery}
            onkeydown={handleGifKeydown}
            class="gif-input"
          />
          <button onclick={handleSearchGifs} class="search-btn">🔍</button>
        </div>

        {#if $gifLoading}
          <div class="gif-loading">Chargement…</div>
        {:else if $gifResults.length > 0}
          <div class="gif-results">
            {#each $gifResults as gif}
              <button class="gif-item" onclick={() => selectGif(gif.media?.[0]?.tinygif?.url)}>
                <img src={gif.media?.[0]?.tinygif?.url} alt={gif.title} />
              </button>
            {/each}
          </div>
        {:else}
          <div class="gif-empty">Recherchez des GIFs pour les envoyer</div>
        {/if}
      </div>
    {/if}

    <!-- Input du message -->
    <form class="message-input-area" onsubmit={handleSubmit}>
      <button type="button" class="gif-toggle" onclick={toggleGifs}>🎬</button>

      <input
        type="text"
        placeholder="Envoyer un message..."
        bind:value={newMessage}
        class="message-input"
        onkeydown={handleMessageKeydown}
      />

      <button type="submit" class="send-btn" disabled={!newMessage.trim()}>
        Envoyer
      </button>
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

  /* -----------------------------------------------------------------
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