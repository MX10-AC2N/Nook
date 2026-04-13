/**
 * chatStore.svelte.ts — Store du chat, Svelte 5 Runes.
 *
 * Session 31 — refonte majeure :
 *   - WS temps réel : new_message, message_edited, message_deleted
 *   - Reconnexion automatique WS (backoff exponentiel)
 *   - Pagination messages (before + limit)
 *   - Edit / Delete de messages
 *   - Badges non-lus (unreadCounts par conversation)
 *   - Notifications navigateur (Permission API)
 */

// -----------------------------------------------------------------
// 1️⃣ Types
// -----------------------------------------------------------------

export interface ChatMessage {
  id: string;
  conversation_id: string;
  sender_id: string;
  sender_name: string;
  sender_public_key: string | null;
  content: string;
  message_type: string;
  file_id: string | null;
  encrypted: boolean;
  nonce: string | null;
  timestamp: number;
  created_at: number;
  edited_at: number | null;
}

export interface ChatState {
  messages: ChatMessage[];
  connectionError: string | null;
  showEmojiPicker: boolean; // picker emoji natif (remplace GIF Tenor S39)
  hasMore: boolean;
  loadingMore: boolean;
  unreadCounts: Record<string, number>;
  wsConnected: boolean;
  /** Signal WS pour les mises à jour de réactions — { messageId, conversationId, ts } */
  lastReactionUpdate: { messageId: string; conversationId: string; ts: number } | null;
}

// -----------------------------------------------------------------
// 2️⃣ État réactif
// -----------------------------------------------------------------

// Svelte 5 reactive state — exported directly (properties are reactive)
export const chatStore = $state<ChatState>({
  messages: [],
  connectionError: null,
  showEmojiPicker: false,
  hasMore: false,
  loadingMore: false,
  unreadCounts: {},
  wsConnected: false,
  lastReactionUpdate: null,
});

// -----------------------------------------------------------------
// 3️⃣ WebSocket — temps réel avec reconnexion automatique
// -----------------------------------------------------------------

let _ws: WebSocket | null = null;
let _wsConvId: string | null = null;
let _wsReconnectTimer: ReturnType<typeof setTimeout> | null = null;
let _wsRetries = 0;
const WS_MAX_RETRIES = 8;

export function connectWs(convId: string): void {
  if (typeof window === 'undefined') return;
  _wsConvId = convId;
  if (_ws?.readyState === WebSocket.OPEN) return;
  _openWs();
}

