<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chatStore,
    loadMessages,
    loadMoreMessages,
    sendMessage,
    editMessage,
    deleteMessage,
    sendEmoji,
    toggleEmojiPicker,
    formatTimestamp,
    setActiveConv,
    disconnectWs,
    requestNotificationPermission,
  
    countReactions,
    toggleReaction,
    loadReactions,
    reactions,
  } from '$lib/chatStore.svelte.ts';
  import { sanitizeHtml } from '$lib/sanitize';
  import {
    recordingState,
    startRecording,
    stopRecording,
    cancelRecording,
    formatDuration,
  } from '$lib/mediaStore.svelte.js';

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
  let activeConvName  = $state('🌿 Nook');
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
  let fileInput      = $state<HTMLInputElement | undefined>(undefined);
  let sending        = $state(false);

  // ─── Messages vocaux ──────────────────────────────────────────────
  // Durée max : 2 min audio, 30s vidéo
  const MAX_AUDIO_SEC = 120;
  const MAX_VIDEO_SEC = 30;

  // Modal nouvelle conversation
  let showNewConv    = $state(false);
  let newConvName    = $state('');
  let selectedUsers  = $state<string[]>([]);
  let creatingConv   = $state(false);
  let convError      = $state<string | null>(null);

  // Renommage inline du groupe actif
  let renamingConv   = $state(false);
  let renameValue    = $state('');

  function startRename() {
    renameValue = activeConv?.name ?? activeConvName.replace(/^[^ ]+ /, '');
    renamingConv = true;
  }

  async function submitRename() {
    if (!activeConv || !renameValue.trim()) { renamingConv = false; return; }
    try {
      const res = await fetch(`/api/conversations/${activeConv.id}/rename`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name: renameValue.trim() }),
      });
      if (!res.ok) {
        const d = await res.json().catch(() => ({}));
        console.error('[Rename]', d.error);
        return;
      }
      const data = await res.json();
      activeConvName = `👥 ${data.name}`;
      // Mettre à jour le cache local sans recharger toute la liste
      conversations = conversations.map(c =>
        c.id === activeConv!.id ? { ...c, name: data.name } : c
      );
      activeConv = { ...activeConv, name: data.name };
    } catch (e) {
      console.error('[Rename]', e);
    } finally {
      renamingConv = false;
    }
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); submitRename(); }
    if (e.key === 'Escape') { renamingConv = false; }
  }

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // État édition de message
  let editingMsgId   = $state<string | null>(null);
  let editingContent = $state('');
  // Menu contextuel (hover)
  let hoveredMsgId   = $state<string | null>(null);

  // ─────────────────────────────────────────────────────────────────
  // Réactions aux messages
  // ─────────────────────────────────────────────────────────────────
  
  // reactions : Map<msgId, { counts: Record<emoji, string[]>, myEmoji: string|null }>
  let reactions = $state<Record<string, { counts: Record<string, string[]>; myEmoji: string | null }>>({});
  // picker étendu ouvert pour quel message
  let emojiPickerMsgId = $state<string | null>(null);
  let _hoverTimer: ReturnType<typeof setTimeout> | null = null;
  let emojiCat    = $state('😊');   // catégorie active dans le picker emoji
  let pickerTab   = $state<'emoji'|'gif'>('emoji'); // onglet actif emoji vs GIF
  let localGifs   = $state<{id:string;category:string;cat_label:string;file:string;title:string}[]>([]);
  let gifCat      = $state('');      // catégorie GIF active
  let gifCats     = $state<string[]>([]);
  let gifsLoaded  = $state(false);
  let gifsError   = $state(false);

  async function loadLocalGifs() {
    if (gifsLoaded) return;
    try {
      const res = await fetch('/gifs/index.json');
      if (!res.ok) throw new Error('index.json introuvable');
      const data = await res.json();
      localGifs = data.gifs ?? [];
      // Construire la liste des catégories dans l'ordre d'apparition
      const seen = new Set<string>();
      const cats: string[] = [];
      for (const g of localGifs) {
        if (!seen.has(g.cat_label)) { seen.add(g.cat_label); cats.push(g.cat_label); }
      }
      gifCats  = cats;
      gifCat   = cats[0] ?? '';
      gifsLoaded = true;
    } catch {
      gifsError = true;
      gifsLoaded = true;
    }
  }

  function handleToggleEmojiPicker() {
    toggleEmojiPicker();
    if (!chatStore.showEmojiPicker) return;
    if (pickerTab === 'gif') loadLocalGifs();
  }

  function handleSelectGif(filename: string) {
    const url = `/gifs/${filename}`;
    sendEmoji(`<img src="${url}" alt="gif" class="chat-gif" loading="lazy" />`, activeConvId);
    chatStore.showEmojiPicker = false;
  }
  // tous les emojis disponibles (picker étendu)
  const ALL_EMOJIS = ['👍','👎','❤️','🔥','😂','😮','😢','😡','🎉','🙏','✅','❌','🤔','😍','🥺','😎'];

  /** Détecte si un message est un unique emoji (affichage agrandi 2.5rem) */
  /** Détecte si un message ne contient QUE des emojis (affichage agrandi) */
  function isEmojiOnly(content: string): boolean {
    const t = content.trim();
    if (t.length === 0 || t.length > 30) return false;
    // Match: one or more emoji sequences, optionally separated by zero-width joiners
    const emojiRe = /^(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F|\p{Extended_Pictographic})(?:\u200D(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F|\p{Extended_Pictographic}))*$/u;
    // Allow sequences of emojis (up to ~10)
    const multiEmojiRe = new RegExp(
      '^((?:[\p{Emoji_Presentation}\p{Emoji}\uFE0F\p{Extended_Pictographic}])(?:\u200D[\p{Emoji_Presentation}\p{Emoji}\uFE0F\p{Extended_Pictographic}])*){1,10}$',
      'u'
    );
    return multiEmojiRe.test(t);
  }

  async function toggleReaction(msgId: string, emoji: string) {
    const cur = reactions[msgId];
    const isMyEmoji = cur?.myEmoji === emoji;

    try {
      const convId = activeConvId;
      let res: Response;
      if (isMyEmoji) {
        res = await fetch(`/api/conversations/${convId}/messages/${msgId}/reactions`, {
          method: 'DELETE', credentials: 'include',
        });
      } else {
        res = await fetch(`/api/conversations/${convId}/messages/${msgId}/reactions`, {
          method: 'POST', credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ emoji }),
        });
      }
      if (res.ok) {
        const data = await res.json();
        reactions[msgId] = { counts: data.counts ?? {}, myEmoji: data.my_emoji ?? null };
      }
    } catch (e) {
      console.error('[Reaction]', e);
    }
    emojiPickerMsgId = null;
  }

  function countReactions(msgId: string): { emoji: string; count: number; names: string }[] {
    const r = reactions[msgId];
    if (!r) return [];
    return Object.entries(r.counts)
      .filter(([, names]) => names.length > 0)
      .map(([emoji, names]) => ({ emoji, count: names.length, names: names.join(', ') }));
  }

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
    if (conv.id === 'default_global') return 'Nook';
    if (conv.is_group) return conv.name ?? 'Groupe sans nom';
    // DM : nom de l'autre participant depuis le cache
    const parts = participantsCache[conv.id] ?? [];
    const other = parts.find(p => p.id !== authStore.user?.id);
    return other ? (other.name ?? other.username) : (conv.name ?? 'Message direct');
  }

  function convAvatar(conv: Conv): string {
    if (conv.id === 'default_global') return '🌿';
    return conv.is_group ? '👥' : '💬';
  }

  // ─────────────────────────────────────────────────────────────────
  // Sélection conversation active
  // ─────────────────────────────────────────────────────────────────

  async function selectConversation(conv: Conv) {
    activeConvId = conv.id;
    activeConv   = conv;

    if (conv.id === 'default_global') {
      activeConvName = '🌿 Nook';
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

    // Activer la conv : connecte le WS, reset badge non-lus, charge les messages
    setActiveConv(conv.id);
    await loadMessages(conv.id);
    // Scroll immédiat en bas après chargement des messages
    await Promise.resolve();
    if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
    // Charger les réactions pour les messages visibles
    await loadReactionsForMessages(conv.id);
    // Fallback polling si WS non disponible
    if (pollTimer) clearInterval(pollTimer);
    if (!chatStore.wsConnected) {
      pollTimer = setInterval(() => loadMessages(conv.id), 8000);
    }
  }

  async function loadReactionsForMessages(convId: string) {
    // Charger les réactions des messages visibles en parallèle (max 50)
    const msgs = chatStore.messages.slice(-50);
    await Promise.allSettled(msgs.map(async (msg) => {
      try {
        const res = await fetch(
          `/api/conversations/${convId}/messages/${msg.id}/reactions`,
          { credentials: 'include' }
        );
        if (res.ok) {
          const data = await res.json();
          reactions[msg.id] = { counts: data.counts ?? {}, myEmoji: data.my_emoji ?? null };
        }
      } catch { /* non-bloquant */ }
    }));
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
    chatStore.showEmojiPicker = false;
    await sendMessage(content, activeConvId);
    sending = false;
  }

  function handleMessageKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSendMessage(); }
  }

  function handleSubmit(e: Event) { e.preventDefault(); handleSendMessage(); }

  function handleSelectEmoji(emoji: string) {
    // Toujours ajouter l'emoji au champ de saisie
    // L'utilisateur peut empiler plusieurs emojis puis envoyer
    newMessage = newMessage + emoji;
    // Ne pas fermer le picker → permet de sélectionner plusieurs emojis d'affilée
  }

  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    const file = input.files[0];

    // Vérification taille côté client (limite backend = 50 Mo)
    const MAX_BYTES = 50 * 1024 * 1024;
    if (file.size > MAX_BYTES) {
      chatStore.connectionError = `Fichier trop volumineux (${(file.size / 1024 / 1024).toFixed(1)} Mo). Limite : 50 Mo.`;
      input.value = '';
      setTimeout(() => chatStore.connectionError = null, 5000);
      return;
    }

    const fd = new FormData();
    fd.append('file', file);
    fd.append('conversation_id', activeConvId);
    fd.append('from_user_id', authStore.user?.id || '');
    try {
      const res = await fetch('/api/upload/chat', { method: 'POST', body: fd, credentials: 'include' });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error ?? `Upload échoué (HTTP ${res.status})`);
      }
      const data = await res.json();
      const isImage = data.is_image ?? file.type.startsWith('image/');
      const uploadContent = isImage
        ? `<img src="/api/download/${data.file_id}" alt="${data.file_name}" class="uploaded-image" />`
        : `<span class="file-attachment">📎 <a href="/api/download/${data.file_id}" download="${data.file_name}">${data.file_name}</a></span>`;
      await sendMessage(uploadContent, activeConvId);
      input.value = '';
    } catch (err: unknown) {
      console.error('[Upload]', err);
      chatStore.connectionError = err instanceof Error ? err.message : "Échec de l'upload";
      setTimeout(() => chatStore.connectionError = null, 5000);
    }
  }

  async function handleVoiceRecord(mediaType: 'audio' | 'video' = 'audio') {
    if (recordingState.isRecording) {
      // Arrêt : récupérer le blob et l'envoyer comme upload
      try {
        const blob = await stopRecording(true);
        if (!blob) return;

        // Vérifier durée max côté client
        const maxSec = mediaType === 'video' ? MAX_VIDEO_SEC : MAX_AUDIO_SEC;
        if (recordingState.duration > maxSec) {
          chatStore.connectionError = `Enregistrement trop long (max ${maxSec}s).`;
          setTimeout(() => chatStore.connectionError = null, 4000);
          return;
        }

        // Uploader via le endpoint existant /api/upload/chat
        const ext  = blob.type.includes('ogg') ? 'ogg' : blob.type.includes('mp4') ? 'mp4' : 'webm';
        const name = `vocal_${Date.now()}.${ext}`;
        const file = new File([blob], name, { type: blob.type });

        const fd = new FormData();
        fd.append('file', file);
        fd.append('conversation_id', activeConvId);
        fd.append('from_user_id', authStore.user?.id || '');

        const res = await fetch('/api/upload/chat', { method: 'POST', body: fd, credentials: 'include' });
        if (!res.ok) throw new Error(`Upload échoué (HTTP ${res.status})`);
        const data = await res.json();

        // Envoyer un message avec le tag html <audio> ou <video>
        // Le contenu est sanitisé côté affichage via DOMPurify qui autorise <audio>/<video>
        const tag     = mediaType === 'video' ? 'video' : 'audio';
        const content = `<${tag} src="/api/download/${data.file_id}" controls preload="none" class="voice-${tag}"></${tag}>`;
        await sendMessage(content, activeConvId);
      } catch (err: unknown) {
        console.error('[VoiceRecord stop]', err);
        chatStore.connectionError = err instanceof Error ? err.message : 'Erreur envoi vocal';
        setTimeout(() => chatStore.connectionError = null, 5000);
      }
    } else {
      // Démarrage
      try {
        await startRecording(mediaType);
      } catch (err: unknown) {
        console.error('[VoiceRecord start]', err);
        // Erreur déjà dans recordingState.error — pas de doublon
      }
    }
  }

  function isMyMessage(senderId: string) { return authStore.user?.id === senderId; }

  function startEdit(msg: { id: string; content: string }) {
    editingMsgId   = msg.id;
    editingContent = msg.content;
  }

  async function submitEdit() {
    if (!editingMsgId || !editingContent.trim()) { cancelEdit(); return; }
    await editMessage(editingMsgId, activeConvId, editingContent.trim());
    cancelEdit();
  }

  function cancelEdit() {
    editingMsgId   = null;
    editingContent = '';
  }

  async function confirmDelete(msgId: string) {
    if (!confirm('Supprimer ce message ?')) return;
    await deleteMessage(msgId, activeConvId);
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(); }
    if (e.key === 'Escape') cancelEdit();
  }

  /** Pagination — déclenché au scroll en haut du conteneur de messages */
  async function handleMessagesScroll(e: Event) {
    const el = e.target as HTMLElement;
    if (el.scrollTop < 80 && chatStore.hasMore && !chatStore.loadingMore) {
      const prevHeight = el.scrollHeight;
      await loadMoreMessages(activeConvId);
      // Maintenir la position de scroll après insertion en haut
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight - prevHeight;
      });
    }
  }

  // ─────────────────────────────────────────────────────────────────
  // Cycle de vie
  // ─────────────────────────────────────────────────────────────────

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await loadConversations();
    await loadMessages(activeConvId);
    await loadReactionsForMessages(activeConvId);
    setActiveConv(activeConvId);
    // Demande permission notifications (non-bloquant)
    requestNotificationPermission();
    // Fallback polling si WS pas connecté après 3s
    setTimeout(() => {
      if (!chatStore.wsConnected) {
        pollTimer = setInterval(() => loadMessages(activeConvId), 8000);
      }
    }, 3000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    disconnectWs();
  });

  $effect(() => {
    const count = chatStore.messages.length;
    if (!chatContainer || count === 0) return;
    // Ne pas forcer le scroll si l'utilisateur a remonté pour lire l'historique
    // Tolérance : si on est à moins de 150px du bas → scroll auto
    const el = chatContainer;
    const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 150;
    if (isNearBottom || count === 1) {
      // Attendre le prochain tick (DOM mis à jour)
      Promise.resolve().then(() => {
        if (chatContainer) {
          chatContainer.scrollTop = chatContainer.scrollHeight;
        }
      });
    }
  });

  // Rafraîchir la réaction d'un seul message à la réception du signal WS
  $effect(() => {
    const update = chatStore.lastReactionUpdate;
    if (!update || update.conversationId !== activeConvId) return;
    const { messageId, conversationId } = update;
    fetch(`/api/conversations/${conversationId}/messages/${messageId}/reactions`, {
      credentials: 'include',
    }).then(r => r.ok ? r.json() : null).then(data => {
      if (data) reactions[messageId] = { counts: data.counts ?? {}, myEmoji: data.my_emoji ?? null };
    }).catch(() => {});
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
            onclick={() => selectConversation({ id: 'default_global', name: 'Nook', is_group: true, updated_at: 0, created_by: '' })}
          >
            <span class="avatar">🌿</span>
            <div class="conversation-info">
              <span class="name">Nook</span>
              <span class="preview">Canal familial</span>
            </div>
            {#if (chatStore.unreadCounts['default_global'] ?? 0) > 0}
              <span class="unread-badge">{chatStore.unreadCounts['default_global']}</span>
            {/if}
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
            {#if (chatStore.unreadCounts[conv.id] ?? 0) > 0}
              <span class="unread-badge">{chatStore.unreadCounts[conv.id]}</span>
            {/if}
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
      {#if renamingConv}
        <input
          class="rename-input"
          type="text"
          bind:value={renameValue}
          onkeydown={handleRenameKeydown}
          onblur={submitRename}
          maxlength="60"
          placeholder="Nom du groupe…"
        />
        <button class="rename-ok" onclick={submitRename} aria-label="Valider">✓</button>
        <button class="rename-cancel" onclick={() => renamingConv = false} aria-label="Annuler">✕</button>
      {:else}
        <h2>{activeConvName}</h2>
        {#if chatStore.connectionError}
          <span class="conn-error">⚠️ {chatStore.connectionError}</span>
        {/if}
        {#if activeConv && activeConv.is_group && activeConv.id !== 'default_global'}
          <button class="rename-btn" onclick={startRename} title="Renommer le groupe" aria-label="Renommer">✏️</button>
        {/if}
        {#if activeConv && !activeConv.is_group && activeConv.id !== 'default_global'}
          <div class="call-actions">
            <a href="/call/{activeConv.id}?type=audio" class="call-btn call-btn--audio" title="Appel audio" aria-label="Démarrer un appel audio">🎤</a>
            <a href="/call/{activeConv.id}?type=video" class="call-btn call-btn--video" title="Appel vidéo" aria-label="Démarrer un appel vidéo">📹</a>
          </div>
        {/if}
      {/if}
    </header>

    <div class="messages-container" bind:this={chatContainer} onscroll={handleMessagesScroll}>
      {#if chatStore.loadingMore}
        <div class="load-more-indicator">⏳ Chargement…</div>
      {:else if chatStore.hasMore}
        <button class="load-more-btn" onclick={() => handleMessagesScroll({ target: chatContainer } as unknown as Event)}>
          ↑ Messages précédents
        </button>
      {/if}

      {#if chatStore.messages.length === 0}
        <div class="empty-state">
          <span class="empty-icon">💬</span>
          <p>Aucun message — soyez le premier à écrire !</p>
        </div>
      {:else}
        {#virtual each chatStore.messages as msg (msg.id)}
          <div
            class="message"
            class:mine={isMyMessage(msg.sender_id)}
            onmouseenter={() => { clearTimeout(_hoverTimer); hoveredMsgId = msg.id; }}
            onmouseleave={() => { _hoverTimer = setTimeout(() => { if (editingMsgId !== msg.id) hoveredMsgId = null; }, 400); }}
          >
            {#if !isMyMessage(msg.sender_id)}
              <div class="message-sender">{msg.sender_name || msg.sender_id}</div>
            {/if}

            {#if editingMsgId === msg.id}
              <div class="edit-zone">
                <textarea
                  class="edit-input"
                  bind:value={editingContent}
                  onkeydown={handleEditKeydown}
                  rows="2"
                ></textarea>
                <div class="edit-actions">
                  <button class="edit-ok" onclick={submitEdit}>✓ Sauvegarder</button>
                  <button class="edit-cancel" onclick={cancelEdit}>✕ Annuler</button>
                </div>
              </div>
            {:else}
              <!-- SEC-01 FIX : DOMPurify sanitize — jamais {@html} brut -->
              <!-- Messages vocaux : <audio>/<video> natif si le contenu commence par ces tags -->
              {#if msg.content.trimStart().startsWith('<audio')}
                <div class="voice-message">
                  🎤 <audio
                    src={msg.content.match(/src="([^"]+)"/)?.[1] ?? ''}
                    controls
                    preload="none"
                    class="voice-audio"
                  ></audio>
                </div>
              {:else if msg.content.trimStart().startsWith('<video')}
                <div class="voice-message">
                  🎥 <video
                    src={msg.content.match(/src="([^"]+)"/)?.[1] ?? ''}
                    controls
                    preload="none"
                    class="voice-video"
                  ></video>
                </div>
              {:else}
                {#if isEmojiOnly(msg.content)}
                  <div class="message-content emoji-only">{msg.content}</div>
                {:else}
                  <div class="message-content">{@html sanitizeHtml(msg.content)}</div>
                {/if}
              {/if}
            {/if}

            <!-- ─── Réactions affichées ─── -->
            {#if countReactions(msg.id).length > 0}
              <div class="reactions-row">
                {#each countReactions(msg.id) as r}
                  <button
                    class="reaction-pill"
                    class:my-reaction={reactions[msg.id]?.myEmoji === r.emoji}
                    onclick={() => toggleReaction(msg.id, r.emoji)}
                    title={r.names}
                    aria-label="{r.emoji} {r.count}"
                  >{r.emoji} {r.count}</button>
                {/each}
              </div>
            {/if}

            <div class="message-meta">
              <span class="message-time">{formatTimestamp(msg.timestamp)}</span>
              {#if msg.edited_at}
                <span class="edited-label">(modifié)</span>
              {/if}
            </div>

            {#if hoveredMsgId === msg.id && editingMsgId !== msg.id}
              <div class="msg-actions" class:mine-actions={isMyMessage(msg.sender_id)}>
                <!-- Bouton réaction rapide — toujours visible au hover -->
                <button
                  class="msg-action-btn reaction-trigger"
                  onclick={(e) => { e.stopPropagation(); emojiPickerMsgId = emojiPickerMsgId === msg.id ? null : msg.id; }}
                  title="Réagir"
                  aria-label="Ajouter une réaction"
                >😊</button>
                {#if isMyMessage(msg.sender_id)}
                  <button class="msg-action-btn" onclick={() => startEdit(msg)} title="Modifier">✏️</button>
                {/if}
                {#if isMyMessage(msg.sender_id) || authStore.isAdmin}
                  <button class="msg-action-btn danger" onclick={() => confirmDelete(msg.id)} title="Supprimer">🗑️</button>
                {/if}
              </div>

              <!-- Emoji picker rapide (6 fixes + picker étendu) -->
              {#if emojiPickerMsgId === msg.id}
                <div
                  class="emoji-picker"
                  class:picker-mine={isMyMessage(msg.sender_id)}
                  role="dialog"
                  aria-label="Choisir une réaction"
                >
                  {#each QUICK_EMOJIS as emoji}
                    <button
                      class="emoji-quick-btn"
                      class:emoji-active={reactions[msg.id]?.myEmoji === emoji}
                      onclick={() => toggleReaction(msg.id, emoji)}
                      aria-label={emoji}
                    >{emoji}</button>
                  {/each}
                  <!-- Bouton + pour picker étendu -->
                  <button
                    class="emoji-more-btn"
                    onclick={(e) => { e.stopPropagation(); /* toggle zone étendue */ const el = e.currentTarget.nextElementSibling as HTMLElement; if (el) el.style.display = el.style.display === 'none' ? 'flex' : 'none'; }}
                    aria-label="Plus d'emojis"
                  >＋</button>
                  <!-- Zone étendue (cachée par défaut) -->
                  <div class="emoji-extended" style="display:none">
                    {#each ALL_EMOJIS as emoji}
                      <button
                        class="emoji-quick-btn"
                        class:emoji-active={reactions[msg.id]?.myEmoji === emoji}
                        onclick={() => toggleReaction(msg.id, emoji)}
                        aria-label={emoji}
                      >{emoji}</button>
                    {/each}
                  </div>
                </div>
              {/if}
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    {#if chatStore.showEmojiPicker}
      <div class="emoji-panel" role="dialog" aria-label="Picker emoji ou GIF" tabindex="-1">
        <div class="ep-tabs">
          <button class="ep-tab" class:active={pickerTab === 'emoji'}
            onclick={() => { pickerTab = 'emoji'; }}>😊 Emoji</button>
          <button class="ep-tab" class:active={pickerTab === 'gif'}
            onclick={() => { pickerTab = 'gif'; loadLocalGifs(); }}>🎬 GIF</button>
          <div class="ep-tab-spacer"></div>
          <button class="ep-close" onclick={toggleEmojiPicker} aria-label="Fermer">✕</button>
        </div>

        {#if pickerTab === 'emoji'}
        <div class="ep-header">
          <div class="ep-cats"><button class="ep-cat-btn" class:active={emojiCat === '😊' } onclick={() => emojiCat = '😊'}>😊</button><button class="ep-cat-btn" class:active={emojiCat === '👋' } onclick={() => emojiCat = '👋'}>👋</button><button class="ep-cat-btn" class:active={emojiCat === '❤️' } onclick={() => emojiCat = '❤️'}>❤️</button><button class="ep-cat-btn" class:active={emojiCat === '🎉' } onclick={() => emojiCat = '🎉'}>🎉</button><button class="ep-cat-btn" class:active={emojiCat === '🐶' } onclick={() => emojiCat = '🐶'}>🐶</button><button class="ep-cat-btn" class:active={emojiCat === '🍕' } onclick={() => emojiCat = '🍕'}>🍕</button><button class="ep-cat-btn" class:active={emojiCat === '⚽' } onclick={() => emojiCat = '⚽'}>⚽</button><button class="ep-cat-btn" class:active={emojiCat === '🌍' } onclick={() => emojiCat = '🌍'}>🌍</button></div>
          <button class="ep-close" onclick={toggleEmojiPicker} aria-label="Fermer">✕</button>
        </div>
        <div class="ep-body">
            {#if emojiCat === '😊'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('😀')} title="😀">😀</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😁')} title="😁">😁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😂')} title="😂">😂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤣')} title="🤣">🤣</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😃')} title="😃">😃</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😄')} title="😄">😄</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😅')} title="😅">😅</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😆')} title="😆">😆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😊')} title="😊">😊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😋')} title="😋">😋</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😎')} title="😎">😎</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥳')} title="🥳">🥳</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤩')} title="🤩">🤩</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😍')} title="😍">😍</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥰')} title="🥰">🥰</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😘')} title="😘">😘</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😗')} title="😗">😗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😙')} title="😙">😙</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😚')} title="😚">😚</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤗')} title="🤗">🤗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤭')} title="🤭">🤭</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤫')} title="🤫">🤫</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤔')} title="🤔">🤔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😐')} title="😐">😐</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😑')} title="😑">😑</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😶')} title="😶">😶</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🙄')} title="🙄">🙄</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😏')} title="😏">😏</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😒')} title="😒">😒</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😞')} title="😞">😞</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😔')} title="😔">😔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😟')} title="😟">😟</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😕')} title="😕">😕</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🙁')} title="🙁">🙁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☹️')} title="☹️">☹️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😣')} title="😣">😣</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😖')} title="😖">😖</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😫')} title="😫">😫</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😩')} title="😩">😩</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥺')} title="🥺">🥺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😢')} title="😢">😢</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😭')} title="😭">😭</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😤')} title="😤">😤</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😠')} title="😠">😠</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😡')} title="😡">😡</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤬')} title="🤬">🤬</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😱')} title="😱">😱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😨')} title="😨">😨</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😰')} title="😰">😰</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😥')} title="😥">😥</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😓')} title="😓">😓</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤯')} title="🤯">🤯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😬')} title="😬">😬</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥴')} title="🥴">🥴</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😵')} title="😵">😵</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤪')} title="🤪">🤪</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😜')} title="😜">😜</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😝')} title="😝">😝</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😛')} title="😛">😛</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤑')} title="🤑">🤑</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('😈')} title="😈">😈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👿')} title="👿">👿</button></div>{/if}
            {#if emojiCat === '👋'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('👍')} title="👍">👍</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👎')} title="👎">👎</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👌')} title="👌">👌</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤌')} title="🤌">🤌</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤏')} title="🤏">🤏</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✌️')} title="✌️">✌️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤞')} title="🤞">🤞</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤟')} title="🤟">🤟</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤘')} title="🤘">🤘</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤙')} title="🤙">🤙</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👈')} title="👈">👈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👉')} title="👉">👉</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👆')} title="👆">👆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👇')} title="👇">👇</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☝️')} title="☝️">☝️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👋')} title="👋">👋</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤚')} title="🤚">🤚</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🖐️')} title="🖐️">🖐️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✋')} title="✋">✋</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🖖')} title="🖖">🖖</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤜')} title="🤜">🤜</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤛')} title="🤛">🤛</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👊')} title="👊">👊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✊')} title="✊">✊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤝')} title="🤝">🤝</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🙌')} title="🙌">🙌</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('👏')} title="👏">👏</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤲')} title="🤲">🤲</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🙏')} title="🙏">🙏</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✍️')} title="✍️">✍️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💪')} title="💪">💪</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦾')} title="🦾">🦾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🫶')} title="🫶">🫶</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('❤️‍🔥')} title="❤️‍🔥">❤️‍🔥</button></div>{/if}
            {#if emojiCat === '❤️'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('❤️')} title="❤️">❤️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧡')} title="🧡">🧡</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💛')} title="💛">💛</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💚')} title="💚">💚</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💙')} title="💙">💙</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💜')} title="💜">💜</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🖤')} title="🖤">🖤</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤍')} title="🤍">🤍</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤎')} title="🤎">🤎</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💔')} title="💔">💔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('❣️')} title="❣️">❣️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💕')} title="💕">💕</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💞')} title="💞">💞</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💓')} title="💓">💓</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💗')} title="💗">💗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💖')} title="💖">💖</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💘')} title="💘">💘</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💝')} title="💝">💝</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💟')} title="💟">💟</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☮️')} title="☮️">☮️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💯')} title="💯">💯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✨')} title="✨">✨</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⭐')} title="⭐">⭐</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌟')} title="🌟">🌟</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💫')} title="💫">💫</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🔥')} title="🔥">🔥</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💥')} title="💥">💥</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('❄️')} title="❄️">❄️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌈')} title="🌈">🌈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☀️')} title="☀️">☀️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌙')} title="🌙">🌙</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⚡')} title="⚡">⚡</button></div>{/if}
            {#if emojiCat === '🎉'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎉')} title="🎉">🎉</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎊')} title="🎊">🎊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎈')} title="🎈">🎈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎁')} title="🎁">🎁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎂')} title="🎂">🎂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍰')} title="🍰">🍰</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥂')} title="🥂">🥂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍾')} title="🍾">🍾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎆')} title="🎆">🎆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎇')} title="🎇">🎇</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('✨')} title="✨">✨</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥳')} title="🥳">🥳</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎤')} title="🎤">🎤</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎵')} title="🎵">🎵</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎶')} title="🎶">🎶</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎸')} title="🎸">🎸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎹')} title="🎹">🎹</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎺')} title="🎺">🎺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎻')} title="🎻">🎻</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥁')} title="🥁">🥁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎮')} title="🎮">🎮</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🕹️')} title="🕹️">🕹️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎯')} title="🎯">🎯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎱')} title="🎱">🎱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏆')} title="🏆">🏆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥇')} title="🥇">🥇</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥈')} title="🥈">🥈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥉')} title="🥉">🥉</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎖️')} title="🎖️">🎖️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏅')} title="🏅">🏅</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎗️')} title="🎗️">🎗️</button></div>{/if}
            {#if emojiCat === '🐶'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐶')} title="🐶">🐶</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐱')} title="🐱">🐱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐭')} title="🐭">🐭</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐹')} title="🐹">🐹</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐰')} title="🐰">🐰</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦊')} title="🦊">🦊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐻')} title="🐻">🐻</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐼')} title="🐼">🐼</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐨')} title="🐨">🐨</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐯')} title="🐯">🐯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦁')} title="🦁">🦁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐮')} title="🐮">🐮</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐷')} title="🐷">🐷</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐸')} title="🐸">🐸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐵')} title="🐵">🐵</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐔')} title="🐔">🐔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐧')} title="🐧">🐧</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐦')} title="🐦">🐦</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦆')} title="🦆">🦆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦅')} title="🦅">🦅</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦉')} title="🦉">🦉</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦇')} title="🦇">🦇</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐺')} title="🐺">🐺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐗')} title="🐗">🐗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦝')} title="🦝">🦝</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦨')} title="🦨">🦨</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦡')} title="🦡">🦡</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦦')} title="🦦">🦦</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦥')} title="🦥">🦥</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐿️')} title="🐿️">🐿️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🦔')} title="🦔">🦔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐾')} title="🐾">🐾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🐲')} title="🐲">🐲</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌸')} title="🌸">🌸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌺')} title="🌺">🌺</button></div>{/if}
            {#if emojiCat === '🍕'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍕')} title="🍕">🍕</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍔')} title="🍔">🍔</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌮')} title="🌮">🌮</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌯')} title="🌯">🌯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥗')} title="🥗">🥗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍜')} title="🍜">🍜</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍱')} title="🍱">🍱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍣')} title="🍣">🍣</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍩')} title="🍩">🍩</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍪')} title="🍪">🍪</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍫')} title="🍫">🍫</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍬')} title="🍬">🍬</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍭')} title="🍭">🍭</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☕')} title="☕">☕</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍵')} title="🍵">🍵</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧃')} title="🧃">🧃</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥤')} title="🥤">🥤</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍺')} title="🍺">🍺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍻')} title="🍻">🍻</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥂')} title="🥂">🥂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍷')} title="🍷">🍷</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍸')} title="🍸">🍸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍹')} title="🍹">🍹</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧁')} title="🧁">🧁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎂')} title="🎂">🎂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍰')} title="🍰">🍰</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥧')} title="🥧">🥧</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍮')} title="🍮">🍮</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍯')} title="🍯">🍯</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧆')} title="🧆">🧆</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥘')} title="🥘">🥘</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🫕')} title="🫕">🫕</button></div>{/if}
            {#if emojiCat === '⚽'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('⚽')} title="⚽">⚽</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏀')} title="🏀">🏀</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏈')} title="🏈">🏈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⚾')} title="⚾">⚾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎾')} title="🎾">🎾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏐')} title="🏐">🏐</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏉')} title="🏉">🏉</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥏')} title="🥏">🥏</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎱')} title="🎱">🎱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏓')} title="🏓">🏓</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏸')} title="🏸">🏸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥊')} title="🥊">🥊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⛷️')} title="⛷️">⛷️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏂')} title="🏂">🏂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏋️')} title="🏋️">🏋️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤸')} title="🤸">🤸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⛹️')} title="⛹️">⛹️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤺')} title="🤺">🤺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏇')} title="🏇">🏇</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧘')} title="🧘">🧘</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏄')} title="🏄">🏄</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🚴')} title="🚴">🚴</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🧗')} title="🧗">🧗</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤾')} title="🤾">🤾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏌️')} title="🏌️">🏌️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏊')} title="🏊">🏊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🤽')} title="🤽">🤽</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥋')} title="🥋">🥋</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🥅')} title="🥅">🥅</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⛳')} title="⛳">⛳</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🎣')} title="🎣">🎣</button></div>{/if}
            {#if emojiCat === '🌍'}<div class="ep-grid"><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌍')} title="🌍">🌍</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌲')} title="🌲">🌲</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌳')} title="🌳">🌳</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌴')} title="🌴">🌴</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌵')} title="🌵">🌵</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌿')} title="🌿">🌿</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('☘️')} title="☘️">☘️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍀')} title="🍀">🍀</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍁')} title="🍁">🍁</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍂')} title="🍂">🍂</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍃')} title="🍃">🍃</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌸')} title="🌸">🌸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌹')} title="🌹">🌹</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌺')} title="🌺">🌺</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌻')} title="🌻">🌻</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌼')} title="🌼">🌼</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('💐')} title="💐">💐</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🍄')} title="🍄">🍄</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌾')} title="🌾">🌾</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌱')} title="🌱">🌱</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🪴')} title="🪴">🪴</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🪸')} title="🪸">🪸</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⛰️')} title="⛰️">⛰️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏔️')} title="🏔️">🏔️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌋')} title="🌋">🌋</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏝️')} title="🏝️">🏝️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🏖️')} title="🏖️">🏖️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌊')} title="🌊">🌊</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌅')} title="🌅">🌅</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌄')} title="🌄">🌄</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌠')} title="🌠">🌠</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌌')} title="🌌">🌌</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌪️')} title="🌪️">🌪️</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('🌈')} title="🌈">🌈</button><button class="ep-emoji" onclick={()=>handleSelectEmoji('⛅')} title="⛅">⛅</button></div>{/if}
        </div>
        {/if}

        {#if pickerTab === 'gif'}
          <div class="ep-body gif-body">
            {#if !gifsLoaded}
              <div class="gif-loading">Chargement…</div>
            {:else if gifsError}
              <div class="gif-loading">
                ⚠️ GIFs non disponibles<br>
                <span class="gif-hint">Lance le workflow <code>fetch-gifs.yml</code> pour peupler la collection.</span>
              </div>
            {:else if localGifs.length === 0}
              <div class="gif-loading">
                Aucun GIF — lance <code>fetch-gifs.yml</code>
              </div>
            {:else}
              <div class="gif-cats">
                {#each gifCats as cat}
                  <button class="ep-cat-btn" class:active={gifCat === cat}
                    onclick={() => gifCat = cat}>{cat}</button>
                {/each}
              </div>
              <div class="gif-grid">
                {#each localGifs.filter(g => g.cat_label === gifCat) as gif}
                  <button class="gif-thumb" onclick={() => handleSelectGif(gif.file)}
                    title={gif.title}>
                    <img src="/gifs/{gif.file}" alt={gif.title} loading="lazy" />
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

      </div>
    {/if}

    {#if recordingState.isRecording}
      <!-- Barre d'enregistrement en cours -->
      <div class="recording-bar">
        <span class="recording-dot"></span>
        <span class="recording-timer">⏱ {formatDuration(recordingState.duration)}</span>
        {#if recordingState.mediaType === 'video'}
          <span class="recording-type">Vidéo</span>
          <span class="recording-limit">(max {MAX_VIDEO_SEC}s)</span>
        {:else}
          <span class="recording-type">Audio</span>
          <span class="recording-limit">(max {MAX_AUDIO_SEC}s)</span>
        {/if}
        {#if recordingState.error}
          <span class="recording-error">{recordingState.error}</span>
        {/if}
        <div class="recording-actions">
          <button type="button" class="rec-btn rec-stop" onclick={() => handleVoiceRecord(recordingState.mediaType ?? 'audio')} title="Envoyer">
            ✅ Envoyer
          </button>
          <button type="button" class="rec-btn rec-cancel" onclick={cancelRecording} title="Annuler">
            ❌
          </button>
        </div>
      </div>
    {/if}

    <form class="input-area" onsubmit={handleSubmit}>
      <button type="button" class="icon-btn" onclick={() => fileInput?.click()} title="Joindre">📎</button>
      <!-- File transfer progress -->


<input type="file" bind:this={fileInput} onchange={handleFileUpload} style="display:none" />
      <button type="button" class="icon-btn emoji-open-btn" onclick={handleToggleEmojiPicker} title="Emoji / GIF" aria-label="Ouvrir le picker emoji ou GIF">😊</button>
      <!-- Bouton message vocal -->
      <button
        type="button"
        class="icon-btn"
        class:recording={recordingState.isRecording && recordingState.mediaType === 'audio'}
        onclick={() => handleVoiceRecord('audio')}
        title={recordingState.isRecording ? 'Arrêter et envoyer' : 'Message vocal'}
        aria-label={recordingState.isRecording ? 'Arrêter et envoyer le message vocal' : 'Démarrer un message vocal'}
        disabled={recordingState.isRecording && recordingState.mediaType === 'video'}
      >🎙️</button>
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
    height: calc(100dvh - var(--header-h, 60px));
    overflow: hidden;
    max-width: 100%;
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
    overflow: hidden;
    height: 100%;
    position: relative;
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
  .rename-btn {
    display: flex; align-items: center; justify-content: center;
    background: none; border: none; cursor: pointer;
    font-size: .95rem; padding: .2rem .3rem; border-radius: .35rem;
    opacity: .5; transition: opacity .15s, background .15s; flex-shrink: 0;
  }
  .rename-btn:hover { opacity: 1; background: var(--bg-secondary, #f1f5f9); }
  .rename-input {
    flex: 1; min-width: 0; font-size: 1rem; font-weight: 600;
    color: var(--text-primary, #1e293b);
    border: none; border-bottom: 2px solid var(--accent, #4ade80);
    background: transparent; outline: none; padding: 0 .25rem;
  }
  .rename-ok, .rename-cancel {
    background: none; border: none; cursor: pointer;
    font-size: 1rem; padding: .2rem .35rem; border-radius: .35rem;
    flex-shrink: 0; transition: background .15s;
  }
  .rename-ok    { color: #166534; }
  .rename-ok:hover   { background: #dcfce7; }
  .rename-cancel     { color: #dc2626; }
  .rename-cancel:hover { background: #fee2e2; }

  /* ─── Messages ─── */
  .messages-container {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: .5rem;
    scroll-behavior: smooth;
    overscroll-behavior: contain;
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
    min-height: 60px; /* Consistent height for virtual scrolling */
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
    max-width: 400px; border-radius: 8px; margin-top: .3rem; display: block;
  }
  .message-time {
    font-size: .68rem; color: var(--text-secondary, #94a3b8);
    margin-top: .25rem; text-align: right;
  }
  .message-meta { display: flex; gap: .4rem; align-items: center; justify-content: flex-end; }
  .edited-label { font-size: .62rem; color: var(--text-secondary, #94a3b8); font-style: italic; }

  /* Unread badge */
  .unread-badge {
    display: flex; align-items: center; justify-content: center;
    background: var(--accent, #4ade80); color: #166534;
    font-size: .65rem; font-weight: 700; border-radius: 999px;
    min-width: 1.25rem; height: 1.25rem; padding: 0 .3rem;
    flex-shrink: 0; margin-left: auto;
  }

  /* Pagination */
  .load-more-btn {
    display: block; width: 100%; background: none;
    border: 1px dashed var(--border, #e2e8f0); border-radius: .5rem;
    color: var(--text-secondary, #94a3b8); font-size: .8rem;
    cursor: pointer; padding: .4rem; margin-bottom: .5rem; transition: background .15s;
  }
  .load-more-btn:hover { background: var(--bg-secondary, #f8fafc); }
  .load-more-indicator { text-align: center; font-size: .8rem; color: var(--text-secondary, #94a3b8); padding: .5rem; }

  /* Menu contextuel message */
  .message { position: relative; }
  .msg-actions {
    position: absolute; top: .2rem; right: .2rem;
    display: flex; gap: .2rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .45rem; padding: .15rem;
    box-shadow: 0 2px 8px rgba(0,0,0,.08);
    z-index: 10;
  }
  .mine-actions { right: auto; left: .2rem; }
  .msg-action-btn {
    background: none; border: none; cursor: pointer; font-size: .8rem;
    padding: .2rem .3rem; border-radius: .3rem; transition: background .12s;
  }
  .msg-action-btn:hover { background: var(--bg-secondary, #f1f5f9); }
  .msg-action-btn.danger:hover { background: var(--error-light, #fee2e2); }

  /* ─── Réactions ─── */
  .reactions-row {
    display: flex; flex-wrap: wrap; gap: .25rem;
    margin-top: .3rem;
  }
  .reaction-pill {
    display: inline-flex; align-items: center; gap: .2rem;
    padding: .15rem .45rem; border-radius: 999px;
    border: 1.5px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    font-size: .8rem; cursor: pointer; transition: all .12s;
    color: var(--text-primary, #1e293b);
  }
  .reaction-pill:hover { border-color: var(--accent, #4ade80); background: var(--bg-tertiary, #f0fdf4); }
  .reaction-pill.my-reaction {
    border-color: var(--accent, #4ade80);
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    font-weight: 600;
  }

  /* Emoji picker */
  .emoji-picker {
    position: absolute; bottom: calc(100% + .3rem); right: .2rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .6rem; padding: .35rem .4rem;
    box-shadow: 0 4px 16px rgba(0,0,0,.12);
    display: flex; flex-wrap: wrap; gap: .2rem; max-width: 240px;
    z-index: 20; animation: pop .12s var(--animation, ease);
  }
  .emoji-picker.picker-mine { right: auto; left: .2rem; }
  .emoji-quick-btn {
    background: none; border: none; font-size: 1.15rem;
    cursor: pointer; padding: .2rem; border-radius: .35rem;
    transition: background .1s; line-height: 1;
  }
  .emoji-quick-btn:hover { background: var(--bg-secondary, #f1f5f9); }
  .emoji-quick-btn.emoji-active {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    border-radius: .35rem;
  }
  .emoji-more-btn {
    background: var(--bg-secondary, #f1f5f9); border: none;
    font-size: .85rem; cursor: pointer; padding: .2rem .4rem;
    border-radius: .35rem; color: var(--text-secondary, #64748b);
    transition: background .1s; font-weight: 600;
  }
  .emoji-more-btn:hover { background: var(--border, #e2e8f0); }
  .emoji-extended {
    width: 100%; flex-wrap: wrap; gap: .2rem;
    border-top: 1px solid var(--border, #e2e8f0);
    padding-top: .3rem; margin-top: .2rem;
  }

  /* Zone édition */
  .edit-zone { display: flex; flex-direction: column; gap: .4rem; }
  .edit-input {
    width: 100%; padding: .4rem .6rem; font-size: .9rem;
    border: 1.5px solid var(--accent, #4ade80); border-radius: .4rem;
    background: var(--bg-primary, #fff); color: var(--text-primary, #1e293b);
    resize: vertical; outline: none; font-family: inherit;
  }
  .edit-actions { display: flex; gap: .4rem; }
  .edit-ok, .edit-cancel {
    padding: .25rem .6rem; border: none; border-radius: .35rem;
    font-size: .8rem; cursor: pointer; transition: background .12s;
  }
  .edit-ok    { background: #dcfce7; color: #166534; }
  .edit-ok:hover    { background: #bbf7d0; }
  .edit-cancel      { background: var(--bg-secondary, #f1f5f9); color: var(--text-primary, #1e293b); }
  .edit-cancel:hover { background: #fee2e2; }

  @keyframes pop { from { opacity:0; transform:translateY(4px); } to { opacity:1; transform:none; } }

  /* ─── Picker emoji natif (remplace GIF Tenor — S39) ─── */
  .emoji-panel {
    flex-shrink: 0; border-top: 1px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    max-height: 260px; display: flex; flex-direction: column;
  }
  .ep-header {
    display: flex; align-items: center; gap: .3rem;
    padding: .4rem .6rem; border-bottom: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
  }
  .ep-cats { display: flex; gap: .2rem; flex: 1; overflow-x: auto; scrollbar-width: none; }
  .ep-cats::-webkit-scrollbar { display: none; }
  .ep-cat-btn {
    flex-shrink: 0; padding: .3rem .4rem; border: none; border-radius: .4rem;
    font-size: 1.1rem; cursor: pointer; background: transparent;
    transition: background .15s; opacity: .6;
  }
  .ep-cat-btn:hover, .ep-cat-btn.active { background: var(--bg-secondary, #f1f5f9); opacity: 1; }
  .ep-close {
    flex-shrink: 0; padding: .3rem .5rem; border: none; border-radius: .4rem;
    cursor: pointer; background: var(--bg-tertiary, #e2e8f0);
    color: var(--text-secondary, #64748b); font-size: .85rem;
  }
  .ep-body { flex: 1; overflow-y: auto; padding: .4rem .5rem; }
  .ep-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(2rem, 1fr));
    gap: .15rem;
  }
  .ep-emoji {
    padding: .25rem; border: none; background: transparent;
    font-size: 1.35rem; cursor: pointer; border-radius: .3rem;
    transition: background .1s, transform .1s; line-height: 1;
  }
  .ep-emoji:hover { background: var(--bg-primary, #fff); transform: scale(1.2); }

  /* ─── Saisie ─── */
  .emoji-only {
    font-size: 4rem !important;
    line-height: 1.2;
    background: transparent !important;
    padding: 0 !important;
    box-shadow: none !important;
  }

  .input-area {
    flex-shrink: 0;
    display: flex; align-items: center; gap: .4rem;
    padding: .7rem 1rem;
    border-top: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
    width: 100%;
  }
  .icon-btn {
    padding: .45rem; background: none; border: none;
    font-size: 1.15rem; cursor: pointer; border-radius: 50%;
    transition: background .15s; flex-shrink: 0;
  }
  .icon-btn:hover { background: var(--bg-secondary, #f1f5f9); }
  .emoji-open-btn {
    font-size: 1.2rem !important;
    transition: transform .15s;
  }
  .emoji-open-btn:hover { transform: scale(1.15); background: var(--bg-secondary, #f1f5f9) !important; }

  /* ─── Onglets Emoji / GIF ─── */
  .ep-tabs {
    display: flex; align-items: center; gap: .2rem;
    padding: .35rem .5rem; border-bottom: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
  }
  .ep-tab {
    padding: .3rem .65rem; border: none; border-radius: .4rem;
    font-size: .82rem; font-weight: 600; cursor: pointer;
    background: transparent; color: var(--text-secondary, #64748b);
    transition: all .15s;
  }
  .ep-tab:hover  { background: var(--bg-secondary, #f1f5f9); color: var(--text-primary); }
  .ep-tab.active { background: var(--accent, #4ade80); color: #fff; }
  .ep-tab-spacer { flex: 1; }

  /* ─── GIFs locaux ─── */
  .gif-body { padding: .4rem .5rem; overflow-y: auto; flex: 1; }
  .gif-cats {
    display: flex; gap: .25rem; flex-wrap: wrap;
    margin-bottom: .5rem;
  }
  .gif-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(85px, 1fr));
    gap: .35rem;
  }
  .gif-thumb {
    border: none; border-radius: .4rem; overflow: hidden;
    cursor: pointer; padding: 0; aspect-ratio: 1;
    background: var(--bg-tertiary, #e2e8f0);
    transition: transform .15s, box-shadow .15s;
  }
  .gif-thumb:hover { transform: scale(1.04); box-shadow: 0 2px 8px rgba(0,0,0,.15); }
  .gif-thumb img   { width: 100%; height: 100%; object-fit: cover; display: block; }
  .gif-loading {
    text-align: center; padding: 1.5rem; color: var(--text-secondary, #94a3b8);
    font-size: .85rem; line-height: 1.6;
  }
  .gif-hint { font-size: .78rem; color: var(--text-muted, #94a3b8); }
  .gif-hint code { font-size: .76rem; background: var(--bg-tertiary); padding: .1rem .3rem; border-radius: .25rem; }
  :global(.chat-gif) { max-width: 600px; max-height: 600px; border-radius: .4rem; display: block; }
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

  /* ─── Messages vocaux ─── */
  .voice-message {
    display: flex; align-items: center; gap: .5rem;
    padding: .2rem 0;
  }
  .voice-audio {
    height: 36px;
    max-width: 220px;
    border-radius: 999px;
    accent-color: var(--accent, #4ade80);
  }
  .voice-video {
    max-width: 240px;
    border-radius: .5rem;
    margin-top: .2rem;
  }

  /* Barre d'enregistrement en cours */
  .recording-bar {
    display: flex; align-items: center; gap: .5rem; flex-wrap: wrap;
    padding: .5rem 1rem;
    background: color-mix(in srgb, var(--accent, #4ade80) 10%, var(--bg-primary, #fff));
    border-top: 1px solid color-mix(in srgb, var(--accent, #4ade80) 30%, transparent);
    font-size: .85rem; flex-shrink: 0;
  }
  .recording-dot {
    width: 10px; height: 10px; border-radius: 50%;
    background: #ef4444; flex-shrink: 0;
    animation: blink 1s ease infinite;
  }
  @keyframes blink { 0%,100% { opacity:1; } 50% { opacity:.3; } }
  .recording-timer { font-weight: 700; color: var(--text-primary, #1e293b); }
  .recording-type { font-size: .78rem; color: var(--text-secondary, #64748b); }
  .recording-limit { font-size: .72rem; color: var(--text-secondary, #94a3b8); }
  .recording-error { font-size: .78rem; color: #dc2626; }
  .recording-actions { display: flex; gap: .35rem; margin-left: auto; }
  .rec-btn {
    padding: .3rem .7rem; border: none; border-radius: .45rem;
    font-size: .82rem; cursor: pointer; font-weight: 600; transition: all .12s;
  }
  .rec-stop   { background: var(--accent, #4ade80); color: #fff; }
  .rec-stop:hover { background: var(--button-hover, #22c55e); }
  .rec-cancel { background: var(--bg-secondary, #f1f5f9); color: #64748b; }
  .rec-cancel:hover { background: #fee2e2; color: #dc2626; }

  /* Bouton micro actif */
  .icon-btn.recording {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    color: #ef4444;
    animation: blink .8s ease infinite;
  }

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
