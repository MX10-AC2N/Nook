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
  sender_avatar_style: string | null;
  sender_avatar_seed: string | null;
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

import { writable, get } from 'svelte/store';
import { callManager } from '$lib/webrtc-calls.svelte.ts';
import { cryptoStore } from '$lib/cryptoStore.svelte';

// Svelte writable store for messages — proper cross-file reactivity
export const messagesStore = writable<ChatMessage[]>([]);

// ── Déchiffrement automatique quand le cryptoStore devient prêt ──────────
// Quand l'utilisateur se reconnecte après un rafraîchissement, les messages
// sont chargés chiffrés. Dès que cryptoStore.ready devient true, on déchiffre.
// FIX : Ne plus s'arrêter après la première tentative (race condition).
let _cryptoReadyInterval: ReturnType<typeof setInterval> | null = null;
let _cryptoReadyAttempts = 0;
const _CRYPTO_READY_MAX_ATTEMPTS = 600; // 10 minutes à 1s d'intervalle

async function _decryptAllIfReady(): Promise<void> {
  if (!cryptoStore.ready) return;
  
  const msgs = get(messagesStore);
  const encrypted = msgs.filter(m => m.encrypted && m.nonce && m.sender_public_key);
  
  if (encrypted.length === 0) {
    // Plus de messages chiffrés, on continue à surveiller un peu au cas où
    if (_cryptoReadyAttempts > 10) {
      _stopCryptoReadyListener();
    }
    return;
  }
  
  console.log(`[Chat] Crypto prêt → déchiffrement de ${encrypted.length} messages (attempt ${_cryptoReadyAttempts})`);
  try {
    const { decryptMessage } = await import('$lib/cryptoStore.svelte');
    for (const msg of encrypted) {
      try {
        msg.content = await decryptMessage({
          messageId: msg.id, conversationId: msg.conversation_id,
          ciphertext: msg.content, nonce: msg.nonce!, senderPubkeyB64: msg.sender_public_key!,
        });
        msg.encrypted = false;
      } catch (e) {
        console.error('[Chat] Erreur déchiffrement message', msg.id, e);
        msg.content = '🔒 Message chiffré (clé indisponible)';
      }
    }
    messagesStore.set([...msgs]);
  } catch (e) {
    console.error('[Chat] Erreur import cryptoStore', e);
  }
}

function _stopCryptoReadyListener(): void {
  if (_cryptoReadyInterval) {
    clearInterval(_cryptoReadyInterval);
    _cryptoReadyInterval = null;
    _cryptoReadyAttempts = 0;
    console.log('[Chat] Listener crypto ready arrêté');
  }
}

function _setupCryptoReadyListener(): void {
  if (_cryptoReadyInterval) return;
  
  // Vérification immédiate
  _decryptAllIfReady();
  
  // Polling léger (1s) — fix race condition : ne pas s'arrêter après 1ère tentative
  _cryptoReadyInterval = setInterval(() => {
    _cryptoReadyAttempts++;
    
    // Sécurité : arrêt après 10 min
    if (_cryptoReadyAttempts > _CRYPTO_READY_MAX_ATTEMPTS) {
      console.warn('[Chat] Listener crypto ready timeout (10 min)');
      _stopCryptoReadyListener();
      return;
    }
    
    _decryptAllIfReady();
  }, 1000);
  
  console.log('[Chat] Listener crypto ready démarré');
}

// Démarrer le listener au chargement du module
_setupCryptoReadyListener();

