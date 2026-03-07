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

  // ─────────────────────────────────────────────────────────────────
  // Types locaux
  // ─────────────────────────────────────────────────────────────────
  interface Conv {
    id: string;
    name: string | null;
    is_group: boolean;
    updated_at: number;
    created_by: string;
  }
  interface AvailUser {
    id: string;
    username: string;
    name: string | null;
  }
  interface Participant {
    id: string;
    username: string;
    name: string | null;
    role: string;
  }

  // ─────────────────────────────────────────────────────────────────
  // État principal
  // ─────────────────────────────────────────────────────────────────
  let conversations   = $state<Conv[]>([]);
  let activeConvId    = $state('default_global');
  let activeConvName  = $state('👨‍👩‍👧‍👦 Groupe Global');
  // Conv complète active — pour savoir si DM (is_group=false) → bouton appel
  let activeConv      = $state<Conv | null>(null);
  let availableUsers  = $state<AvailUser[]>([]);
  let loadingConvs    = $state(true);

  // Cache participants par conv : Map<convId, Participant[]>
  // Évite les requêtes répétées pour afficher les noms dans la sidebar
  let participantsCache = $state<Record<string, Participant[]>>({});

  // Envoi message
  let newMessage     = $state('');
  let chatContainer  = $state<HTMLElement | undefined>(undefined);
  let gifSearchQuery = $state('');
  let fileInput      = $state<HTMLInputElement | undefined>(undefined);
  let sending        = $state(false);

  // Modal nouvelle conversation
  let showNewConv    = $state(false);
  let newConvName    = $state('');
  let selectedUsers  = $state<string[]>([]);
  let creatingConv   = $state(false);
  let convError      = $state<string | null>(null);

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // ─────────────────────────────────────────────────────────────────
  // Chargement
  // ─────────────────────────────────────────────────────────────────

  async function loadConversations() {
    loadingConvs = true;
    try {
      const res = await fetch('/api/conversations', { credentials: 'include' });
      if (!res.ok) return;
      const data = await res.json();
      // Backend retourne Vec<Conversation> direct (tableau JSON)
      const list: Conv[] = Array.isArray(data) ? data : (data.conversations ?? data ?? []);
      list.sort((a, b) => (b.updated_at ?? 0) - (a.updated_at ?? 0));
      conversations = list;

      // Pré-charger les noms des participants pour les DM
      await Promise.all(
        list.filter(c => !c.is_group).map(c => loadParticipantsForConv(c.id))
      );
    } catch (err) {
      console.error('[Chat] loadConversations:', err);
    } finally {
      loadingConvs = false;
    }
  }

  async function loadParticipantsForConv(convId: string): Promise<Participant[]> {
    // Utiliser le cache si disponible
    if (participantsCache[convId]) return participantsCache[convId];
    try {
      const res = await fetch(`/api/conversations/${convId}/participants`, { credentials: 'include' });
      if (!res.ok) return [];
      const data = await res.json();
      const parts: Participant[] = data.participants ?? [];
      // Mise à jour du cache sans réassigner l'objet entier (règle Svelte 5)
      participantsCache[convId] = parts;
      return parts;
    } catch {
      return [];
    }
  }

  async function loadAvailableUsers() {
    try {
      const res = await fetch('/api/users/available', { credentials: 'include' });
      if (!res.ok) return;
      const data = await res.json();
      availableUsers = data.users ?? [];
    } catch { /* silencieux */ }
  }

  // ─────────────────────────────────────────────────────────────────
  // Affichage sidebar
  // ─────────────────────────────────────────────────────────────────

  /** Nom affiché dans la sidebar pour une conv */
  function convDisplayName(conv: Conv): string {
    if (conv.id === 'default_global') return 'Groupe Global';
    if (conv.is_group) return conv.name ?? 'Groupe sans nom';
    // DM : nom de l'autre participant depuis le cache
    const parts = participantsCache[conv.id] ?? [];
    const other = parts.find(p => p.id !== authStore.user?.id);
    return other ? (other.name ?? other.username) : (conv.name ?? 'Message direct');
  }

  function convAvatar(conv: Conv): string {
    if (conv.id === 'default_global') return '👨‍👩‍👧‍👦';
    return conv.is_group ? '👥' : '💬';
  }

  // ─────────────────────────────────────────────────────────────────
  // Sélection conversation active
  // ─────────────────────────────────────────────────────────────────

  async function selectConversation(conv: Conv) {
    activeConvId = conv.id;
    activeConv   = conv;

    if (conv.id === 'default_global') {
      activeConvName = '👨‍👩‍👧‍👦 Groupe Global';
    } else if (conv.is_group) {
      activeConvName = conv.name ?? 'Groupe sans nom';
    } else {
      // DM : charger les participants si pas en cache
      const parts = await loadParticipantsForConv(conv.id);
      const other = parts.find(p => p.id !== authStore.user?.id);
      activeConvName = other
        ? `💬 ${other.name ?? other.username}`
        : (conv.name ?? '💬 Message direct');
    }

    // Recharger les messages et relancer le polling sur la nouvelle conv
    await loadMessages(conv.id);
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(() => loadMessages(conv.id), 5000);
  }

  // ─────────────────────────────────────────────────────────────────
  // Nouvelle conversation (modal)
  // ─────────────────────────────────────────────────────────────────

  async function openNewConv() {
    showNewConv = true;
    newConvName = '';
    selectedUsers = [];
    convError = null;
    await loadAvailableUsers();
  }

  function toggleUserSelect(userId: string) {
    if (selectedUsers.includes(userId)) {
      selectedUsers = selectedUsers.filter(id => id !== userId);
    } else {
      selectedUsers = [...selectedUsers, userId];
    }
  }

  /** Label du bouton de création selon la sélection */
  function createBtnLabel(): string {
    if (creatingConv) return 'Création…';
    if (selectedUsers.length === 0) return 'Sélectionner des membres';
    if (selectedUsers.length === 1) return '💬 Ouvrir la conversation';
    return `👥 Créer le groupe (${selectedUsers.length} membres)`;
  }

  async function createConversation() {
    if (selectedUsers.length === 0 || creatingConv) return;
    creatingConv = true;
    convError = null;

    try {
      const isGroup = selectedUsers.length > 1;
      const name = isGroup ? (newConvName.trim() || null) : null;

      const res = await fetch('/api/conversations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          name,
          is_group: isGroup,
          participant_ids: selectedUsers,
        }),
      });

      if (!res.ok) {
        const d = await res.json().catch(() => ({}));
        throw new Error(d.message ?? `Erreur HTTP ${res.status}`);
      }

      // Backend retourne Conversation direct (pas enveloppé)
      const newConv: Conv = await res.json();
      showNewConv = false;

      // Recharger la liste et sélectionner la nouvelle conv
      await loadConversations();
      await selectConversation(newConv);

    } catch (err) {
      convError = err instanceof Error ? err.message : 'Erreur inconnue';
      console.error('[Chat] createConversation:', err);
    } finally {
      creatingConv = false;
    }
  }

  // ─────────────────────────────────────────────────────────────────
  // Envoi messages
  // ─────────────────────────────────────────────────────────────────

  async function handleSendMessage() {
    if (!newMessage.trim() || sending) return;
    sending = true;
    const content = newMessage;
    newMessage = '';
    await sendMessage(content, activeConvId, [], new Uint8Array());
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
    sendGif(url, activeConvId, [], new Uint8Array());
    toggleGifs();
  }

  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    const file = input.files[0];
    const fd = new FormData();
    fd.append('file', file);
    fd.append('conversation_id', activeConvId);
    fd.append('from_user_id', authStore.user?.id || '');
    try {
      const res = await fetch('/api/upload/chat', { method: 'POST', body: fd, credentials: 'include' });
      if (!res.ok) throw new Error('Upload échoué');
      const data = await res.json();
      const content = file.type.startsWith('image/')
        ? `<img src="${data.url}" alt="${data.file_name}" class="uploaded-image" />`
        : `<span class="file-attachment">📎 <a href="${data.url}" download="${data.file_name}">${data.file_name}</a></span>`;
      await sendMessage(content, activeConvId, [], new Uint8Array());
      input.value = '';
    } catch (err) {
      console.error('[Upload]', err);
      alert("Échec de l'upload");
    }
  }

  function isMyMessage(senderId: string) { return authStore.user?.id === senderId; }

  // ─────────────────────────────────────────────────────────────────
  // Cycle de vie
  // ─────────────────────────────────────────────────────────────────

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await loadConversations();
    await loadMessages(activeConvId);
    pollTimer = setInterval(() => loadMessages(activeConvId), 5000);
  });

  onDestroy(() => { if (pollTimer) clearInterval(pollTimer); });

  $effect(() => {
    const _ = chatStore.messages.length;
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
    <div class="sidebar-header">
      <h2>Conversations</h2>
      <button class="btn-new-conv" onclick={openNewConv} title="Nouvelle conversation">＋</button>
    </div>

    <div class="conversation-list">
      {#if loadingConvs}
        <div class="sidebar-loading">
          <span class="loading-dots">···</span>
        </div>
      {:else}
        <!-- Groupe global toujours en premier -->
        {#if conversations.length === 0 || !conversations.find(c => c.id === 'default_global')}
          <button
            class="conversation-item"
            class:active={activeConvId === 'default_global'}
            onclick={() => selectConversation({ id: 'default_global', name: 'Groupe Global', is_group: true, updated_at: 0, created_by: '' })}
          >
            <span class="avatar">👨‍👩‍👧‍👦</span>
            <div class="conversation-info">
              <span class="name">Groupe Global</span>
              <span class="preview">Canal familial</span>
            </div>
          </button>
        {/if}

        {#each conversations as conv (conv.id)}
          <button
            class="conversation-item"
            class:active={conv.id === activeConvId}
            onclick={() => selectConversation(conv)}
          >
            <span class="avatar">{convAvatar(conv)}</span>
            <div class="conversation-info">
              <span class="name">{convDisplayName(conv)}</span>
              <span class="preview">{conv.is_group ? 'Groupe' : 'Message privé'}</span>
            </div>
          </button>
        {/each}

        {#if conversations.length === 0}
          <p class="sidebar-empty">Appuyez sur ＋ pour commencer une conversation</p>
        {/if}
      {/if}
    </div>
  </aside>

  <!-- ─── ZONE CHAT ─── -->
  <main class="chat-area">

    <header class="chat-header">
      <h2>{activeConvName}</h2>
      {#if chatStore.connectionError}
        <span class="conn-error">⚠️ {chatStore.connectionError}</span>
      {/if}
      {#if activeConv && !activeConv.is_group && activeConv.id !== 'default_global'}
        <div class="call-actions">
          <a
            href="/call/{activeConv.id}?type=audio"
            class="call-btn call-btn--audio"
            title="Appel audio"
            aria-label="Démarrer un appel audio"
          >🎤</a>
          <a
            href="/call/{activeConv.id}?type=video"
            class="call-btn call-btn--video"
            title="Appel vidéo"
            aria-label="Démarrer un appel vidéo"
          >📹</a>
        </div>
      {/if}
    </header>

    <div class="messages-container" bind:this={chatContainer}>
      {#if chatStore.messages.length === 0}
        <div class="empty-state">
          <span class="empty-icon">💬</span>
          <p>Aucun message — soyez le premier à écrire !</p>
        </div>
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

    <form class="input-area" onsubmit={handleSubmit}>
      <button type="button" class="icon-btn" onclick={() => fileInput?.click()} title="Joindre">📎</button>
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

<!-- ─── MODAL NOUVELLE CONVERSATION ─── -->
{#if showNewConv}
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Nouvelle conversation"
    onclick={(e) => { if ((e.target as HTMLElement).classList.contains('modal-overlay')) showNewConv = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') showNewConv = false; }}
  >
    <div class="modal">
      <div class="modal-header">
        <h3>Nouvelle conversation</h3>
        <button class="modal-close" onclick={() => showNewConv = false} aria-label="Fermer">✕</button>
      </div>

      <div class="modal-body">
        <!-- Nom du groupe (seulement si plusieurs membres) -->
        {#if selectedUsers.length > 1}
          <label class="form-label">
            Nom du groupe
            <input
              type="text"
              class="form-input"
              bind:value={newConvName}
              placeholder="Famille, Projet, Amis…"
              maxlength="60"
            />
          </label>
        {/if}

        <p class="form-label-title">
          {#if selectedUsers.length === 0}
            Choisissez un membre pour démarrer
          {:else if selectedUsers.length === 1}
            1 personne sélectionnée — conversation privée
          {:else}
            {selectedUsers.length} personnes — groupe
          {/if}
        </p>

        <div class="user-list">
          {#if availableUsers.length === 0}
            <div class="user-list-empty">
              Aucun autre membre disponible.<br/>
              <small>Les membres doivent être approuvés par l'admin.</small>
            </div>
          {:else}
            {#each availableUsers as u (u.id)}
              <button
                class="user-item"
                class:selected={selectedUsers.includes(u.id)}
                onclick={() => toggleUserSelect(u.id)}
              >
                <span class="user-avatar">
                  {selectedUsers.includes(u.id) ? '✓' : '👤'}
                </span>
                <div class="user-item-info">
                  <span class="user-name">{u.name ?? u.username}</span>
                  {#if u.name}
                    <span class="user-handle">@{u.username}</span>
                  {/if}
                </div>
              </button>
            {/each}
          {/if}
        </div>

        {#if convError}
          <p class="form-error">⚠️ {convError}</p>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn-cancel" onclick={() => showNewConv = false}>Annuler</button>
        <button
          class="btn-create-conv"
          disabled={selectedUsers.length === 0 || creatingConv}
          onclick={createConversation}
        >
          {createBtnLabel()}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .chat-page {
    display: flex;
    height: calc(100vh - 60px);
    overflow: hidden;
  }

  /* ─── Sidebar ─── */
  .conversations-sidebar {
    width: 260px;
    flex-shrink: 0;
    background: var(--bg-secondary, #f1f5f9);
    border-right: 1px solid var(--border, #e2e8f0);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: .85rem 1rem .5rem;
    flex-shrink: 0;
  }
  .sidebar-header h2 {
    font-size: .78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .06em;
    color: var(--text-secondary, #64748b);
    margin: 0;
  }
  .btn-new-conv {
    width: 26px; height: 26px;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent, #4ade80); color: #fff;
    border: none; border-radius: 50%;
    font-size: 1.1rem; line-height: 1;
    cursor: pointer; transition: background .15s;
    flex-shrink: 0;
  }
  .btn-new-conv:hover { background: var(--button-hover, #22c55e); }

  .conversation-list {
    flex: 1;
    overflow-y: auto;
    padding: .25rem .5rem .75rem;
    display: flex;
    flex-direction: column;
    gap: .2rem;
  }
  .sidebar-loading {
    padding: 1.5rem;
    text-align: center;
    color: var(--text-secondary, #94a3b8);
    font-size: 1.1rem;
    letter-spacing: .2em;
  }
  .sidebar-empty {
    padding: .75rem;
    font-size: .78rem;
    color: var(--text-secondary, #94a3b8);
    text-align: center;
    line-height: 1.5;
  }

  .conversation-item {
    display: flex;
    align-items: center;
    gap: .65rem;
    padding: .6rem .7rem;
    background: none;
    border: none;
    border-radius: .6rem;
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background .12s;
  }
  .conversation-item:hover { background: var(--bg-tertiary, #e2e8f0); }
  .conversation-item.active { background: var(--bg-tertiary, #e2e8f0); }

  .avatar { font-size: 1.3rem; flex-shrink: 0; }
  .conversation-info { flex: 1; min-width: 0; }
  .conversation-info .name {
    display: block;
    font-weight: 600;
    font-size: .88rem;
    color: var(--text-primary, #1e293b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .conversation-info .preview {
    display: block;
    font-size: .74rem;
    color: var(--text-secondary, #64748b);
  }

  /* ─── Zone chat ─── */
  .chat-area {
    flex: 1; min-width: 0;
    display: flex; flex-direction: column;
    background: var(--bg-primary, #fff);
  }
  .chat-header {
    padding: .75rem 1rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border, #e2e8f0);
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  .chat-header h2 { margin: 0; font-size: 1.05rem; color: var(--text-primary, #1e293b); flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .conn-error { font-size: .78rem; color: #dc2626; }
  .call-actions { display: flex; gap: .35rem; flex-shrink: 0; margin-left: auto; }
  .call-btn {
    display: flex; align-items: center; justify-content: center;
    width: 32px; height: 32px; border-radius: 50%;
    background: var(--bg-secondary, #f1f5f9);
    font-size: .95rem; text-decoration: none;
    transition: background .15s, transform .15s;
  }
  .call-btn:hover { background: var(--accent, #4ade80); transform: scale(1.1); }

  /* ─── Messages ─── */
  .messages-container {
    flex: 1; overflow-y: auto;
    padding: 1rem;
    display: flex; flex-direction: column; gap: .5rem;
  }
  .empty-state {
    flex: 1;
    display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: .5rem;
    color: var(--text-secondary, #94a3b8);
    text-align: center;
  }
  .empty-icon { font-size: 2rem; }
  .empty-state p { margin: 0; font-size: .9rem; }

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
  @keyframes pop { from { opacity:0; transform:translateY(4px); } to { opacity:1; transform:none; } }

  /* ─── GIF ─── */
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
    background: var(--bg-primary, #fff); color: var(--text-primary, #1e293b);
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

  /* ─── Saisie ─── */
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
    background: var(--bg-primary, #fff); color: var(--text-primary, #1e293b);
  }
  .message-input:focus { border-color: var(--accent, #4ade80); }
  .message-input:disabled { opacity: .6; }
  .send-btn {
    flex-shrink: 0;
    padding: .6rem 1.2rem;
    background: var(--accent, #4ade80); color: #fff;
    border: none; border-radius: 9999px;
    font-weight: 700; font-size: .88rem; cursor: pointer;
    transition: all .15s;
  }
  .send-btn:hover:not(:disabled) { background: var(--button-hover, #22c55e); transform: translateY(-1px); }
  .send-btn:disabled { opacity: .45; cursor: not-allowed; }

  /* ─── Modal ─── */
  .modal-overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg-primary, #fff);
    border-radius: 1rem;
    width: 100%; max-width: 420px;
    max-height: 85vh;
    display: flex; flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,.25);
    overflow: hidden;
  }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border, #e2e8f0);
    flex-shrink: 0;
  }
  .modal-header h3 { margin: 0; font-size: 1rem; color: var(--text-primary, #1e293b); }
  .modal-close {
    background: none; border: none; font-size: 1.1rem;
    cursor: pointer; color: var(--text-secondary, #64748b);
    padding: .2rem .4rem;
  }
  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 1rem 1.25rem;
    display: flex; flex-direction: column; gap: .75rem;
  }
  .form-label {
    display: block;
    font-size: .82rem; font-weight: 600;
    color: var(--text-secondary, #64748b);
  }
  .form-label-title {
    font-size: .82rem; font-weight: 600;
    color: var(--text-secondary, #64748b);
    margin: 0;
  }
  .form-input {
    display: block; width: 100%;
    margin-top: .3rem;
    padding: .55rem .85rem;
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: .5rem; font-size: .9rem; outline: none;
    background: var(--bg-primary, #fff); color: var(--text-primary, #1e293b);
    box-sizing: border-box;
    transition: border-color .15s;
  }
  .form-input:focus { border-color: var(--accent, #4ade80); }
  .form-error {
    color: #dc2626; font-size: .83rem; margin: 0;
  }

  .user-list {
    display: flex; flex-direction: column; gap: .3rem;
  }
  .user-list-empty {
    padding: 1rem; text-align: center;
    color: var(--text-secondary, #94a3b8); font-size: .85rem;
    line-height: 1.6;
  }
  .user-item {
    display: flex; align-items: center; gap: .7rem;
    padding: .6rem .85rem;
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: .6rem;
    background: none; cursor: pointer;
    transition: all .12s; text-align: left;
  }
  .user-item:hover {
    border-color: var(--accent, #4ade80);
    background: var(--bg-secondary, #f8fafc);
  }
  .user-item.selected {
    border-color: var(--accent, #4ade80);
    background: #f0fdf4;
  }
  .user-avatar {
    font-size: 1.15rem;
    width: 1.5rem; text-align: center;
    flex-shrink: 0;
    color: var(--accent, #4ade80);
    font-weight: 700;
  }
  .user-item-info { flex: 1; min-width: 0; }
  .user-name {
    display: block;
    font-size: .9rem; font-weight: 600;
    color: var(--text-primary, #1e293b);
  }
  .user-handle {
    display: block;
    font-size: .76rem;
    color: var(--text-secondary, #94a3b8);
  }

  .modal-footer {
    display: flex; gap: .6rem; justify-content: flex-end;
    padding: .85rem 1.25rem;
    border-top: 1px solid var(--border, #e2e8f0);
    flex-shrink: 0;
  }
  .btn-cancel {
    padding: .55rem 1rem;
    background: var(--bg-secondary, #f1f5f9);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .5rem; font-size: .88rem; cursor: pointer;
    color: var(--text-secondary, #64748b);
    transition: background .12s;
  }
  .btn-cancel:hover { background: var(--bg-tertiary, #e2e8f0); }
  .btn-create-conv {
    padding: .55rem 1.2rem;
    background: var(--accent, #4ade80); color: #fff;
    border: none; border-radius: .5rem;
    font-size: .88rem; font-weight: 700; cursor: pointer;
    transition: background .12s;
  }
  .btn-create-conv:hover:not(:disabled) { background: var(--button-hover, #22c55e); }
  .btn-create-conv:disabled { opacity: .5; cursor: not-allowed; }

  /* ─── Mobile ─── */
  @media (max-width: 640px) {
    .chat-page { flex-direction: column; }
    .conversations-sidebar {
      width: 100%; max-height: 90px;
      border-right: none; border-bottom: 1px solid var(--border, #e2e8f0);
    }
    .conversation-list {
      flex-direction: row;
      overflow-x: auto;
      padding: .25rem .5rem;
    }
    .conversation-item { flex-shrink: 0; max-width: 140px; }
    .conversation-info .preview { display: none; }
    .message { max-width: 88%; }
    .modal { max-width: 96vw; margin: .75rem; }
  }
</style>