function _openWs(): void {
  if (_ws) { _ws.onclose = null; _ws.close(); _ws = null; }
  if (typeof window === 'undefined') return;

  const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${window.location.host}/ws`);
  _ws = ws;

  ws.onopen = () => {
    chatStore.wsConnected = true;
    _wsRetries = 0;
    if (_wsReconnectTimer) { clearTimeout(_wsReconnectTimer); _wsReconnectTimer = null; }
  };

  ws.onmessage = (ev) => {
    try { _handleWsMessage(JSON.parse(ev.data as string)); } catch { /* non-JSON ok */ }
  };

  ws.onerror = () => {};

  ws.onclose = () => {
    chatStore.wsConnected = false;
    if (_wsRetries < WS_MAX_RETRIES) {
      const delay = Math.min(1000 * 2 ** _wsRetries, 30_000);
      _wsRetries++;
      _wsReconnectTimer = setTimeout(_openWs, delay);
    }
  };
}

function _handleWsMessage(msg: Record<string, unknown>): void {
  const type = msg.type as string | undefined;

  if (type === 'new_message') {
    const convId = msg.conversation_id as string;
    const raw    = msg.message as ChatMessage;
    if (!raw || !convId) return;
    if (convId === _wsConvId) {
      _injectMessage(raw);
    } else {
      chatStore.unreadCounts[convId] = (chatStore.unreadCounts[convId] ?? 0) + 1;
      _sendBrowserNotification(raw.sender_name, raw.content);
    }
    return;
  }

  if (type === 'message_edited') {
    const id      = msg.message_id as string;
    const content = msg.content as string;
    const editedAt = msg.edited_at as number;
    const idx = chatStore.messages.findIndex(m => m.id === id);
    if (idx !== -1) {
      chatStore.messages[idx] = { ...chatStore.messages[idx], content, edited_at: editedAt };
    }
    return;
  }

  if (type === 'message_deleted') {
    const id = msg.message_id as string;
    chatStore.messages = chatStore.messages.filter(m => m.id !== id);
    return;
  }

  if (type === 'reaction_updated') {
    const msgId = msg.message_id as string;
    const reactionData = msg.reactions;
    if (msgId && reactionData) {
      reactions[msgId] = reactionData;
    }
    chatStore.lastReactionUpdate = {
      messageId: msgId,
      conversationId: msg.conversation_id as string,
      ts: Date.now(),
    };
    return;
  }

  // ── Poll notifications ──
  if (type === 'new_poll') {
    const title = msg.title as string || 'Nouveau sondage';
    notifyPoll('📊 Sondage créé', title, '/polls');
    return;
  }

  if (type === 'poll_voted') {
    const voter = msg.voter as string || 'Quelqu\'un';
    notifyPoll('🗳️ Nouveau vote', `${voter} a voté`, '/polls');
    return;
  }

  if (type === 'poll_closed') {
    notifyPoll('📊 Sondage fermé', 'Un sondage est terminé', '/polls');
    return;
  }

  // ── Calendar notifications ──
  if (type === 'new_event') {
    const title = msg.title as string || 'Nouvel événement';
    const creator = msg.creator as string || 'Quelqu\'un';
    notifyCalendar('📅 Événement créé', `${creator}: ${title}`, '/calendar');
    return;
  }

  // ── Admin notifications ──
  if (type === 'user_approved') {
    notifyAdmin('✅ Utilisateur approuvé', 'Votre compte a été approuvé !', '/chat');
    return;
  }
}

async function _injectMessage(raw: ChatMessage): Promise<void> {
  if (raw.encrypted && raw.nonce && raw.sender_public_key) {
    try {
      const { cryptoStore: cs, decryptMessage } = await import('$lib/cryptoStore.svelte');
      if (cs.ready) {
        raw.content = await decryptMessage({
          messageId: raw.id, conversationId: raw.conversation_id,
          ciphertext: raw.content, nonce: raw.nonce!, senderPubkeyB64: raw.sender_public_key!,
        });
      }
    } catch { raw.content = '🔒 Message chiffré (clé indisponible)'; }
  }
  if (!chatStore.messages.find(m => m.id === raw.id)) {
    chatStore.messages = [...chatStore.messages, raw];
  }
}

export function disconnectWs(): void {
  if (_wsReconnectTimer) { clearTimeout(_wsReconnectTimer); _wsReconnectTimer = null; }
  if (_ws) { _ws.onclose = null; _ws.close(); _ws = null; }
  _wsConvId = null;
  chatStore.wsConnected = false;
  _wsRetries = 0;
}

export function setActiveConv(convId: string): void {
  _wsConvId = convId;
  chatStore.unreadCounts[convId] = 0;
  _updatePageTitle();
  connectWs(convId);
}

// -----------------------------------------------------------------
// 4️⃣ Notifications navigateur
// -----------------------------------------------------------------

export async function requestNotificationPermission(): Promise<void> {
  if (typeof Notification === 'undefined') return;
  if (Notification.permission === 'default') {
    await Notification.requestPermission();
  }
}

import { notifyMessage, notifyPoll, notifyCalendar, notifyAdmin } from '$lib/notificationStore.svelte';

function _sendBrowserNotification(sender: string, content: string): void {
  const text = content.startsWith('<img') ? '📷 Image' : content.startsWith('<audio') ? '🎙️ Message vocal' : content.slice(0, 80);
  if (typeof document !== 'undefined' && document.visibilityState === 'visible') return;
  notifyMessage(sender, text);
}

// -----------------------------------------------------------------
// 5️⃣ Helpers
// -----------------------------------------------------------------

export function toggleEmojiPicker(): void {
  chatStore.showEmojiPicker = !chatStore.showEmojiPicker;
}

export function setConnectionError(err: string | null): void {
  chatStore.connectionError = err;
}

export function resetChat(): void {
  chatStore.messages = [];
  chatStore.connectionError = null;
  chatStore.showEmojiPicker = false;
  
  
  chatStore.hasMore = false;
  chatStore.loadingMore = false;
}

export function formatTimestamp(ts: number): string {
  const date = new Date(ts * 1000);
  const now  = new Date();
  if (date.toDateString() === now.toDateString())
    return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  if (date.getFullYear() === now.getFullYear())
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

// -----------------------------------------------------------------
// 6️⃣ Déchiffrement batch
// -----------------------------------------------------------------

async function _decryptBatch(msgs: ChatMessage[]): Promise<ChatMessage[]> {
  try {
    const { cryptoStore: cs, decryptMessage } = await import('$lib/cryptoStore.svelte');
    if (!cs.ready) return msgs;
    for (const msg of msgs) {
      if (msg.encrypted && msg.nonce && msg.sender_public_key) {
        try {
          msg.content = await decryptMessage({
            messageId: msg.id, conversationId: msg.conversation_id,
            ciphertext: msg.content, nonce: msg.nonce!, senderPubkeyB64: msg.sender_public_key!,
          });
        } catch { msg.content = '🔒 Message chiffré (clé indisponible)'; }
      }
    }
  } catch { /* cryptoStore pas dispo */ }
  return msgs;
}

// -----------------------------------------------------------------
// 7️⃣ API — loadMessages (50 derniers)
// -----------------------------------------------------------------

const PAGE_SIZE = 50;

export async function loadMessages(conversationId: string): Promise<void> {
  try {
    const res = await fetch(
      `/api/conversations/${conversationId}/messages?limit=${PAGE_SIZE}`,
      { credentials: 'include' }
    );
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    const msgs: ChatMessage[] = Array.isArray(data) ? data : (data.messages ?? []);
    console.log('[Chat] loadMessages:', msgs.length, 'messages loaded for', conversationId);
    msgs.sort((a, b) => a.created_at - b.created_at);
    await _decryptBatch(msgs);
    chatStore.messages = [...msgs];
    chatStore.hasMore  = msgs.length >= PAGE_SIZE;
    chatStore.connectionError = null;
    console.log('[Chat] chatStore.messages set:', chatStore.messages.length);
  } catch (err) {
    chatStore.connectionError = 'Erreur de chargement des messages';
    console.error('[Chat] loadMessages:', err);
  }
}

/** Charge les messages plus anciens (pagination vers le haut) */
export async function loadMoreMessages(conversationId: string): Promise<void> {
  if (chatStore.loadingMore || !chatStore.hasMore) return;
  const oldest = chatStore.messages[0];
  if (!oldest) return;
  chatStore.loadingMore = true;
  try {
    const res = await fetch(
      `/api/conversations/${conversationId}/messages?limit=${PAGE_SIZE}&before=${oldest.created_at}`,
      { credentials: 'include' }
    );
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    const older: ChatMessage[] = Array.isArray(data) ? data : (data.messages ?? []);
    older.sort((a, b) => a.created_at - b.created_at);
    await _decryptBatch(older);
    chatStore.messages = [...older, ...chatStore.messages];
    chatStore.hasMore  = older.length >= PAGE_SIZE;
  } catch (err) {
    console.error('[Chat] loadMoreMessages:', err);
  } finally {
    chatStore.loadingMore = false;
  }
}

// -----------------------------------------------------------------
// 8️⃣ API — sendMessage
// -----------------------------------------------------------------

export async function sendMessage(content: string, conversationId: string): Promise<void> {
  if (!content.trim()) return;
  try {
    const { cryptoStore: cs, encryptMessage } = await import('$lib/cryptoStore.svelte');
    let body: Record<string, unknown>;
    if (cs.ready) {
      try {
        const enc = await encryptMessage(content.trim(), conversationId);
        body = { content: enc.ciphertext, encrypted: true, nonce: enc.nonce, encrypted_keys: enc.encryptedKeys };
      } catch {
        body = { content: content.trim(), encrypted: false };
      }
    } else {
      body = { content: content.trim(), encrypted: false };
    }
    const res = await fetch(`/api/conversations/${conversationId}/messages`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      credentials: 'include', body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const msgData = await res.json();
    // Optimistic update: add message to DOM immediately
    if (msgData && msgData.id) {
      const alreadyExists = chatStore.messages.some(m => m.id === msgData.id);
      if (!alreadyExists) {
        chatStore.messages = [...chatStore.messages, msgData];
      }
    }
    chatStore.connectionError = null;
    // Also reload from server to get authoritative state
    await loadMessages(conversationId);
  } catch (err) {
    chatStore.connectionError = "Erreur lors de l'envoi du message";
    console.error('[Chat] sendMessage:', err);
  }
}

// -----------------------------------------------------------------
// 9️⃣ API — editMessage
// -----------------------------------------------------------------

export async function editMessage(msgId: string, convId: string, newContent: string): Promise<boolean> {
  try {
    const res = await fetch(`/api/conversations/${convId}/messages/${msgId}`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      credentials: 'include', body: JSON.stringify({ content: newContent }),
    });
    if (!res.ok) return false;
    if (!chatStore.wsConnected) {
      const idx = chatStore.messages.findIndex(m => m.id === msgId);
      if (idx !== -1) {
        chatStore.messages[idx] = { ...chatStore.messages[idx], content: newContent, edited_at: Math.floor(Date.now() / 1000) };
      }
    }
    return true;
  } catch { return false; }
}

// -----------------------------------------------------------------
// 🔟 API — deleteMessage
// -----------------------------------------------------------------

export async function deleteMessage(msgId: string, convId: string): Promise<boolean> {
  try {
    const res = await fetch(`/api/conversations/${convId}/messages/${msgId}`, {
      method: 'DELETE', credentials: 'include',
    });
    if (res.status !== 204 && !res.ok) return false;
    if (!chatStore.wsConnected) {
      chatStore.messages = chatStore.messages.filter(m => m.id !== msgId);
    }
    return true;
  } catch { return false; }
}

// -----------------------------------------------------------------

// 1️⃣1️⃣ API — sendEmoji (envoie un emoji comme message standalone)
// -----------------------------------------------------------------

export async function sendEmoji(emoji: string, conversationId: string): Promise<void> {
  await sendMessage(emoji, conversationId);
}
