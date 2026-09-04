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
  sender_key_version: number | null;
  content: string;
  message_type: string;
  file_id: string | null;
  encrypted: boolean;
  nonce: string | null;
  group_key_version: number | null;
  timestamp: number;
  created_at: number;
  edited_at: number | null;
  // ADR-017: reply-to
  reply_to_id: string | null;
  reply_to: ReplyToPreview | null;
}

/** Preview du message cité (ADR-017), reconstruit côté client depuis les champs reply_to_* */
export interface ReplyToPreview {
  id: string;
  sender_name: string | null;
  content: string | null;
  message_type: string | null;
  file_id: string | null;
  nonce: string | null;
  encrypted: boolean;
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
  /** WebSocket instance for direct access */
  ws: WebSocket | null;
  /** Participants cache per conversation */
  participants: Map<string, Participant[]>;
  /** WebSocket round-trip latency in ms (null = not measured) */
  latencyMs: number | null;
}

export interface Participant {
  id: string;
  username: string;
  name: string | null;
  role: string;
}

// -----------------------------------------------------------------
// 2️⃣ État réactif
// -----------------------------------------------------------------

import { writable, get } from 'svelte/store';
import { callManager } from '$lib/webrtc-calls.svelte.ts';
import { cryptoStore, decryptMessage, hasKeys } from '$lib/cryptoStore.svelte';

// Svelte writable store for messages — proper cross-file reactivity
export const messagesStore = writable<ChatMessage[]>([]);

// ── Déchiffrement automatique quand le cryptoStore devient prêt ──────────
// Quand l'utilisateur se reconnecte après un rafraîchissement, les messages
// sont chargés chiffrés. Dès que cryptoStore.ready devient true, on déchiffre.
// FIX : Ne plus s'arrêter après la première tentative (race condition).
let _cryptoReadyInterval: ReturnType<typeof setInterval> | null = null;
let _cryptoReadyAttempts = 0;
const _CRYPTO_READY_MAX_ATTEMPTS = 600; // 10 minutes à 1s d'intervalle
const _FAILED_DECRYPT_IDS = new Set<string>(); // Messages définitivement en échec
let _messagesReloaded = false; // Indique si on a rechargé les messages serveur après restauration crypto
let _wasCryptoReady = false; // Track crypto ready state for reactive effect
// Track optimistic message updates to preserve them during reload
const _optimisticPlaintext = new Map<string, { content: string; encrypted: boolean }>();