// Other state as $state (less critical for cross-file reactivity)
export const chatStore = $state<Omit<ChatState, 'messages'>>({
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
    messagesStore.update(msgs => msgs.map(m => m.id === id ? { ...m, content, edited_at: editedAt } : m));
    return;
  }

  if (type === 'message_deleted') {
    const id = msg.message_id as string;
    messagesStore.update(msgs => msgs.filter(m => m.id !== id));
    return;
  }

  if (type === 'reaction_updated') {
    const msgId = msg.message_id as string;
    const convId = msg.conversation_id as string;
    // Reactions are handled by the +page.svelte via lastReactionUpdate signal
    chatStore.lastReactionUpdate = {
      messageId: msgId,
      conversationId: convId,
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

  // ── Call signaling ──
  const CALL_TYPES = ['call_request', 'call_accepted', 'call_rejected', 'offer', 'answer', 'ice', 'ice_candidate', 'webrtc_offer', 'webrtc_answer', 'webrtc_ice_candidate', 'join', 'leave', 'decline'];
  if (type && CALL_TYPES.includes(type)) {
    console.log(`[WS] Call signal received: ${type}`, msg);
    // Forward to webrtc-calls handler via its signal handler
    callManager.handleSignal?.(msg as any);
    return;
  }

  // ── Admin notifications ──
  if (type === 'user_approved') {
    notifyAdmin('✅ Utilisateur approuvé', 'Votre compte a été approuvé !', '/chat');
    return;
  }
}

async function _injectMessage(raw: ChatMessage): Promise<void> {
  const existing = get(messagesStore).findIndex(m => m.id === raw.id);
  // If existing message is already plaintext, skip encrypted WS update
  if (existing !== -1 && !get(messagesStore)[existing].encrypted && raw.encrypted) {
    console.debug('[WS] Skip encrypted update for plaintext msg', raw.id);
    return;
  }
  if (raw.encrypted && raw.nonce && raw.sender_public_key) {
    try {
      const { cryptoStore: cs, decryptMessage } = await import('$lib/cryptoStore.svelte');
      if (cs.ready) {
        raw.content = await decryptMessage({
          messageId: raw.id, conversationId: raw.conversation_id,
          ciphertext: raw.content, nonce: raw.nonce!, senderPubkeyB64: raw.sender_public_key!,
        });
        raw.encrypted = false;
      }
    } catch { raw.content = '🔒 Message chiffré (clé indisponible)'; }
  }
  if (existing === -1) {
    messagesStore.update(msgs => [...msgs, raw]);
  } else {
    messagesStore.update(msgs => { msgs[existing] = raw; return [...msgs]; });
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

function _updatePageTitle(): void {
  if (typeof document === 'undefined') return;
  const unread = Object.values(chatStore.unreadCounts).reduce((a, b) => a + b, 0);
  document.title = unread > 0 ? `(${unread}) Nook` : 'Nook';
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
  messagesStore.set([]);
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
    messagesStore.set([...msgs]);
    chatStore.hasMore  = msgs.length >= PAGE_SIZE;
    chatStore.connectionError = null;
    console.log('[Chat] messagesStore set:', get(messagesStore).length);
    // APRÈS chargement, si crypto prêt → déchiffrer (race condition)
    if (cryptoStore.ready) _decryptAllIfReady();
  } catch (err) {
    chatStore.connectionError = 'Erreur de chargement des messages';
    console.error('[Chat] loadMessages:', err);
  }
}

/** Charge les messages plus anciens (pagination vers le haut) */
export async function loadMoreMessages(conversationId: string): Promise<void> {
  if (chatStore.loadingMore || !chatStore.hasMore) return;
  const msgs = get(messagesStore);
  const oldest = msgs[0]; // Premier message (le plus ancien) pour charger les précédents
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
    // Append older messages at end (bottom) — display is reversed
    messagesStore.update(msgs => [...msgs, ...older]);
    chatStore.hasMore  = older.length >= PAGE_SIZE;
  } catch (err) {
    console.error('[Chat] loadMoreMessages:', err);
  } finally {
    chatStore.loadingMore = false;
    // APRÈS chargement, si crypto prêt → déchiffrer
    if (cryptoStore.ready) _decryptAllIfReady();
  }
}

// -----------------------------------------------------------------
// 8️⃣ API — sendMessage
// -----------------------------------------------------------------

export async function sendMessage(content: string, conversationId: string): Promise<ChatMessage | null> {
  if (!content.trim()) return null;
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
    const msgData: ChatMessage = await res.json();
    chatStore.connectionError = null;
    // Use plaintext content locally (API returns encrypted)
    msgData.content = content.trim();
    msgData.encrypted = false;
    // Always update store with plaintext — overwrites WS encrypted entry if race
    messagesStore.update(msgs => {
      const idx = msgs.findIndex(m => m.id === msgData.id);
      if (idx !== -1) { msgs[idx] = msgData; return [...msgs]; }
      return [...msgs, msgData];
    });
    // Recharger après 500ms pour assurer la persistance backend
    setTimeout(() => loadMessages(conversationId), 500);
    return msgData;
  } catch (err) {
    chatStore.connectionError = "Erreur lors de l'envoi du message";
    console.error('[Chat] sendMessage:', err);
    return null;
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
    // Optimistic local update (WS will confirm or overwrite)
    const editedAt = Math.floor(Date.now() / 1000);
    messagesStore.update(msgs => msgs.map(m => m.id === msgId ? { ...m, content: newContent, edited_at: editedAt } : m));
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
      messagesStore.update(msgs => msgs.filter(m => m.id !== msgId));
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
