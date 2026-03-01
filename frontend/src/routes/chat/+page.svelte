<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chatStore,
    loadMessages,
    sendMessage,
    sendGif,
    searchGifs,
    toggleGifs,
    formatTimestamp,
  } from '$lib/chatStore.svelte.ts';

  // -----------------------------------------------------------------
  // États locaux
  // -----------------------------------------------------------------
  let newMessage     = $state('');
  let conversationId = $state('default_global');
  let chatContainer  = $state<HTMLElement | undefined>(undefined);
  let gifSearchQuery = $state('');
  let fileInput      = $state<HTMLInputElement | undefined>(undefined);
  let sending        = $state(false);

  // Polling 5s — en attendant le WS chat
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // -----------------------------------------------------------------
  // Handlers
  // -----------------------------------------------------------------
  async function handleSendMessage() {
    if (!newMessage.trim() || sending) return;
    sending = true;
    const content = newMessage;
    newMessage = '';
    await sendMessage(content, conversationId);
    sending = false;
  }

  function handleMessageKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSendMessage(); }
  }

  function handleSubmit(e: Event) { e.preventDefault(); handleSendMessage(); }

  async function handleSearchGifs() {
    if (gifSearchQuery.trim()) await searchGifs(gifSearchQuery);
  }

  function handleGifKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); handleSearchGifs(); }
  }

  function handleSelectGif(url: string) {
    sendGif(url, conversationId);
    toggleGifs();
  }

  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    const file = input.files[0];
    const fd = new FormData();
    fd.append('file', file);
    fd.append('conversation_id', conversationId);
    fd.append('from_user_id', authStore.user?.id || '');
    try {
      const res = await fetch('/api/upload/chat', { method: 'POST', body: fd, credentials: 'include' });
      if (!res.ok) throw new Error('Upload échoué');
      const data = await res.json();
      const content = file.type.startsWith('image/')
        ? `<img src="${data.url}" alt="${data.file_name}" class="uploaded-image" />`
        : `<span class="file-attachment">📎 <a href="${data.url}" download="${data.file_name}">${data.file_name}</a></span>`;
      await sendMessage(content, conversationId);
      input.value = '';
    } catch (err) {
      console.error('[Upload]', err);
      alert("Échec de l'upload");
    }
  }

  function isMyMessage(senderId: string) { return authStore.user?.id === senderId; }

  // -----------------------------------------------------------------
  // Cycle de vie
  // -----------------------------------------------------------------
  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await loadMessages(conversationId);
    pollTimer = setInterval(() => loadMessages(conversationId), 5000);
  });

  onDestroy(() => { if (pollTimer) clearInterval(pollTimer); });

  // Auto-scroll quand de nouveaux messages arrivent
  $effect(() => {
    const _ = chatStore.messages.length; // dépendance réactive
    if (chatContainer) {
      Promise.resolve().then(() => {
        if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
      });
    }
  });
</script>

<svelte:head><title>Chat — Nook</title></svelte:head>

