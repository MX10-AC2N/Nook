<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get as storeGet } from 'svelte/store';
    import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chatStore,
    messagesStore,
    loadMoreMessages,
    loadMessages,
    sendMessage,
    editMessage,
    deleteMessage,
    sendEmoji,
    toggleEmojiPicker,
    formatTimestamp,
    setActiveConv,
    disconnectWs,
    requestNotificationPermission,
    MAX_BYTES_SERVER,
    cancelTransfer,
    triggerDecryptAllIfReady,
    initCryptoListener,
  } from '$lib/chatStore';
  import type { ChatMessage } from '$lib/chatStore';
  import { sanitizeHtml, highlightMentions } from '$lib/sanitize';
  import {
    recordingState,
    startRecording,
    stopRecording,
    cancelRecording,
    formatDuration,
  } from '$lib/mediaStore.svelte.js';
  import Avatar from '$lib/components/Avatar.svelte';
  import 'emoji-picker-element';
  import MissedCalls from '$lib/components/MissedCalls.svelte';
  import MessageSearch from '$lib/components/MessageSearch.svelte';
  import { callStore, callManager } from '$lib/webrtc-calls.svelte.ts';

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

  // ────────────────────────────────────────────
  // Svelte Action for emoji-picker events
  // ────────────────────────────────────────────
  function emojiPickerAction(element: HTMLElement, msgId: string) {
    function handler(e: Event) {
      const emoji = (e as CustomEvent).detail.unicode;
      toggleReaction(msgId, emoji);
      emojiPickerMsgId = null;
    }
    element.addEventListener('emoji-click', handler as EventListener);
    return {
      destroy() {
        element.removeEventListener('emoji-click', handler as EventListener);
      }
    };
  }

  // ────────────────────────────────────────────
  // État principal — messages from store (single source of truth)
  // ────────────────────────────────────────────
  let conversations   = $state<Conv[]>([]);
  // Persist active conversation in localStorage + URL param to survive refresh
  const getStoredConvId = () => {
    if (typeof window !== 'undefined') {
      // 1. Check URL param first (most reliable for refresh)
      const params = new URLSearchParams(window.location.search);
      const urlConvId = params.get('conv');
      if (urlConvId) {
        return urlConvId;
      }
    }
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('nook_activeConvId') || 'default_global';
    }
    return 'default_global';
  };
  let activeConvId    = $state(getStoredConvId());
  // Save to localStorage whenever it changes
  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('nook_activeConvId', activeConvId);
    }
  });

  // Update URL param whenever activeConvId changes (for refresh persistence)
  $effect(() => {
    if (typeof window !== 'undefined' && activeConvId) {
      const params = new URLSearchParams(window.location.search);
      const current = params.get('conv');
      if (current !== activeConvId) {
        params.set('conv', activeConvId);
        const newUrl = `${window.location.pathname}?${params.toString()}`;
        window.history.replaceState({}, '', newUrl);
      }
    }
  });

  // Read messages directly from the writable store
  let localMessages = $state<ChatMessage[]>([]);
  let reversedMessages = $derived(localMessages.slice().reverse());
  // Sync store → local state for reactivity
  // Use a promise-based bridge so callers can await the update after set()
  let _messagesResolve: (() => void) | null = null;
  $effect(() => {
    const unsub = messagesStore.subscribe(msgs => {
      localMessages = [...msgs];
      _messagesResolve?.();
      _messagesResolve = null;
    });
    return unsub;
  });

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

  // ─── Mention autocomplete ─────────────────────────────────────────
  let mentionQuery    = $state('');
  let mentionStart    = $state(-1);
  let showMentions    = $derived(mentionStart >= 0);
  let filteredMentions = $derived(
    availableUsers.filter(u => {
      const q = mentionQuery.toLowerCase();
      return (u.username?.toLowerCase().includes(q) || u.name?.toLowerCase().includes(q));
    }).slice(0, 5)
  );

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
  
  // Transferts P2P en cours
  interface P2PTransfer {
    fileName: string;
    fileSize: number;
    progress: number;
    speed: number;
    status: 'encrypting' | 'sending' | 'completed' | 'error';
    error?: string;
  }
  let p2pTransfers = $state<Map<string, P2PTransfer>>(new Map());

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

  // ── P2P Helper: format error messages ──────────────────────────────
  function getP2PErrorMessage(error: any): string {
    if (!error) return 'Erreur inconnue';
    
    const msg = error instanceof Error ? error.message : String(error);
    
    // Timeout
    if (msg.includes('Timeout') || msg.includes('timeout')) {
      return "Délai d'attente dépassé (10s). L'utilisateur est peut-être hors ligne.";
    }
    
    // WebSocket errors
    if (msg.includes('WebSocket') || msg.includes('ws')) {
      return 'Connexion au serveur perdue. Vérifiez votre connexion.';
    }
    
    // DataChannel errors
    if (msg.includes('DataChannel') || msg.includes('channel')) {
      return 'Canal de données impossible à établir.';
    }
    
    // ICE/STUN/TURN errors
    if (msg.includes('ICE') || msg.includes('STUN') || msg.includes('TURN')) {
      return 'Erreur de configuration réseau (ICE/STUN/TURN).';
    }
    
    // Generic WebRTC error
    if (msg.includes('RTCPeerConnection') || msg.includes('WebRTC')) {
      return 'Erreur de connexion WebRTC.';
    }
    
    return msg || 'Erreur inconnue';
  }
  // ── P2P Helper: calculate remaining seconds ───────────────
  function getRemainingSeconds(transfer: any): number {
    if (!transfer || transfer.speed <= 0) return 0;
    const remainingBytes = transfer.fileSize * (100 - transfer.progress) / 100;
    return Math.max(1, Math.round(remainingBytes / (transfer.speed * 1024)));
  }


  // ── P2P Helper: play notification sound ──────────────────────
  function playNotificationSound(type: 'success' | 'error' = 'success') {
    try {
      const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioCtx.createOscillator();
      const gainNode = audioCtx.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioCtx.destination);
      
      if (type === 'success') {
        // Success: short ascending beep
        oscillator.frequency.setValueAtTime(800, audioCtx.currentTime);
        oscillator.frequency.setValueAtTime(1200, audioCtx.currentTime + 0.1);
      } else {
        // Error: short descending beep
        oscillator.frequency.setValueAtTime(600, audioCtx.currentTime);
        oscillator.frequency.setValueAtTime(400, audioCtx.currentTime + 0.1);
      }
      
      gainNode.gain.setValueAtTime(0.3, audioCtx.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioCtx.currentTime + 0.2);
      
      oscillator.start(audioCtx.currentTime);
      oscillator.stop(audioCtx.currentTime + 0.2);
      
      setTimeout(() => audioCtx.close(), 300);
    } catch (e) {
      console.warn('[P2P] Sound notification failed:', e);
    }
  }

  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let typingUsers = $state<string[]>([]);
  let typingTimeout: ReturnType<typeof setTimeout> | null = null;
  let sidebarOpen = $state(false);

  // État édition de message
  let editingMsgId   = $state<string | null>(null);
  let editingContent = $state('');
  // Menu contextuel (hover)
  let hoveredMsgId   = $state<string | null>(null);

  // ─────────────────────────────────────────────────────────────────
  // Réactions aux messages
  // ─────────────────────────────────────────────────────────────────
  

  // picker étendu ouvert pour quel message
  let reactions = $state<Record<string, { counts: Record<string, string[]>; myEmoji: string | null }>>({});
  const QUICK_EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '😡'] as const;

  // picker étendu ouvert pour quel message
  let emojiPickerMsgId = $state<string | null>(null);
  let extendedEmojiMsgId = $state<string | null>(null);  // ← NOUVEAU : zone étendue
  let messageMenuMsgId = $state<string | null>(null);   // menu contextuel (éditer/supprimer)
  let emojiPickerPos = $state<{ top: number; left: number; right: number }>({ top: 0, left: 0, right: 0 });

  function openMsgEmojiPicker(msgId: string, targetEl?: HTMLElement) {
    extendedEmojiMsgId = msgId;
    emojiPickerMsgId = null;
    // Calculer la position du picker par rapport au target (fixed → coordonnées viewport)
    if (targetEl) {
      const rect = targetEl.getBoundingClientRect();
      const pickerHeight = 360; // max-height from CSS
      const spaceBelow = window.innerHeight - rect.bottom;
      const spaceAbove = rect.top;
      const openBelow = spaceBelow >= pickerHeight + 10; // 10px margin
      
      emojiPickerPos = {
        top: openBelow ? rect.bottom + 6 : rect.top - pickerHeight - 6,
        left: rect.left,
        right: window.innerWidth - rect.right,
      };
    }
  }
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
  // tous les emojis disponibles (picker étendu - 80+ emojis courants)
  const ALL_EMOJIS = [
    // Visages souriants
    '😀','😃','😄','😁','😆','😅','🤣','😂','🙂','🙃',
    '😉','😊','😇','🥰','😍','🤩','😘','😗','😚','😙',
    '🥲','😋','😛','😜','🤪','😝','🤑','🤗','🤭','🤫',
    '🤔','🤐','🤨','😐','😑','😶','😏','😒','🙄','😬',
    '🙁','😌','😔','😪','🤤','😴','😷','🤒','🤕','🤢',
    // Cœurs & amour
    '❤️','🧡','💛','💚','💙','💜','🖤','🤍','💔','❣️',
    '💕','💞','💓','💗','💖','💘','💝','💟',
    // Mains & gestes
    '👍','👎','👌','🤌','🤞','🤟','🤘','🤙','👈','👉',
    '👆','🖕','👇','☝️','👋','🤚','🖐','🖖','✋','🤛',
    '🤜','🤝','🙏','✌️','🤞','🤟','🤘','🤙','💪','🦾',
    // Animaux
    '🐶','🐱','🐭','🐹','🐰','🦊','🐻','🐼','🐨','🐯',
    '🦁','🐮','🐷','🐸','🐵','🙈','🙉','🙊','🐒','🐔',
    // Nourriture
    '🍎','🍐','🍊','🍋','🍌','🍉','🍇','🍓','🫐','🍈',
    '🍒','🍑','🥭','🍍','🥥','🥝','🍅','🥑','🍆','🥦',
    // Activités & objets
    '⚽','🏀','🏈','⚾','🥎','🎾','🎳','🏏','🏹','🎣',
    '🤿','🎯','🪀','🪁','🔮','🎨','🎭','🎪','🎫','🎬',
    // Symboles & drapeaux (quelques-uns)
    '🎉','🎊','🎋','🎍','🎎','🎏','🎐','🎑','🎆','🎇',
    '🏁','🚩','🏴','🏳️','🏳️🌈','🏴☠️','🇫🇷','🇨🇦','🇫🇷','🇺🇸',
    // Emojis étendus (ZWJ, familles, professions, drapeaux modernes)
    '🧑‍🚀','🧑‍🎓','🧑‍🎤','🧑‍🏫','🧑‍⚕️','🧑‍⚖️','🧑‍🌾','🧑‍🍳','🧑‍🔧','🧑‍🏭',
    '👨‍👩‍👦','👨‍👩‍👧','👨‍👩‍👧‍👦','👩‍👩‍👦','👩‍👩‍👧','👩‍👩‍👧‍👦','👨‍👨‍👦','👨‍👨‍👧','👨‍👨‍👧‍👦',
    '👩‍👦','👩‍👧','👨‍👦','👨‍👧',
    '🏳️‍⚧️','🏳️‍🌈','🇬🇧','🇪🇺','🇪🇸','🇮🇹','🇬🇷','🇵🇹','🇷🇺','🇯🇵',
    '🧑‍🎄','🧑‍🎅','🎅','🤶','🦌','🎄','🎁','🔔','⛄','❄️',
  ];

  // EXTENDED_EMOJIS removed for debugging

  /** Détecte si un message est un unique emoji (affichage agrandi 2.5rem) */
  /** Détecte si un message ne contient QUE des emojis (affichage agrandi) */
  function isEmojiOnly(content: string): boolean {
    if (!content) return false;
    const t = content.trim();
    if (t.length === 0 || t.length > 30) return false;
    // Simple approach: check if all non-whitespace chars are emoji (codepoint > 0x2300)
    // This catches 👋, 🤜, 🤛, 🎉, etc. while excluding regular text
    for (const ch of t) {
      const code = ch.codePointAt(0);
      if (!code) return false;
      // Skip zero-width joiners and variation selectors
      if (code === 0x200D || code === 0xFE0F || code === 0x200B) continue;
      // Emoji range checks
      if (code < 0x2300) return false; // Below emoji ranges
      // Allow known emoji blocks
      if (code >= 0x2600 && code <= 0x27BF) continue; // Misc symbols, dingbats
      if (code >= 0x2300 && code <= 0x23FF) continue; // Misc technical
      if (code >= 0x2B50 && code <= 0x2B59) continue; // Stars, shapes
      if (code >= 0x1F000 && code <= 0x1FFFF) continue; // Emoticons, symbols
      if (code >= 0x2700 && code <= 0x27BF) continue; // Dingbats
      if (code >= 0xFE00 && code <= 0xFE0F) continue; // Variation selectors
      if (code >= 0x1F900 && code <= 0x1F9FF) continue; // Supplemental symbols
      if (code >= 0x1FA00 && code <= 0x1FA6F) continue; // Chess symbols
      if (code >= 0x1FA70 && code <= 0x1FAFF) continue; // Symbols extended
      // If we get here, it's not an emoji
      return false;
    }
    return true;
  }

  async function toggleReaction(msgId: string, emoji: string) {
    const cur = reactions[msgId];
    const isMyEmoji = cur?.myEmoji === emoji;

    console.log('[toggleReaction] Called with:', { msgId, emoji, isMyEmoji, cur, activeConvId });

    try {
      const convId = activeConvId;
      if (!convId) {
        console.error('[toggleReaction] Aucune conversation active');
        return;
      }
      let res: Response;
      if (isMyEmoji) {
        res = await fetch(`/api/conversations/${convId}/messages/${msgId}/reactions`, {
          method: 'DELETE', credentials: 'include',
        });
      } else {
        console.log('[toggleReaction] Adding reaction', emoji);
        res = await fetch(`/api/conversations/${convId}/messages/${msgId}/reactions`, {
          method: 'POST', credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ emoji }),
        });
      }
      console.log('[toggleReaction] Response:', { status: res.status, ok: res.ok, statusText: res.statusText });
      if (res.ok) {
        const data = await res.json();
        console.log('[toggleReaction] Data:', data);
        const updated = { counts: data.counts ?? {}, myEmoji: data.my_emoji ?? null };
        // Use spread to trigger Svelte 5 reactivity reliably
        reactions = { ...reactions, [msgId]: updated };
        console.log('[toggleReaction] Updated reactions:', reactions[msgId]);
      } else {
        const text = await res.text().catch(() => '');
        console.error('[toggleReaction] API error:', res.status, text);
      }
    } catch (e) {
      console.error('[Reaction]', e);
    }
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

    // Fermer le sidebar mobile d'abord
    sidebarOpen = false;
    
    // Activer la conv : connecte le WS, reset badge non-lus
    setActiveConv(conv.id);
    // Load messages (single call) — retourne les messages chargés pour éviter la race condition
    const msgs = await loadMessages(conv.id);
    await loadReactionsForMessages(conv.id, msgs);
    // Scroll to top après chargement — nouveaux messages en haut
    await Promise.resolve();
    if (chatContainer) chatContainer.scrollTop = 0;
    // Fallback polling si WS non disponible
    if (pollTimer) clearInterval(pollTimer);
    if (!chatStore.wsConnected) {
      pollTimer = setInterval(() => loadMessages(conv.id), 8000);
    }
  }

  async function loadReactionsForMessages(convId: string, msgs?: ChatMessage[]) {
    // Si msgs est fourni, l'utiliser directement (évite race condition après selectConversation)
    const toLoad = (msgs ?? localMessages.slice(-50));
    await Promise.allSettled(toLoad.map(async (msg) => {
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
    
    // Reset textarea height
    const textarea = document.querySelector('.message-input') as HTMLTextAreaElement;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = '44px';
    }

    try {
      await sendMessage(content, activeConvId);
      // Scroller vers le haut pour voir le nouveau message
      if (chatContainer) chatContainer.scrollTop = 0;
    } catch (e) {
      console.error('[Chat] send error:', e);
    } finally {
      sending = false;
    }
  }

  function handleTyping() {
    // Auto-resize textarea
    const textarea = document.querySelector('.message-input') as HTMLTextAreaElement;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = Math.min(textarea.scrollHeight, 160) + 'px';
    }
    // Mention autocomplete: detect @ in the message
    const cursor = (document.querySelector('.message-input') as HTMLTextAreaElement)?.selectionStart ?? newMessage.length;
    const beforeCursor = newMessage.slice(0, cursor);
    const atMatch = beforeCursor.match(/@(\w*)$/);
    if (atMatch) {
      mentionStart = cursor - atMatch[0].length;
      mentionQuery = atMatch[1];
    } else {
      mentionStart = -1;
      mentionQuery = '';
    }

    // Send typing indicator via WebSocket
    if (chatStore.ws && chatStore.ws.readyState === WebSocket.OPEN) {
      chatStore.ws.send(JSON.stringify({
        type: 'typing',
        conversation_id: activeConvId,
      }));
    }
    
    // Clear previous timeout
    if (typingTimeout) clearTimeout(typingTimeout);
    
    // Stop typing after 3 seconds of inactivity
    typingTimeout = setTimeout(() => {
      if (chatStore.ws && chatStore.ws.readyState === WebSocket.OPEN) {
        chatStore.ws.send(JSON.stringify({
          type: 'stop_typing',
          conversation_id: activeConvId,
        }));
      }
    }, 3000);
  }

  function selectMention(username: string) {
    if (mentionStart < 0) return;
    const before = newMessage.slice(0, mentionStart);
    const after = newMessage.slice(mentionStart + mentionQuery.length + 1); // +1 for @
    newMessage = before + '@' + username + ' ' + after;
    mentionStart = -1;
    mentionQuery = '';
    // Focus back on input
    const input = document.querySelector('.message-input') as HTMLTextAreaElement;
    input?.focus();
    // Trigger resize after mention insertion
    if (input) {
      input.style.height = 'auto';
      input.style.height = Math.min(input.scrollHeight, 160) + 'px';
    }
  }

  function handleMessageKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSendMessage(); }
  }

  function handleSubmit(e: Event) { e.preventDefault(); handleSendMessage(); }

  function handleSelectEmoji(emoji: string) {
    // Toujours ajouter l'emoji au champ de saisie
    // L'utilisateur peut empiler plusieurs emojis d'affilée puis envoyer
    newMessage = newMessage + emoji;
    // Trigger textarea resize
    const textarea = document.querySelector('.message-input') as HTMLTextAreaElement;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = Math.min(textarea.scrollHeight, 160) + 'px';
    }
    // Ne pas fermer le picker → permet de sélectionner plusieurs emojis d'affilée
  }

  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    const file = input.files[0];
    // Vérifier si on a une connexion P2P active
    const hasP2PConnection = callStore.isInCall && callStore.fileDataChannels.size > 0;
    
    // Vérifier si la conversation est un groupe (P2P interdit en groupe)
    const isGroup = activeConv?.is_group === true;
    // Logique de routage

    // Logique de routage
    
    // Vérifier si on peut faire un transfert P2P
    const canDoP2P = !isGroup && (hasP2PConnection || file.size > MAX_BYTES_SERVER);
    
    if (file.size > MAX_BYTES_SERVER && isGroup) {
      chatStore.connectionError = `Fichier > 50 Mo impossible en groupe. Utilisez un fichier plus petit ou envoyez en privé.`;
      input.value = '';
      setTimeout(() => chatStore.connectionError = null, 5000);
      return;
    }
    
    if (file.size > MAX_BYTES_SERVER && !canDoP2P) {
      // Fichier > 50 Mo mais pas de connexion P2P disponible
      chatStore.connectionError = `Fichier > 50 Mo nécessite une conversation 1-à-1.`;
      input.value = '';
      setTimeout(() => chatStore.connectionError = null, 5000);
      return;
    }
    
    // Choix de la méthode de transfert
    if (file.size > MAX_BYTES_SERVER || hasP2PConnection) {
      // Transfert P2P via DataChannel
      await handleP2PFileTransfer(file, input);
    } else {
      // Upload serveur classique
      await handleServerFileUpload(file, input);
    }
  }
  
  // Upload serveur classique (<= 50 Mo)
  async function handleServerFileUpload(file: File, input: HTMLInputElement) {
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
      const isAudio = data.is_audio ?? file.type.startsWith('audio/');
      const isVideo = data.is_video ?? file.type.startsWith('video/');
      let uploadContent: string;
      if (isImage) {
        uploadContent = `<div class="file-preview"><img src="/api/download/${data.file_id}" alt="${data.file_name}" class="uploaded-image" /><a href="/api/download/${data.file_id}" download="${data.file_name}" class="file-download" title="Télécharger">⬇️</a></div>`;
      } else if (isAudio) {
        uploadContent = `<div class="file-audio"><audio src="/api/download/${data.file_id}" controls preload="none" class="chat-audio"></audio><a href="/api/download/${data.file_id}" download="${data.file_name}" class="file-download" title="Télécharger">⬇️</a></div>`;
      } else if (isVideo) {
        uploadContent = `<div class="file-video"><video src="/api/download/${data.file_id}" controls preload="none" class="chat-video"></video><a href="/api/download/${data.file_id}" download="${data.file_name}" class="file-download" title="Télécharger">⬇️</a></div>`;
      } else {
        uploadContent = `<span class="file-attachment">📎 <a href="/api/download/${data.file_id}" download="${data.file_name}">${data.file_name}</a></span>`;
      }
      await sendMessage(uploadContent, activeConvId);
      input.value = '';
    } catch (err: unknown) {
      console.error('[Upload]', err);
      chatStore.connectionError = err instanceof Error ? err.message : "Échec de l'upload";
      setTimeout(() => chatStore.connectionError = null, 5000);
    }
  }
  
  // Transfert P2P via DataChannel
  async function handleP2PFileTransfer(file: File, input: HTMLInputElement) {
    const { sendFile } = await import('$lib/file-transfer.svelte.ts');
    
    // Trouver un DataChannel disponible
    let channel: RTCDataChannel | null = null;
    let targetUserId: string | null = null;
    
    for (const [userId, ch] of callStore.fileDataChannels.entries()) {
      if (ch.readyState === 'open') {
        channel = ch;
        targetUserId = userId;
        break;
      }
    }
    
    // Si aucun canal dispo → essayer de créer avec retry
    if (!channel || !targetUserId) {
      const participants = participantsCache[activeConvId];
      if (!participants || participants.length === 0) {
        chatStore.connectionError = 'Aucun participant trouvé dans la conversation.';
        input.value = '';
        setTimeout(() => chatStore.connectionError = null, 5000);
        return;
      }
      targetUserId = participants.find(p => p.id !== authStore.user?.id)?.id;
    }
    
    if (!targetUserId) {
        chatStore.connectionError = "Impossible de déterminer l'utilisateur cible.";
      input.value = '';
      setTimeout(() => chatStore.connectionError = null, 5000);
      return;
    }
    
    // Afficher l'UI de progression
    const progressId = `p2p_${Date.now()}`;
    p2pTransfers.set(progressId, {
      fileName: file.name,
      fileSize: file.size,
      progress: 0,
      speed: 0,
      status: 'connecting'
    });
    p2pTransfers = new Map(p2pTransfers);
    
    try {
      chatStore.connectionError = `🔄 Établissement de la connexion P2P vers ${targetUserId}...`;
      
      if (!channel || channel.readyState !== 'open') {
        const maxRetries = 2;
        let lastError: any = null;
        
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
          try {
            chatStore.connectionError = `🔄 Connexion P2P (tentative ${attempt}/${maxRetries})...`;
            channel = await callManager.createFileTransferConnection(targetUserId);
            chatStore.connectionError = null;
            lastError = null;
            break;
          } catch (error) {
            lastError = error;
            console.warn(`[P2P] Tentative ${attempt}/${maxRetries} échouée:`, error);
            if (attempt < maxRetries) {
              await new Promise(resolve => setTimeout(resolve, 2000));
            }
          }
        }
        
        if (lastError) {
          const transfer = p2pTransfers.get(progressId);
          if (transfer) {
            transfer.status = 'error';
            transfer.error = getP2PErrorMessage(lastError);
            p2pTransfers = new Map(p2pTransfers);
          }
          const errorMsg = getP2PErrorMessage(lastError);
          chatStore.connectionError = `Impossible d'établir la connexion P2P: ${errorMsg}`;
          input.value = '';
          setTimeout(() => chatStore.connectionError = null, 5000);
          return;
        }
      }
      
      // Appel sendFile avec les 6 paramètres requis
      sendFile(
        file,
        channel,
        activeConvId,
        (pct, speed) => {
          const transfer = p2pTransfers.get(progressId);
          if (transfer) {
            transfer.progress = pct;
            transfer.speed = speed;
            transfer.status = pct < 30 ? 'encrypting' : 'sending';
            p2pTransfers = new Map(p2pTransfers);
          }
        },
        async (fileId) => {
          const transfer = p2pTransfers.get(progressId);
          if (transfer) {
            transfer.status = 'completed';
            transfer.progress = 100;
            p2pTransfers = new Map(p2pTransfers);
            playNotificationSound('success');
          }
          
          const content = `<span class="file-p2p">📁 <strong>${file.name}</strong> (${(file.size / 1024 / 1024).toFixed(1)} Mo) — transféré en P2P</span>`;
          await sendMessage(content, activeConvId);
          
          setTimeout(() => {
            p2pTransfers.delete(progressId);
            p2pTransfers = new Map(p2pTransfers);
          }, 3000);
          
          input.value = '';
        },
        (error) => {
          const transfer = p2pTransfers.get(progressId);
          if (transfer) {
            transfer.status = 'error';
            transfer.error = error;
            p2pTransfers = new Map(p2pTransfers);
            playNotificationSound('error');
          }
          
          chatStore.connectionError = `Échec du transfert P2P: ${error}`;
          setTimeout(() => chatStore.connectionError = null, 5000);
        }
      );
      
    } catch (err) {
      console.error('[P2P Transfer]', err);
      p2pTransfers.delete(progressId);
      p2pTransfers = new Map(p2pTransfers);
      playNotificationSound('error');
      chatStore.connectionError = err instanceof Error ? err.message : "Échec du transfert P2P";
      input.value = '';
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
      // Démarrage — getUserMedia requis
      if (!navigator.mediaDevices?.getUserMedia) {
        const isStandalone = window.matchMedia('(display-mode: standalone)').matches;
        if (location.protocol === 'http:' && location.hostname !== 'localhost') {
          chatStore.connectionError = `Accédez via HTTPS pour l'audio: https://${location.hostname}:6443`;
        } else {
          chatStore.connectionError = 'Enregistrement vocal non disponible';
        }
        setTimeout(() => chatStore.connectionError = null, 6000);
        return;
      }
      try {
        await startRecording(mediaType);
      } catch (err: unknown) {
        console.error('[VoiceRecord start]', err);
        chatStore.connectionError = err instanceof Error ? err.message : 'Erreur microphone';
        setTimeout(() => chatStore.connectionError = null, 5000);
      }
    }
  }

  function isMyMessage(senderId: string) { 
    return authStore.user?.id === senderId || senderId === 'Moi' || !senderId;
  }

  function startEdit(msg: { id: string; content: string }) {
    console.log('[Edit] startEdit called for msg:', msg.id, 'content preview:', msg.content.substring(0, 50));
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

  /** Pagination — déclenché au scroll en bas du conteneur (messages anciens) */
  async function handleMessagesScroll(e: Event) {
    const el = e.target as HTMLElement;
    const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (distFromBottom < 80 && chatStore.hasMore && !chatStore.loadingMore) {
      const prevHeight = el.scrollHeight;
      await loadMoreMessages(activeConvId);
      // Maintenir la position de scroll après insertion en bas
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollTop + (el.scrollHeight - prevHeight);
      });
    }
  }

  // ─────────────────────────────────────────────────────────────────
  // Cycle de vie
  // ─────────────────────────────────────────────────────────────────



  onMount(async () => {
    // Wait for authStore to finish loading before checking auth state
    while (authStore.loading) { await new Promise(r => setTimeout(r, 50)); }
    if (!authStore.isAuthenticated) return; // Le layout gérera le redirect
    await loadConversations();
    
    // FIX refresh: read conversationId from URL parameter
    const urlConvId = $page.params.conversationId;
    if (urlConvId && urlConvId !== 'default_global') {
      const targetConv = conversations.find(c => c.id === urlConvId);
      if (targetConv) {
        await selectConversation(targetConv);
        return; // selectConversation already loads messages
      }
    }
    
    // FIX refresh: if no URL param, try localStorage or keep current activeConvId
    const storedConvId = typeof localStorage !== 'undefined' ? localStorage.getItem('nook_activeConvId') : null;
    if (storedConvId && conversations.some(c => c.id === storedConvId)) {
      activeConvId = storedConvId;
      // Restore activeConv & activeConvName for UI banner
      const restoredConv = conversations.find(c => c.id === storedConvId);
      if (restoredConv) {
        activeConv = restoredConv;
        if (restoredConv.id === 'default_global') {
          activeConvName = '🌿 Nook';
        } else if (restoredConv.is_group) {
          activeConvName = restoredConv.name ?? 'Groupe sans nom';
        } else {
          const parts = participantsCache[restoredConv.id] ?? [];
          const other = parts.find(p => p.id !== authStore.user?.id);
          activeConvName = other ? (other.name ?? other.username) : (restoredConv.name ?? 'Message direct');
        }
      }
    }

    await loadMessages(activeConvId);
    await loadReactionsForMessages(activeConvId);
    // Démarrer le listener crypto pour le déchiffrement automatique des messages existants
    initCryptoListener();
    setActiveConv(activeConvId);
    // Scroll to top after initial load — newest messages are at top
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (chatContainer) {
          chatContainer.scrollTop = 0;
        }
      });
    });
    // Demande permission notifications (non-bloquant)
    requestNotificationPermission();
    // Charger les utilisateurs disponibles pour l'autocomplete @mentions
    await loadAvailableUsers();
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

  // DEBUG: Update decryption status in UI
  $effect(() => {
    const encryptedCount = localMessages.filter(m => m.encrypted).length;
    const totalCount = localMessages.length;
    const el = document.getElementById('debug-decrypt-status');
    if (el) {
      el.textContent = `Total: ${totalCount} | Encrypted: ${encryptedCount} | ${encryptedCount === 0 && totalCount > 0 ? '✅ All decrypted' : encryptedCount > 0 ? '⏳ Decrypting...' : '📭 No messages'}`;
    }
  });

  // DEBUG: Manual retry decrypt
  if (typeof window !== 'undefined') {
    window.addEventListener('debug-retry-decrypt', async () => {
      const { loadMessages, _decryptAllIfReady } = await import('$lib/chatStore.svelte.ts');
      await _decryptAllIfReady();
    });
  }

  let initialScrollDone = $state(false);

  $effect(() => {
    const count = localMessages.length;
    if (!chatContainer || count === 0) return;
    const el = chatContainer;
    // Force scroll to top on first render (newest messages at top)
    if (!initialScrollDone) {
      initialScrollDone = true;
      Promise.resolve().then(() => {
        if (chatContainer) chatContainer.scrollTop = 0;
      });
      return;
    }
    // Auto-scroll to top if user is near top (viewing recent messages)
    const isNearTop = el.scrollTop < 150;
    if (isNearTop) {
      Promise.resolve().then(() => {
        if (chatContainer) chatContainer.scrollTop = 0;
      });
    }
  });

  // Rafraîchir la réaction d'un seul message à la réception du signal WS
  // Handle typing events from WebSocket
  $effect(() => {
    if (chatStore.ws) {
      const handler = (event: MessageEvent) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.type === 'typing' && msg.conversation_id === activeConvId) {
            if (!typingUsers.includes(msg.user_id)) {
              typingUsers = [...typingUsers, msg.user_id];
            }
            // Auto-remove after 5 seconds
            setTimeout(() => {
              typingUsers = typingUsers.filter(u => u !== msg.user_id);
            }, 5000);
          } else if (msg.type === 'stop_typing' && msg.conversation_id === activeConvId) {
            typingUsers = typingUsers.filter(u => u !== msg.user_id);
          }
        } catch {}
      };
      chatStore.ws.addEventListener('message', handler);
      return () => chatStore.ws?.removeEventListener('message', handler);
    }
  });

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
  <aside class="conversations-sidebar" class:open={sidebarOpen} role="navigation" aria-label="Conversations">
    <div class="sidebar-header">
      <h2>Conversations</h2>
      <button class="btn-new-conv" onclick={openNewConv} title="Nouvelle conversation">＋</button>
    </div>

    <!-- Appels manqués -->
    <MissedCalls />

    <!-- Recherche de messages -->
    <MessageSearch />

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
              <span class="preview">
                {#if conv.is_group}
                  Groupe
                {:else}
                  Message privé
                {/if}
              </span>
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

  <!-- Sidebar backdrop (mobile) -->
  <div class="sidebar-backdrop" class:visible={sidebarOpen} onclick={() => sidebarOpen = false} aria-hidden="true"></div>

  <!-- ─── ZONE CHAT ─── -->
  <main class="chat-area">

    <header class="chat-header">
      <button class="btn-menu-mobile" onclick={() => sidebarOpen = !sidebarOpen} aria-label="Menu" aria-expanded={sidebarOpen}>
        ☰
      </button>
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
        
        <!-- Transferts P2P en cours -->
        {#if p2pTransfers.size > 0}
          <div class="p2p-transfers">
            {#each [...p2pTransfers.entries()] as [id, transfer]}
              <div class="p2p-transfer" class:completed={transfer.status === 'completed'} class:error={transfer.status === 'error'}>
                <div class="p2p-transfer-info">
                  <span class="p2p-transfer-name">{transfer.fileName}</span>
                  <span class="p2p-transfer-size">{(transfer.fileSize / 1024 / 1024).toFixed(1)} Mo</span>
                </div>
                <div class="p2p-transfer-progress">
                  <div class="p2p-progress-bar">
                    <div class="p2p-progress-fill" style="width: {transfer.progress}%"></div>
                  </div>
                  <div class="p2p-transfer-stats">
                    {#if transfer.status === 'connecting'}
                      <span>🔄 Connexion P2P en cours...</span>
                    {:else if transfer.status === 'encrypting'}
                      <span>🔐 Chiffrement...</span>
                    {:else if transfer.status === 'sending'}
                      <span>📤 Envoi... {transfer.speed.toFixed(0)} KB/s</span>
                      {#if transfer.speed > 0}
                        <span class="time-remaining">
                          {getRemainingSeconds(transfer)}s restantes
                        </span>
                      {/if}
                    {:else if transfer.status === 'completed'}
                      <span>✅ Terminé</span>
                    {:else if transfer.status === 'error'}
                      <span>❌ {transfer.error}</span>
                    {/if}
                    <span>{transfer.progress.toFixed(0)}%</span>
                    {#if transfer.status !== 'completed' && transfer.status !== 'error'}
                      <button class="p2p-cancel-btn" onclick={() => cancelTransfer(id)} title="Annuler">✕</button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
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

  <div class="messages-container" bind:this={chatContainer} onscroll={handleMessagesScroll} onclick={() => { if (emojiPickerMsgId) emojiPickerMsgId = null; }} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); if (emojiPickerMsgId) emojiPickerMsgId = null; }}}>
      {#if localMessages.length === 0}
        {#if loadingConvs}
          <div class="empty-state">
            <span class="loading-dots">···</span>
          </div>
        {:else}
          <div class="empty-state">
            <span class="empty-icon">💬</span>
            <p>Aucun message — soyez le premier à écrire !</p>
          </div>
        {/if}
      {/if}

      {#each reversedMessages as msg (msg.id)}
        <div class="message-wrapper" role="presentation"
             onmouseenter={() => { hoveredMsgId = msg.id; }}
             onmouseleave={() => { if (emojiPickerMsgId !== msg.id && extendedEmojiMsgId !== msg.id) hoveredMsgId = null; }}
             class:is-emoji-only={isEmojiOnly(msg.content)}
             class:mine={msg.sender_id === authStore.user?.id}
        >
          <!-- Sender avatar (for other users' messages) -->
          {#if msg.sender_id !== authStore.user?.id}
            <div class="message-avatar">
              <Avatar
                username={msg.sender_name}
                name={msg.sender_name}
                size={28}
                userId={msg.sender_id}
                style={msg.sender_avatar_style ?? ''}
                seed={msg.sender_avatar_seed ?? ''}
              />
            </div>
          {/if}

          <!-- Message column: bubble + reactions stacked vertically -->
          <div class="message-column">
            <!-- Message content -->
            <div class="message {msg.sender_id === authStore.user?.id ? 'mine' : 'theirs'}">
              {#if msg.encrypted}
                <div class="encrypted-placeholder">🔒 Message chiffré (clé indisponible)</div>
              {:else if isEmojiOnly(msg.content)}
                <div class="emoji-only">{@html msg.content}</div>
              {:else}
                {@html msg.content}
              {/if}
            <!-- Message editing UI (when this message is being edited) -->
            {#if editingMsgId === msg.id}
              <div class="message-edit-input">
                <input
                  type="text"
                  class="edit-input"
                  bind:value={editingContent}
                  onkeydown={handleEditKeydown}
                  onblur={submitEdit}
                  autofocus
                />
              </div>
            {/if}
            </div>

            <!-- Message reactions (below the bubble) -->
            {#if countReactions(msg.id).length > 0}
              <div class="message-reactions">
                {#each countReactions(msg.id) as reaction}
                  <button
                    class="reaction-badge"
                    class:my-reaction={reactions[msg.id]?.myEmoji === reaction.emoji}
                    onclick={() => toggleReaction(msg.id, reaction.emoji)}
                    title="{reaction.names}"
                  >
                    <span class="reaction-emoji">{reaction.emoji}</span>
                    <span class="reaction-count">{reaction.count}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Message actions (visible on hover) -->
          {#if hoveredMsgId === msg.id || emojiPickerMsgId === msg.id || extendedEmojiMsgId === msg.id}
            <div class="message-actions">
              <!-- Quick reactions -->
              {#each QUICK_EMOJIS as emoji}
                <button class="quick-react-btn" onclick={() => toggleReaction(msg.id, emoji)} title="Réagir avec {emoji}">
                  {emoji}
                </button>
              {/each}
              <!-- More reactions button -->
              <button class="action-btn react-more" onclick={(e: MouseEvent) => openMsgEmojiPicker(msg.id, e.currentTarget as HTMLElement)} title="Plus d'emojis">
                😊+
              </button>
              <!-- Message menu (...) — only for own messages -->
              {#if isMyMessage(msg.sender_id)}
                <button class="action-btn msg-menu-toggle" onclick={() => messageMenuMsgId = (messageMenuMsgId === msg.id ? null : msg.id)} title="Message options" class:active={messageMenuMsgId === msg.id}>⋯</button>
                {#if messageMenuMsgId === msg.id}
                  <div class="message-menu-dropdown">
                    <button class="msg-menu-item" onclick={() => { startEdit(msg); messageMenuMsgId = null; }}>✏️ Éditer</button>
                    <button class="msg-menu-item delete" onclick={() => { confirmDelete(msg.id); messageMenuMsgId = null; }}>🗑️ Supprimer</button>
                  </div>
                {/if}
              {/if}
            </div>
          {/if}

          <!-- Extended emoji picker for this message (uses emoji-picker-element) -->
          {#if emojiPickerMsgId === msg.id}
            <emoji-picker
              use:emojiPickerAction={msg.id}
              class="msg-emoji-picker"
              data-emojis-per-row="8"
              style="top: {emojiPickerPos.top}px; left: {emojiPickerPos.left}px;"
            ></emoji-picker>
            <button class="ep-close-sm" onclick={() => emojiPickerMsgId = null}>✕</button>
          {/if}

          <!-- Extended emoji picker (ALL_EMOJIS grid) -->
          {#if extendedEmojiMsgId === msg.id}
            <div class="extended-emoji-picker" style="top: {emojiPickerPos.top}px; left: {emojiPickerPos.left}px;">
              <div class="extended-emoji-header">
                <span>Plus d'emojis</span>
                <button class="ep-close-sm" onclick={() => extendedEmojiMsgId = null}>✕</button>
              </div>
              <div class="extended-emoji-grid">
                {#each ALL_EMOJIS as emoji}
                  <button class="extended-emoji-btn" onclick={() => { toggleReaction(msg.id, emoji); extendedEmojiMsgId = null; }} title="{emoji}">{emoji}</button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/each}

        {#if chatStore.loadingMore}
          <div class="load-more-indicator">⏳ Chargement…</div>
        {:else if chatStore.hasMore}
          <button class="load-more-btn" onclick={() => handleMessagesScroll({ target: chatContainer } as unknown as Event)}>
            ↓ Messages plus anciens
          </button>
        {/if}
    </div>

    {#if typingUsers.length > 0}
      <div class="typing-indicator">
        <span class="typing-dots">
          <span></span><span></span><span></span>
        </span>
        {typingUsers.length === 1 ? 'Quelqu\'un' : typingUsers.length + ' personnes'} est en train d'écrire…
      </div>
    {/if}

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

    <form class="input-area">
      <!-- Mention autocomplete dropdown -->
      {#if showMentions && filteredMentions.length > 0}
        <div class="mention-dropdown">
          {#each filteredMentions as u}
            <button
              type="button"
              class="mention-option"
              onclick={() => selectMention(u.username)}
            >
              <Avatar username={u.username} name={u.name} size={24} userId={u.id} />
              <span class="mention-name">{u.name ?? u.username}</span>
              <span class="mention-handle">@{u.username}</span>
            </button>
          {/each}
        </div>
      {/if}
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
      <textarea
        class="message-input"
        placeholder="Envoyer un message..."
        bind:value={newMessage}
        onkeydown={handleMessageKeydown}
        oninput={handleTyping}
        disabled={false}
        rows={2}
        aria-label="Message"
      ></textarea>
      <button
        type="button"
        class="send-btn"
        onclick={handleSendMessage}
        disabled={!newMessage.trim() || sending}
      >
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
    tabindex="0"
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
    flex-direction: row;
    height: 100%;
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

  .avatar { display: flex; align-items: center; justify-content: center; border-radius: 50%; color: #fff; font-weight: 600; flex-shrink: 0; }
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

  /* Online indicator */
  .online-indicator {
    display: inline-block;
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--text-muted, #94a3b8);
    margin-right: 0.25rem;
    vertical-align: middle;
  }
  
  .online-indicator.online {
    background: var(--accent, #4ade80);
    box-shadow: 0 0 0 2px var(--bg-secondary, #f8fafc);
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
    padding: .15rem .8rem;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border, #e2e8f0);
    display: flex;
    align-items: center;
    gap: .5rem;
    min-height: 30px;
  }
  .chat-header h2 { margin: 0; font-size: .85rem; color: var(--text-primary, #1e293b); flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .conn-error { font-size: .78rem; color: #dc2626; }
  
  /* Transferts P2P */
  .p2p-transfers {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--bg-primary, #fff);
    border-bottom: 1px solid var(--border, #e2e8f0);
    padding: .5rem;
    z-index: 10;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
  
  .p2p-transfer {
    display: flex;
    flex-direction: column;
    gap: .25rem;
    padding: .5rem;
    border-radius: .35rem;
    background: var(--bg-secondary, #f8fafc);
    margin-bottom: .5rem;
  }
  
  .p2p-transfer:last-child {
    margin-bottom: 0;
  }
  
  .p2p-transfer.completed {
    background: rgba(34, 197, 94, 0.1);
  }
  
  .p2p-transfer.error {
    background: rgba(239, 68, 68, 0.1);
  }
  
  .p2p-transfer-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: .85rem;
  }
  
  .p2p-transfer-name {
    font-weight: 500;
    color: var(--text-primary, #1e293b);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 200px;
  }
  
  .p2p-transfer-size {
    color: var(--text-secondary, #64748b);
    font-size: .78rem;
  }
  
  .p2p-transfer-progress {
    display: flex;
    flex-direction: column;
    gap: .25rem;
  }
  
  .p2p-progress-bar {
    height: 6px;
    background: var(--border, #e2e8f0);
    border-radius: 3px;
    overflow: hidden;
  }
  
  .p2p-progress-fill {
    height: 100%;
    background: var(--accent, #4ade80);
    transition: width .3s ease;
  }
  
  .p2p-transfer.completed .p2p-progress-fill {
    background: #22c55e;
  }
  
  .p2p-transfer.error .p2p-progress-fill {
    background: #ef4444;
  }
  
  .p2p-transfer-stats {
    display: flex;
    justify-content: space-between;
    font-size: .78rem;
    color: var(--text-secondary, #64748b);
  }

  .p2p-cancel-btn {
    background: none;
    border: none;
    color: #dc2626;
    cursor: pointer;
    font-size: .9rem;
    padding: 0 4px;
    margin-left: 8px;
  }
  
  .time-remaining {
    font-size: .7rem;
    color: var(--text-secondary, #64748b);
    margin-left: 4px;
    font-style: italic;
  }
  
  .p2p-transfer-stats span:first-child {
    display: flex;
    align-items: center;
    gap: .25rem;
  }
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
    overflow-x: visible;
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
  }
  .message.mine {
    background: var(--chat-mine, #dcfce7);
    align-self: flex-end;
  }
  .message-sender {
    font-size: .75rem; font-weight: 700;
    color: var(--accent, #4ade80); margin-bottom: .15rem;
  }
  .message-header {
    display: flex; align-items: center; gap: 6px; margin-bottom: 4px;
  }
  .mine-header {
    flex-direction: row-reverse;
  }
  .message-content .mention {
    display: inline; background: var(--accent-light, rgba(74, 222, 128, 0.2));
    color: var(--accent-dark, var(--accent)); font-weight: 700;
    padding: 1px 4px; border-radius: 4px; cursor: pointer;
  }
  .mention-dropdown {
    position: absolute; bottom: 100%; left: 0; right: 0;
    background: var(--bg-primary, #fff); border: 2px solid var(--border);
    border-radius: var(--radius-lg, 12px); box-shadow: var(--shadow-lg, 0 4px 12px rgba(0,0,0,.15));
    max-height: 180px; overflow-y: auto; z-index: 100;
    padding: 4px; margin-bottom: 4px;
  }
  .mention-option {
    display: flex; align-items: center; gap: 8px; width: 100%;
    padding: 8px 12px; border: none; background: transparent;
    cursor: pointer; border-radius: 8px; font-size: .9rem; color: var(--text-primary);
  }
  .mention-option:hover { background: var(--bg-tertiary, #e2e8f0); }
  .mention-handle { color: var(--text-secondary, #64748b); font-size: .8rem; }
  .message-content {
    font-size: .9rem; color: var(--text-primary, #1e293b); line-height: 1.5;
  }
  .message-content :global(img.uploaded-image),
  .message-content :global(img.chat-gif) {
    max-width: 100%; height: auto; border-radius: 8px; margin-top: .3rem; display: block;
  }
  .message-content :global(.file-preview) {
    position: relative; display: inline-block; max-width: 100%;
  }
  .message-content :global(.file-preview img) {
    max-width: 100%; height: auto; border-radius: 8px; display: block;
  }
  .message-content :global(.file-download) {
    position: absolute; top: 6px; right: 6px;
    background: rgba(0,0,0,.6); color: #fff;
    border-radius: 50%; width: 28px; height: 28px;
    display: flex; align-items: center; justify-content: center;
    text-decoration: none; font-size: .8rem;
    opacity: .7; transition: opacity .2s;
  }
  .message-content :global(.file-preview:hover .file-download),
  .message-content :global(.file-audio:hover .file-download),
  .message-content :global(.file-video:hover .file-download) {
    opacity: 1;
  }
  .message-content :global(.file-audio),
  .message-content :global(.file-video) {
    position: relative; display: inline-block;
  }
  .message-content :global(audio.chat-audio) {
    max-width: 300px; height: 36px; border-radius: 8px;
  }
  .message-content :global(video.chat-video) {
    max-width: 350px; border-radius: 8px;
  }
  .message-content :global(.file-attachment a) {
    color: var(--accent, #4ade80); text-decoration: underline;
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

  /* Sidebar backdrop — hidden on desktop, shown on mobile when sidebar open */
  .sidebar-backdrop {
    display: none;
  }

  /* Menu contextuel message */
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
    display: inline-flex; align-items: center; gap: .3rem;
    padding: .25rem .6rem; border-radius: 999px;
    border: 1.5px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    font-size: 1.1rem; cursor: pointer; transition: all .15s;
    color: var(--text-primary, #1e293b);
    line-height: 1.2;
  }
  .reaction-pill:hover { border-color: var(--accent, #4ade80); background: var(--bg-tertiary, #f0fdf4); }
  .reaction-pill.my-reaction {
    border-color: var(--accent, #4ade80);
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    font-weight: 600;
  }

  /* ─── Typing indicator ─── */
  .typing-indicator {
    display: flex; align-items: center; gap: .5rem;
    padding: .3rem 1rem;
    font-size: .75rem;
    color: var(--text-secondary, #94a3b8);
    flex-shrink: 0;
  }
  .typing-dots {
    display: inline-flex; gap: .15rem;
  }
  .typing-dots span {
    width: 4px; height: 4px;
    border-radius: 50%;
    background: var(--text-secondary, #94a3b8);
    animation: typingBounce 1.2s infinite;
  }
  .typing-dots span:nth-child(2) { animation-delay: .2s; }
  .typing-dots span:nth-child(3) { animation-delay: .4s; }
  @keyframes typingBounce {
    0%, 60%, 100% { transform: translateY(0); }
    30% { transform: translateY(-4px); }
  }

  /* Emoji picker */
  .emoji-picker {
    position: fixed;
    z-index: 9999;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .6rem; padding: .35rem .4rem;
    box-shadow: 0 4px 16px rgba(0,0,0,.12);
    display: flex; flex-wrap: wrap; gap: .2rem; max-width: 240px;
    animation: pop .12s var(--animation, ease);
  }
  .emoji-picker.picker-mine { left: auto; right: 10px; }
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
    font-size: 3.5rem !important;
    line-height: 1.2;
    background: transparent !important;
    padding: .2rem .4rem !important;
    box-shadow: none !important;
    border-radius: 0 !important;
    text-align: center;
  }

  .input-area {
    flex-shrink: 0;
    position: relative;
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
    border-radius: .75rem; font-size: .9rem; outline: none;
    transition: border-color .15s, height .1s;
    background: var(--bg-primary, #fff); color: var(--text-primary, #1e293b);
    resize: none;
    min-height: 72px;
    max-height: 160px;
    line-height: 1.4;
    overflow-y: auto;
    font-family: inherit;
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
  /* ════════════════════════════════════════════════════════
     MOBILE — Responsive styles (< 640px)
     ════════════════════════════════════════════════════════ */

  /* ── Layout: sidebar hidden by default, overlay mode ── */
  .chat-page {
    position: relative;
    height: 100%; /* Fill app-main (which is viewport-height - header - footer) */
  }

  .conversations-sidebar {
    position: fixed;
    top: 0;
    left: 0;
    width: 85vw;
    max-width: 320px;
    height: 100%;
    z-index: 100;
    transform: translateX(-100%);
    transition: transform 0.25s ease;
    box-shadow: 4px 0 20px rgba(0,0,0,0.15);
  }

  .conversations-sidebar.open {
    transform: translateX(0);
  }

  /* Backdrop when sidebar open */
  .sidebar-backdrop {
    display: none;
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    z-index: 99;
  }

  .sidebar-backdrop.visible {
    display: block;
  }

  /* Chat main takes full width */
  .chat-area {
    width: 100% !important;
    flex: 1 !important;
    min-height: 0; /* Allow flex child to shrink */
  }

  /* ── Chat header: hamburger + compact ── */
  .chat-header {
    padding: .1rem .5rem !important;
    gap: .25rem;
    min-height: 28px !important;
  }

  .btn-menu-mobile {
    display: flex !important;
    width: 32px;
    height: 32px;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1.3rem;
    flex-shrink: 0;
  }

  .btn-menu-mobile:active {
    background: var(--bg-secondary, #f1f5f9);
  }

  /* ── Messages: full width, smaller padding ── */
  .message {
    max-width: 92% !important;
  }

  .message-content {
    font-size: .92rem;
    padding: .45rem .7rem;
  }

  /* ── Touch targets: minimum 44px ── */
  .conversation-item {
    padding: .65rem .6rem !important;
    min-height: 48px;
  }

  .msg-action-btn {
    min-width: 36px;
    min-height: 36px;
  }

  button {
    min-height: 36px;
  }

  /* ── Input area: compact ── */
  .input-area {
    padding: .4rem .5rem !important;
  }

  .message-input {
    font-size: 16px; /* Prevent iOS zoom */
    padding: .5rem .7rem;
    min-height: 72px;
    max-height: 160px;
    border-radius: .75rem;
    resize: none;
    overflow-y: auto;
    line-height: 1.4;
  }

  .input-actions {
    gap: .2rem;
  }

  .input-actions button {
    width: 34px;
    height: 34px;
    font-size: 1rem;
  }

  /* ── Emoji picker: smaller ── */
  .emoji-picker,
  [data-testid="emoji-picker"] {
    max-width: 280px !important;
    right: 0 !important;
    left: auto !important;
  }

  /* ── Reactions: compact ── */
  .reactions-row {
    max-width: 100%;
  }

  .reaction-pill {
    font-size: 1.1rem;
    padding: .25rem .6rem;
  }

  /* ── Audio/Video call buttons ── */
  .btn-call {
    width: 34px;
    height: 34px;
  }

  /* ── Scrollbar: thinner on mobile ── */
  ::-webkit-scrollbar {
    width: 4px;
  }

  /* ── GIF viewer ── */
  :global(.chat-gif) {
    max-width: 260px !important;
    max-height: 260px !important;
  }

  /* ── Hide desktop-only elements ── */
  .sidebar-header h2 {
    font-size: .7rem;
  }

  /* ── Conversation info: smaller ── */
  .conversation-info .name {
    font-size: .85rem;
  }

  .conversation-info .last-msg {
    font-size: .72rem;
  }
  } /* end @media */
  .typing-indicator {
    display: flex;
    align-items: center;
    gap: .5rem;
    padding: .5rem 1rem;
    font-size: .8rem;
    color: var(--text-secondary, #64748b);
    animation: fadeIn .3s ease;
  }
  .typing-dots {
    display: flex;
    gap: 3px;
  }
  .typing-dots span {
    width: 6px;
    height: 6px;
    background: var(--accent, #4ade80);
    border-radius: 50%;
    animation: typingBounce 1.2s ease-in-out infinite;
  }
  .typing-dots span:nth-child(2) { animation-delay: 0.2s; }
  .typing-dots span:nth-child(3) { animation-delay: 0.4s; }
  @keyframes typingBounce {
    0%, 60%, 100% { transform: translateY(0); }
    30% { transform: translateY(-4px); }
  }

  /* ─── Message Wrapper & Reactions ─── */
  .message-wrapper {
    position: relative;
    margin-bottom: .5rem;
    display: flex;
    flex-direction: row;
    align-items: flex-end;
    gap: .5rem;
  }
  .message-wrapper.mine {
    flex-direction: row-reverse;
  }
  .message-wrapper.is-emoji-only {
    flex-direction: column;
    justify-content: center;
    margin: .8rem 0;
  }
  .message-avatar {
    flex-shrink: 0;
    margin-bottom: .1rem;
  }

  /* Message column: stacks message bubble + reactions vertically */
  .message-column {
    display: flex;
    flex-direction: column;
    max-width: 70%;
    align-items: center;
  }

  .message-wrapper.is-emoji-only .message {
    font-size: 2.5rem;
    background: none;
    padding: .5rem;
    box-shadow: none;
  }
  .message {
    padding: .5rem .75rem;
    border-radius: .65rem;
    word-break: break-word;
    line-height: 1.45;
    font-size: .92rem;
    width: fit-content;
    max-width: 100%;
  }
  .message.mine {
    background: var(--accent, #4ade80);
    color: #fff;
    border-bottom-right-radius: .2rem;
  }
  .message.theirs {
    background: var(--bg-secondary, #f1f5f9);
    color: var(--text-primary, #1e293b);
    border-bottom-left-radius: .2rem;
  }

  /* Message actions (visible on hover) */
  .message-actions {
    display: flex;
    gap: .25rem;
    padding: .25rem;
    position: absolute;
    bottom: 100%;
    left: 0;
    margin-bottom: .2rem;
    animation: fadeIn .15s ease;
    z-index: 10;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .5rem;
    box-shadow: 0 4px 12px rgba(0,0,0,.1);
  }
  .message-wrapper.mine .message-actions {
    left: auto;
    right: 0;
  }
  .quick-react-btn {
    width: 28px; height: 28px;
    display: flex; align-items: center; justify-content: center;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .35rem;
    background: var(--bg-primary, #fff);
    cursor: pointer;
    font-size: 1rem;
    transition: all .12s;
    padding: 0;
  }
  .quick-react-btn:hover {
    background: var(--bg-secondary, #f1f5f9);
    transform: scale(1.15);
    border-color: var(--accent, #4ade80);
  }
  .action-btn {
    width: 28px; height: 28px;
    display: flex; align-items: center; justify-content: center;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .35rem;
    background: var(--bg-primary, #fff);
    cursor: pointer;
    font-size: .82rem;
    transition: all .12s;
    padding: 0;
  }
  .action-btn:hover {
    background: var(--bg-secondary, #f1f5f9);
    border-color: var(--accent, #4ade80);
  }
  .react-more {
    font-size: .82rem;
    width: auto;
    padding: 0 .4rem;
  }

  /* Message menu */
  .msg-menu-toggle {
    font-size: .9rem;
    width: auto;
    padding: 0 .35rem;
    letter-spacing: .15em;
  }
  .msg-menu-toggle.active {
    background: var(--accent, #4ade80);
    color: #fff;
    border-color: var(--accent, #4ade80);
  }
  .message-menu-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 60;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .5rem;
    box-shadow: 0 4px 16px rgba(0,0,0,.15);
    padding: .25rem 0;
    min-width: 130px;
  }
  .msg-menu-item {
    display: flex;
    align-items: center;
    gap: .4rem;
    width: 100%;
    padding: .35rem .6rem;
    border: none;
    background: none;
    cursor: pointer;
    font-size: .85rem;
    color: var(--text-primary, #1e293b);
    text-align: left;
    transition: background .1s;
  }
  .msg-menu-item:hover { background: var(--bg-secondary, #f1f5f9); }
  .msg-menu-item.delete { color: var(--danger, #ef4444); }
  .msg-menu-item.delete:hover { background: #fef2f2; }

  /* Extended emoji picker for messages (emoji-picker-element) */
  /* position: relative pour que le conteneur parent fixe les limites */
  .msg-emoji-picker {
    position: fixed;
    z-index: 50;
    width: 320px;
    max-height: 400px;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .5rem;
    box-shadow: 0 4px 16px rgba(0,0,0,0.15);
    background: var(--bg-primary, #fff);
  }
  /* Override emoji-picker-element default styles to match Nook theme */
  .msg-emoji-picker emoji-picker {
    --emoji-picker-background: var(--bg-primary, #fff);
    --emoji-picker-border-color: var(--border, #e2e8f0);
    --emoji-picker-input-background: var(--bg-secondary, #f1f5f9);
    --emoji-picker-input-text-color: var(--text-primary, #1e293b);
    --emoji-picker-category-color: var(--text-secondary, #64748b);
    --emoji-picker-hover-background: var(--bg-secondary, #f1f5f9);
    --emoji-picker-selected-background: var(--accent, #4ade80);
    font-size: 0.9rem;
  }
  .ep-close-sm {
    position: absolute;
    top: .25rem; right: .25rem;
    width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    border: none;
    background: none;
    cursor: pointer;
    font-size: .7rem;
    color: var(--text-secondary, #94a3b8);
    padding: 0;
  }
  .ep-close-sm:hover { color: var(--text-primary, #1e293b); }

  /* Extended emoji picker (ALL_EMOJIS grid) */
  .extended-emoji-picker {
    position: fixed;
    z-index: 50;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .5rem;
    box-shadow: 0 4px 16px rgba(0,0,0,0.15);
    padding: .4rem;
    max-width: 340px;
    max-height: 360px;
    overflow-y: auto;
    animation: pop .12s ease;
  }
  .extended-emoji-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 .3rem .3rem;
    border-bottom: 1px solid var(--border, #e2e8f0);
    margin-bottom: .3rem;
    font-size: .8rem;
    font-weight: 600;
    color: var(--text-secondary, #64748b);
  }
  .extended-emoji-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: .15rem;
  }
  .extended-emoji-btn {
    padding: .2rem; border: none; background: transparent;
    font-size: 1.2rem; cursor: pointer; border-radius: .3rem;
    transition: background .1s, transform .1s; line-height: 1;
  }
  .extended-emoji-btn:hover { background: var(--bg-secondary, #f1f5f9); transform: scale(1.2); }

  /* Message reactions display */
  .message-reactions {
    display: flex;
    gap: .25rem;
    flex-wrap: wrap;
    margin-top: .3rem;
  }
  .reaction-badge {
    display: flex;
    align-items: center;
    gap: .2rem;
    padding: .15rem .4rem;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 999px;
    background: var(--bg-primary, #fff);
    cursor: pointer;
    font-size: .78rem;
    transition: all .12s;
  }
  .reaction-badge:hover {
    background: var(--bg-secondary, #f1f5f9);
    border-color: var(--accent, #4ade80);
  }
  .reaction-badge.my-reaction {
    background: color-mix(in srgb, var(--accent, #4ade80) 15%, var(--bg-primary, #fff));
    border-color: var(--accent, #4ade80);
  }
  .reaction-emoji {
    font-size: .95rem;
  }
  .reaction-count {
    font-weight: 600;
    color: var(--text-secondary, #64748b);
  }
  .my-reaction .reaction-count {
    color: var(--accent, #4ade80);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .encrypted-placeholder {
    color: #94a3b8;
    font-style: italic;
    font-size: 0.85rem;
    padding: 0.3rem 0.5rem;
  }
</style>