async function _decryptAllIfReady(): Promise<void> {
  console.log('[Chat] _decryptAllIfReady called, cryptoStore.ready=', cryptoStore.ready, 'hasKeys=', hasKeys(), '_messagesReloaded=', _messagesReloaded);
  if (!cryptoStore.ready || !hasKeys()) { console.log('[Chat] cryptoStore not ready or no keys, returning'); return; }

  // Clear failed decrypt IDs on crypto ready — allows retry of previously failed messages
  if (_FAILED_DECRYPT_IDS.size > 0) {
    console.log('[Chat] Clearing _FAILED_DECRYPT_IDS (size:', _FAILED_DECRYPT_IDS.size, ')');
    _FAILED_DECRYPT_IDS.clear();
  }

  // Premier déclenchement après restauration crypto : recharger les messages depuis le serveur
  // pour restaurer les champs E2EE (nonce, sender_public_key) qui peuvent avoir été perdus
  if (!_messagesReloaded) {
    _messagesReloaded = true;
    console.log('[Chat] First decryptAllIfReady - reloading messages from server');
    const convId = typeof window !== 'undefined' ? new URLSearchParams(window.location.search).get('conv') || 'default_global' : 'default_global';
    await loadMessages(convId);
    // After reload, re-apply optimistic plaintext messages that may have been overwritten
    if (_optimisticPlaintext.size > 0) {
      const msgs = get(messagesStore);
      let changed = false;
      for (let i = 0; i < msgs.length; i++) {
        const opt = _optimisticPlaintext.get(msgs[i].id);
        if (opt && msgs[i].encrypted && !opt.encrypted) {
          msgs[i] = { ...msgs[i], content: opt.content, encrypted: opt.encrypted };
          changed = true;
        }
      }
      if (changed) messagesStore.set([...msgs]);
      _optimisticPlaintext.clear();
    }
    return;
  }

  const msgs = get(messagesStore);
  const encrypted = msgs.filter(m => m.encrypted && m.nonce && m.sender_public_key && !_FAILED_DECRYPT_IDS.has(m.id));
  console.log('[Chat] _decryptAllIfReady: found', encrypted.length, 'encrypted messages to decrypt (failed:', _FAILED_DECRYPT_IDS.size, ')');

  try {
    for (const msg of encrypted) {
      try {
        msg.content = await decryptMessage({
          messageId: msg.id, conversationId: msg.conversation_id,
          ciphertext: msg.content, nonce: msg.nonce!,
          senderPubkeyB64: msg.sender_public_key || '',
          senderKeyVersion: msg.sender_key_version || undefined,
          groupKeyVersion: msg.group_key_version || undefined,
        });
        msg.encrypted = false;
        console.log('[Chat] Decrypt SUCCESS (2nd pass) for msg', msg.id.slice(0,8));
      } catch (e) {
        console.error('[Chat] Erreur déchiffrement message', msg.id, e);
        _FAILED_DECRYPT_IDS.add(msg.id);
        msg.content = '🔒 Message chiffré (clé indisponible)';
        msg.encrypted = false;
        msg.nonce = null;
        msg.sender_public_key = null;
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

// Exporté pour initialisation côté composant (chat/+page.svelte) — évite les $effect orphelins
// au chargement du module lors de l'import dynamique dans le root layout.
export function initCryptoListener(): void {
  _setupCryptoReadyListener();
}

// Other state as $state (less critical for cross-file reactivity)
export const chatStore = $state<Omit<ChatState, 'messages'>>({
  connectionError: null,
  showEmojiPicker: false,
  hasMore: false,
  loadingMore: false,
  unreadCounts: {},
  wsConnected: false,
  lastReactionUpdate: null,
  ws: null,
  participants: new Map<string, Participant[]>(),
  latencyMs: null,
});

// -----------------------------------------------------------------
// 3️⃣ WebSocket — temps réel avec reconnexion automatique
// Heartbeat constants
const WS_PING_INTERVAL_MS = 25_000; // Send ping if no message for 25s
const WS_PONG_TIMEOUT_MS = 5_000;   // Wait for pong (browser auto-handles)

// -----------------------------------------------------------------

let _ws: WebSocket | null = null;
let _wsConvId: string | null = null;
let _wsReconnectTimer: ReturnType<typeof setTimeout> | null = null;
let _wsRetries = 0;
const WS_MAX_RETRIES = 8;
let _wsHeartbeatTimer: ReturnType<typeof setTimeout> | null = null;
let _wsLastMessageTime = 0;
let _wsPingSentTime = 0; // timestamp when last app-level ping was sent, for latency calculation

export function connectWs(convId: string): void {
  if (typeof window === 'undefined') return;
  console.log('[chatStore] connectWs called:', convId, '_ws:', _ws, 'readyState:', _ws?.readyState, 'OPEN:', WebSocket.OPEN);
  _wsConvId = convId;
  
  // Clean up stale connections (CLOSED, CLOSING, CONNECTING)
  if (_ws && _ws.readyState !== WebSocket.OPEN) {
    console.log('[chatStore] connectWs: cleaning up stale WS, readyState:', _ws.readyState);
    _ws.onclose = null;
    _ws.close();
    _ws = null;
  }
  
  if (_ws?.readyState === WebSocket.OPEN) {
    console.log('[chatStore] connectWs: WS already OPEN, returning');
    return;
  }
  
  console.log('[chatStore] connectWs: calling _openWs');
  _openWs();
}

function _openWs(): void {
  if (_ws) { _ws.onclose = null; _ws.close(); _ws = null; }
  if (typeof window === 'undefined') return;

  const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${window.location.host}/api/webrtc/ws`);
  _ws = ws;
  chatStore.ws = ws;

  ws.onopen = () => {
    chatStore.wsConnected = true;
    _wsRetries = 0;
    if (_wsReconnectTimer) { clearTimeout(_wsReconnectTimer); _wsReconnectTimer = null; }
    
    // Initialize heartbeat timer
    _wsLastMessageTime = Date.now();
    _startHeartbeat();
  };

  ws.onmessage = (ev) => {
    // Update last message time on any received message
    _wsLastMessageTime = Date.now();
    try { _handleWsMessage(JSON.parse(ev.data as string)); } catch { /* non-JSON ok */ }
  };

  ws.onerror = () => {};

  ws.onclose = () => {
    chatStore.wsConnected = false;
    chatStore.ws = null;
    chatStore.latencyMs = null;
    _wsPingSentTime = 0;
    _stopHeartbeat();
    if (_wsRetries < WS_MAX_RETRIES) {
      const delay = Math.min(1000 * 2 ** _wsRetries, 30_000);
      _wsRetries++;
      _wsReconnectTimer = setTimeout(_openWs, delay);
    }
  };
}

function _startHeartbeat(): void {
  _stopHeartbeat();
  _wsHeartbeatTimer = setInterval(() => {
    if (!_ws || _ws.readyState !== WebSocket.OPEN) {
      _stopHeartbeat();
      return;
    }
    const now = Date.now();
    // Send ping if no message received for WS_PING_INTERVAL_MS
    if (now - _wsLastMessageTime >= WS_PING_INTERVAL_MS) {
      console.log('[chatStore] Heartbeat: sending ping');
      _wsPingSentTime = now;
      _ws.send(JSON.stringify({ type: 'ping' }));
    }
  }, 5000); // Check every 5 seconds
}

function _stopHeartbeat(): void {
  if (_wsHeartbeatTimer) {
    clearInterval(_wsHeartbeatTimer);
    _wsHeartbeatTimer = null;
  }
}

function _handleWsMessage(msg: Record<string, unknown>): void {
  const type = msg.type as string | undefined;

  // Heartbeat: respond to ping with pong
  if (type === 'ping') {
    if (_ws && _ws.readyState === WebSocket.OPEN) {
      _ws.send(JSON.stringify({ type: 'pong' }));
    }
    return;
  }

  if (type === 'pong') {
    // Pong received - calculate round-trip latency
    if (_wsPingSentTime > 0) {
      const rtt = Date.now() - _wsPingSentTime;
      chatStore.latencyMs = rtt;
      _wsPingSentTime = 0;
      console.log(`[chatStore] Pong received, latency: ${rtt}ms`);
    }
    return;
  }

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
    notifyPoll('📊 Sondage créé', title);
    return;
  }

  if (type === 'poll_voted') {
    const unknownUser = 'Quelqu\'un';
    const voter = (msg.voter as string) || unknownUser;
    notifyPoll('🗳️ Nouveau vote', `${voter} a voté`);
    return;
  }

  if (type === 'poll_closed') {
      notifyPoll('📊 Sondage fermé', 'Un sondage est terminé');
      return;
    }

    // ── Calendar notifications ──
            if (type === 'new_event') {
              const title = msg.title as string || 'Nouvel événement';
              const creator = msg.creator as string || "Quelqu'un";
              notifyCalendar('📅 Événement créé', `${creator}: ${title}`);
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
      notifyAdmin('✅ Utilisateur approuvé', 'Votre compte a été approuvé !');
      return;
    }
  }

  async function _injectMessage(raw: ChatMessage): Promise<void> {
    // Check if message already exists and is already plaintext (optimistic update)
    const currentMsgs = get(messagesStore);
    const existing = currentMsgs.findIndex(m => m.id === raw.id);
    // If existing message is already plaintext, skip encrypted WS update
    if (existing !== -1 && !currentMsgs[existing].encrypted && raw.encrypted) {
      console.debug('[WS] Skip encrypted update for plaintext msg', raw.id);
      return;
    }
    if (raw.encrypted && raw.nonce) {
      try {
        const { cryptoStore: cs, decryptMessage } = await import('$lib/cryptoStore.svelte');
        console.log('[WS] _injectMessage: cs.ready=', cs.ready, 'msg=', raw.id.slice(0,8));
        if (cs.ready) {
          raw.content = await decryptMessage({
            messageId: raw.id, conversationId: raw.conversation_id,
            ciphertext: raw.content, nonce: raw.nonce!,
            senderPubkeyB64: raw.sender_public_key || '',
            senderKeyVersion: raw.sender_key_version || undefined,
            groupKeyVersion: raw.group_key_version || undefined,
          });
          raw.encrypted = false;
          console.log('[WS] Decrypt SUCCESS for msg', raw.id.slice(0,8));
        } else {
          // Crypto not ready — set placeholder but KEEP encrypted fields for retry
          console.log('[WS] Crypto not ready, setting placeholder for msg', raw.id.slice(0,8));
          raw.content = '🔒 Message chiffré (clé indisponible)';
          // DO NOT set raw.encrypted = false — _decryptAllIfReady will retry
        }
      } catch (e) {
        console.error('[WS] Decrypt error for msg', raw.id.slice(0,8), e);
        _FAILED_DECRYPT_IDS.add(raw.id);
        raw.content = '🔒 Message chiffré (clé indisponible)';
        // DO NOT null encrypted fields — _decryptAllIfReady will retry when crypto ready
      }
    }
    // ADR-017: reconstruire reply_to depuis les champs plats du serveur
    const rawAny = raw as unknown as Record<string, unknown>;
    if (rawAny['reply_to_id'] !== undefined) {
      raw.reply_to = buildReplyTo(rawAny);
    }
    // Re-check store state after await to avoid duplicate messages from optimistic update
    const latestMsgs = get(messagesStore);
    const latestIdx = latestMsgs.findIndex(m => m.id === raw.id);
    if (latestIdx === -1) {
      messagesStore.update(msgs => [...msgs, raw]);
    } else {
      // Only overwrite if the current version is still encrypted (don't clobber plaintext optimistic update)
      if (latestMsgs[latestIdx].encrypted && !raw.encrypted) {
        messagesStore.update(msgs => { msgs[latestIdx] = raw; return [...msgs]; });
      } else if (!latestMsgs[latestIdx].encrypted) {
        console.debug('[WS] Skipping WS update — plaintext message already in store', raw.id.slice(0,8));
      }
    }
  }

export function disconnectWs(): void {
  if (_wsReconnectTimer) { clearTimeout(_wsReconnectTimer); _wsReconnectTimer = null; }
  _stopHeartbeat();
  if (_ws) { _ws.onclose = null; _ws.close(); _ws = null; }
  chatStore.ws = null;
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
  _optimisticPlaintext.clear();
}

export function formatTimestamp(ts: number): string {
  const date = new Date(ts * 1000);
  const now  = new Date();
  const timeStr = date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  if (date.toDateString() === now.toDateString())
    return timeStr;
  if (date.getFullYear() === now.getFullYear())
    return `${date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} à ${timeStr}`;
  return `${date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })} à ${timeStr}`;
}

// -----------------------------------------------------------------
// 6️⃣bis ADR-017 — Reconstruit reply_to depuis les champs plats du serveur
// -----------------------------------------------------------------

/**
 * Le backend sert des champs plats reply_to_* (sqlx-friendly). Cette fonction
 * reconstitue l'objet ReplyToPreview attendu par l'UI.
 * Si reply_to_id est présent mais reply_to_sender_name est null → message cité supprimé.
 */
export function buildReplyTo(raw: Record<string, unknown>): ReplyToPreview | null {
  const id = (raw['reply_to_id'] as string | null) ?? null;
  if (!id) return null;
  const senderName = (raw['reply_to_sender_name'] as string | null) ?? null;
  // Message cité supprimé (SET NULL côté serveur) → preview nulle
  if (senderName === null && (raw['reply_to_content'] as string | null) === null) {
    return null;
  }
  return {
    id,
    sender_name: senderName,
    content: (raw['reply_to_content'] as string | null) ?? null,
    message_type: (raw['reply_to_message_type'] as string | null) ?? null,
    file_id: (raw['reply_to_file_id'] as string | null) ?? null,
    nonce: (raw['reply_to_nonce'] as string | null) ?? null,
    encrypted: Boolean(raw['reply_to_encrypted']),
  };
}

/** Applique buildReplyTo sur une liste de messages récupérés du serveur */
export function hydrateReplyTo(msgs: ChatMessage[]): void {
  for (const m of msgs) {
    const raw = m as unknown as Record<string, unknown>;
    if (raw['reply_to_id'] !== undefined) {
      m.reply_to = buildReplyTo(raw);
    }
  }
}

// -----------------------------------------------------------------
// 6️⃣ Déchiffrement batch
// -----------------------------------------------------------------

async function _decryptBatch(msgs: ChatMessage[]): Promise<ChatMessage[]> {
  try {
    const { cryptoStore: cs, decryptMessage } = await import('$lib/cryptoStore.svelte');
    console.log('[Chat] _decryptBatch called, cs.ready=', cs.ready, 'messages=', msgs.length);
    if (!cs.ready) { console.log('[Chat] cryptoStore not ready, skipping decrypt'); return msgs; }
    for (const msg of msgs) {
      if (msg.encrypted && msg.nonce && !_FAILED_DECRYPT_IDS.has(msg.id)) {
        try {
          console.log('[Chat] Attempting decrypt for msg', msg.id.slice(0,8), 'group_key_version:', msg.group_key_version);
          msg.content = await decryptMessage({
            messageId: msg.id, conversationId: msg.conversation_id,
            ciphertext: msg.content, nonce: msg.nonce!,
            senderPubkeyB64: msg.sender_public_key || '',
            senderKeyVersion: msg.sender_key_version || undefined,
            groupKeyVersion: msg.group_key_version || undefined,
          });
          msg.encrypted = false;
          console.log('[Chat] Decrypt SUCCESS for msg', msg.id.slice(0,8));
        } catch (e) {
          console.error('[Chat] decryptMessage error for msg', msg.id.slice(0,8), e);
          _FAILED_DECRYPT_IDS.add(msg.id);
        }
      }
    }
  } catch (e) { console.error('[Chat] _decryptBatch error:', e); }
  return msgs;
}

// -----------------------------------------------------------------
// 7️⃣ API — loadMessages (50 derniers)
// -----------------------------------------------------------------

const PAGE_SIZE = 50;

export async function loadMessages(conversationId: string): Promise<ChatMessage[]> {
  try {
    const url = new URL(`/api/conversations/${conversationId}/messages`, window.location.origin);
    url.searchParams.set('limit', PAGE_SIZE.toString());
    url.searchParams.set('order', 'desc');
    url.searchParams.set('_ts', Date.now().toString()); // cache-bust
    const res = await fetch(url.toString(), {
      credentials: 'include',
      cache: 'no-store',
      headers: { 'Cache-Control': 'no-store' }
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    const msgs: ChatMessage[] = Array.isArray(data) ? data : (data.messages ?? []);
    msgs.sort((a, b) => a.created_at - b.created_at);
    hydrateReplyTo(msgs);
    await _decryptBatch(msgs);
    messagesStore.set([...msgs]);
    chatStore.hasMore  = msgs.length >= PAGE_SIZE;
    chatStore.connectionError = null;
    // APRÈS chargement, si crypto prêt → déchiffrer (race condition)
    if (cryptoStore.ready) _decryptAllIfReady();
    return msgs;
  } catch (err) {
    chatStore.connectionError = 'Erreur de chargement des messages';
    console.error('[Chat] loadMessages:', err);
    return [];
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
    hydrateReplyTo(older);
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

export async function sendMessage(
  content: string,
  conversationId: string,
  replyToId?: string | null
): Promise<ChatMessage | null> {
  if (!content.trim()) return null;
  try {
    const { cryptoStore: cs, encryptMessage } = await import('$lib/cryptoStore.svelte');
    let body: Record<string, unknown>;
    if (cs.ready) {
      try {
        const enc = await encryptMessage(content.trim(), conversationId);
        if ('group_key_version' in enc) {
          // Format nouveau : group key (default_global)
          body = { content: enc.ciphertext, encrypted: true, nonce: enc.nonce, group_key_version: enc.group_key_version };
        } else {
          // Format ancien : encrypted_keys (DMs, groupes normaux)
          body = { content: enc.ciphertext, encrypted: true, nonce: enc.nonce, encrypted_keys: enc.encryptedKeys };
        }
      } catch {
        body = { content: content.trim(), encrypted: false };
      }
    } else {
      body = { content: content.trim(), encrypted: false };
    }
    if (replyToId) {
      body.reply_to_id = replyToId;
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
    // Reconstruire reply_to depuis les champs plats du serveur si présent
    if (msgData.reply_to_id) {
      msgData.reply_to = buildReplyTo(msgData as unknown as Record<string, unknown>);
    }
    // Track optimistic plaintext update to preserve during crypto reload
    _optimisticPlaintext.set(msgData.id, { content: content.trim(), encrypted: false });
    // Always update store with plaintext — overwrites WS encrypted entry if race
    messagesStore.update(msgs => {
      const idx = msgs.findIndex(m => m.id === msgData.id);
      if (idx !== -1) { msgs[idx] = msgData; return [...msgs]; }
      return [...msgs, msgData];
    });
    // Le WebSocket apportera la confirmation — pas de reload nécessaire
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
  console.log('[editMessage] PATCH', convId, msgId, 'preview:', newContent.substring(0, 50));
  try {
    let body: Record<string, unknown>;
    if (cryptoStore.ready) {
      try {
        const { encryptMessage } = await import('$lib/cryptoStore.svelte');
        const enc = await encryptMessage(newContent.trim(), convId);
        body = { content: enc.ciphertext, encrypted: true, nonce: enc.nonce, encrypted_keys: enc.encryptedKeys };
      } catch { body = { content: newContent.trim(), encrypted: false }; }
    } else { body = { content: newContent.trim(), encrypted: false }; }
    const res = await fetch(`/api/conversations/${convId}/messages/${msgId}`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      credentials: 'include', body: JSON.stringify(body),
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

// -----------------------------------------------------------------
// 📦 Constants & internal exports
// -----------------------------------------------------------------

/** Max file size for server upload (50 MB) — larger files must use P2P */
export const MAX_BYTES_SERVER = 50 * 1024 * 1024;

/** Cancel an in-progress P2P file transfer */
export function cancelTransfer(fileId: string): void {
  // This is called from the chat page to cancel a transfer
  // The actual cancellation logic is in file-transfer.svelte.ts
  // We emit a custom event that the file-transfer module listens to
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('cancel-file-transfer', { detail: { fileId } }));
  }
}

/** Force re-check and decrypt all pending messages (exported for debug/manual retry) */
export async function triggerDecryptAllIfReady(): Promise<void> {
  return _decryptAllIfReady();
}