<div class="chat-page">

  <!-- ─── SIDEBAR ─── -->
  <aside class="conversations-sidebar">
    <h2>Conversations</h2>
    <div class="conversation-list">
      <button class="conversation-item active">
        <span class="avatar">👨‍👩‍👧‍👦</span>
        <div class="conversation-info">
          <span class="name">Groupe Global</span>
          <span class="preview">Canal familial</span>
        </div>
      </button>
    </div>
  </aside>

  <!-- ─── ZONE CHAT ─── -->
  <main class="chat-area">

    <header class="chat-header">
      <h2>👨‍👩‍👧‍👦 Groupe Global</h2>
      {#if chatStore.connectionError}
        <span class="conn-error">⚠️ {chatStore.connectionError}</span>
      {/if}
    </header>

    <!-- Messages — lit chatStore.messages directement ($state réactif) -->
    <div class="messages-container" bind:this={chatContainer}>
      {#if chatStore.messages.length === 0}
        <div class="empty-state">Aucun message — soyez le premier à écrire 👋</div>
      {:else}
        {#each chatStore.messages as msg (msg.id)}
          <div class="message" class:mine={isMyMessage(msg.sender_id)}>
            {#if !isMyMessage(msg.sender_id)}
              <div class="message-sender">{msg.sender_name || msg.sender_id}</div>
            {/if}
            <div class="message-content">{@html msg.content}</div>
            <div class="message-time">{formatTimestamp(msg.timestamp)}</div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- GIF panel -->
    {#if chatStore.showGifs}
      <div class="gif-panel">
        <div class="gif-search">
          <input type="text" placeholder="Rechercher des GIFs…"
            bind:value={gifSearchQuery} onkeydown={handleGifKeydown} class="gif-input" />
          <button onclick={handleSearchGifs} class="search-btn">🔍</button>
          <button onclick={toggleGifs} class="close-btn">✕</button>
        </div>
        {#if chatStore.gifLoading}
          <div class="gif-status">Chargement…</div>
        {:else if chatStore.gifResults.length > 0}
          <div class="gif-grid">
            {#each chatStore.gifResults as gif}
              <button class="gif-item"
                onclick={() => handleSelectGif(gif.media?.[0]?.tinygif?.url ?? '')}>
                <img src={gif.media?.[0]?.tinygif?.url ?? ''} alt={gif.title} loading="lazy" />
              </button>
            {/each}
          </div>
        {:else}
          <div class="gif-status">Tapez un mot pour chercher des GIFs</div>
        {/if}
      </div>
    {/if}

    <!-- Zone saisie -->
    <form class="input-area" onsubmit={handleSubmit}>
      <button type="button" class="icon-btn" onclick={() => fileInput?.click()} title="Joindre un fichier">📎</button>
      <input type="file" bind:this={fileInput} onchange={handleFileUpload} style="display:none" />

      <button type="button" class="icon-btn" onclick={toggleGifs} title="GIF">🎬</button>

      <input
        type="text"
        class="message-input"
        placeholder="Envoyer un message..."
        bind:value={newMessage}
        onkeydown={handleMessageKeydown}
        disabled={sending}
      />

      <button type="submit" class="send-btn" disabled={!newMessage.trim() || sending}>
        {sending ? '…' : 'Envoyer'}
      </button>
    </form>

  </main>
</div>

<style>
  .chat-page {
    display: flex;
    height: calc(100vh - 60px);
    overflow: hidden;
  }

  /* ── Sidebar ── */
  .conversations-sidebar {
    width: 260px;
    flex-shrink: 0;
    background: var(--bg-secondary, #f1f5f9);
    border-right: 1px solid var(--border, #e2e8f0);
    padding: 1rem;
    overflow-y: auto;
  }
  .conversations-sidebar h2 {
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .06em;
    color: var(--text-secondary, #64748b);
    margin-bottom: .75rem;
  }
  .conversation-list { display: flex; flex-direction: column; gap: .25rem; }
  .conversation-item {
    display: flex; align-items: center; gap: .75rem;
    padding: .6rem .75rem;
    background: none; border: none; border-radius: .5rem;
    cursor: pointer; text-align: left; width: 100%;
    transition: background .15s;
  }
  .conversation-item:hover, .conversation-item.active {
    background: var(--bg-tertiary, #e2e8f0);
  }
  .avatar { font-size: 1.4rem; flex-shrink: 0; }
  .conversation-info { flex: 1; min-width: 0; }
  .conversation-info .name {
    display: block; font-weight: 600; font-size: .9rem;
    color: var(--text-primary, #1e293b);
  }
  .conversation-info .preview {
    display: block; font-size: .78rem; color: var(--text-secondary, #64748b);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  /* ── Zone chat ── */
  .chat-area {
    flex: 1; min-width: 0;
    display: flex; flex-direction: column;
    background: var(--bg-primary, #fff);
  }
  .chat-header {
    padding: .75rem 1rem; flex-shrink: 0;
    border-bottom: 1px solid var(--border, #e2e8f0);
    display: flex; align-items: center; gap: 1rem;
  }
  .chat-header h2 { margin: 0; font-size: 1.05rem; color: var(--text-primary, #1e293b); }
  .conn-error { font-size: .78rem; color: #dc2626; margin-left: auto; }

  /* ── Messages ── */
  .messages-container {
    flex: 1; overflow-y: auto;
    padding: 1rem; display: flex; flex-direction: column; gap: .5rem;
  }
  .empty-state {
    flex: 1; display: flex; align-items: center; justify-content: center;
    color: var(--text-secondary, #94a3b8); font-size: .9rem; text-align: center;
  }
  .message {
    max-width: 72%; padding: .55rem .9rem;
    border-radius: 1rem;
    background: var(--chat-theirs, #f1f5f9);
    align-self: flex-start;
    word-break: break-word;
    animation: pop .18s ease;
  }
  .message.mine {
    background: var(--chat-mine, #dcfce7);
    align-self: flex-end;
  }
  .message-sender {
    font-size: .75rem; font-weight: 700;
    color: var(--accent, #4ade80); margin-bottom: .15rem;
  }
  .message-content {
    font-size: .9rem; color: var(--text-primary, #1e293b); line-height: 1.5;
  }
  .message-content :global(img.uploaded-image),
  .message-content :global(img.chat-gif) {
    max-width: 260px; border-radius: 8px; margin-top: .3rem; display: block;
  }
  .message-time {
    font-size: .68rem; color: var(--text-secondary, #94a3b8);
    margin-top: .25rem; text-align: right;
  }
  @keyframes pop { from { opacity:0; transform:translateY(5px); } to { opacity:1; transform:none; } }

  /* ── GIF panel ── */
  .gif-panel {
    flex-shrink: 0; border-top: 1px solid var(--border, #e2e8f0);
    padding: .75rem 1rem; background: var(--bg-secondary, #f8fafc);
    max-height: 220px; overflow-y: auto;
  }
  .gif-search { display: flex; gap: .4rem; margin-bottom: .6rem; }
  .gif-input {
    flex: 1; padding: .4rem .7rem;
    border: 1.5px solid var(--border, #e2e8f0); border-radius: .45rem;
    font-size: .88rem; outline: none;
  }
  .gif-input:focus { border-color: var(--accent, #4ade80); }
  .search-btn, .close-btn {
    padding: .4rem .7rem; border: none; border-radius: .45rem;
    cursor: pointer; font-size: .88rem;
  }
  .search-btn { background: var(--accent, #4ade80); color: #fff; }
  .close-btn  { background: var(--bg-tertiary, #e2e8f0); color: var(--text-secondary, #64748b); }
  .gif-status { text-align: center; padding: .75rem; color: var(--text-secondary, #94a3b8); font-size: .83rem; }
  .gif-grid { display: flex; flex-wrap: wrap; gap: .35rem; }
  .gif-item {
    width: 85px; height: 85px;
    border: none; border-radius: .4rem; overflow: hidden;
    cursor: pointer; padding: 0; transition: transform .15s;
  }
  .gif-item:hover { transform: scale(1.05); }
  .gif-item img { width: 100%; height: 100%; object-fit: cover; }

  /* ── Zone saisie ── */
  .input-area {
    flex-shrink: 0;
    display: flex; align-items: center; gap: .4rem;
    padding: .7rem 1rem;
    border-top: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
  }
  .icon-btn {
    padding: .45rem; background: none; border: none;
    font-size: 1.15rem; cursor: pointer; border-radius: 50%;
    transition: background .15s; flex-shrink: 0;
  }
  .icon-btn:hover { background: var(--bg-secondary, #f1f5f9); }
  .message-input {
    flex: 1; min-width: 0;
    padding: .6rem 1rem;
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: 9999px; font-size: .9rem; outline: none;
    transition: border-color .15s;
  }
  .message-input:focus { border-color: var(--accent, #4ade80); }
  .message-input:disabled { opacity: .6; }
  .send-btn {
    flex-shrink: 0;
    padding: .6rem 1.2rem;
    background: var(--accent, #4ade80); color: #fff;
    border: none; border-radius: 9999px;
    font-weight: 700; font-size: .88rem; cursor: pointer;
    transition: all .15s; white-space: nowrap;
  }
  .send-btn:hover:not(:disabled) { background: var(--button-hover, #22c55e); transform: translateY(-1px); }
  .send-btn:disabled { opacity: .45; cursor: not-allowed; }

  /* ── Responsive mobile ── */
  @media (max-width: 640px) {
    .chat-page { flex-direction: column; }
    .conversations-sidebar {
      width: 100%; max-height: 90px; padding: .4rem;
      border-right: none; border-bottom: 1px solid var(--border, #e2e8f0);
    }
    .conversation-list { flex-direction: row; overflow-x: auto; }
    .conversation-item { flex-shrink: 0; padding: .35rem .5rem; }
    .conversation-info .preview { display: none; }
    .message { max-width: 88%; }
  }
</style>
