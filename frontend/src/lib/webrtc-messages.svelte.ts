// frontend/src/lib/webrtc-messages.svelte.ts
// ════════════════════════════════════════════════════════════
// Shared WebSocket connection manager for Nook
// Replaces duplicate WS code in chatStore, chessStore, 
// conversationStore, and webrtc-calls
// ════════════════════════════════════════════════════════════

import { browser } from '$app/environment';

// ────────────────────────────────────────────────────────────
// Types
// ────────────────────────────────────────────────────────────
export type MessageHandler = (msg: Record<string, unknown>) => void;

// ────────────────────────────────────────────────────────────
// State
// ────────────────────────────────────────────────────────────
// Reactive connection state
export const wsConnected = $state(false);
export const wsReconnecting = $state(false);

let ws: WebSocket | null = null;
let _wsRetries = 0;
let _wsTimer: ReturnType<typeof setTimeout> | null = null;
const MAX_RETRIES = 12;
const MAX_BACKOFF = 30_000; // 30 seconds

// Message handlers registry: type -> handlers
const handlers: Map<string, Set<MessageHandler>> = new Map();

// Global handlers (catch-all for unregistered types)
const globalHandlers: Set<MessageHandler> = new Set();

// Current context for filtering (gameId, convId)
let currentGameId: string | null = null;
let currentConvId: string | null = null;

// ────────────────────────────────────────────────────────────
// Registration API
// ────────────────────────────────────────────────────────────

/**
 * Register a handler for a specific message type.
 * Each handler is called for every message matching its type.
 */
export function onMessageType(type: string, handler: MessageHandler): void {
  if (!handlers.has(type)) {
    handlers.set(type, new Set());
  }
  handlers.get(type)!.add(handler);
}

/**
 * Register a catch-all handler (called for ALL messages).
 */
export function onAnyMessage(handler: MessageHandler): void {
  globalHandlers.add(handler);
}

/**
 * Remove a specific handler.
 */
export function removeHandler(type: string, handler: MessageHandler): boolean {
  const set = handlers.get(type);
  if (set) {
    return set.delete(handler);
  }
  return false;
}

// ────────────────────────────────────────────────────────────
// Context API
// ────────────────────────────────────────────────────────────

/**
 * Set the current game context (for chess message filtering).
 * Messages for other games are silently ignored.
 */
export function setContextGameId(gameId: string | null): void {
  currentGameId = gameId;
}

/**
 * Set the current conversation context (for chat message filtering).
 */
export function setContextConvId(convId: string | null): void {
  currentConvId = convId;
}

// ────────────────────────────────────────────────────────────
// Connection Management
// ────────────────────────────────────────────────────────────

/**
 * Connect the WebSocket. Creates a single shared connection.
 * Subsequent calls return early if already connected.
 */
export function connect(): void {
  if (!browser) return;
  if (ws?.readyState === WebSocket.OPEN) return;
  
  disconnect();
  _connect(window.location.host);
}

/**
 * Disconnect and clean up all resources.
 */
export function disconnect(): void {
  if (_wsTimer) {
    clearTimeout(_wsTimer);
    _wsTimer = null;
  }
  if (ws) {
    ws.onclose = null;
    ws.onerror = null;
    ws.close();
    ws = null;
  }
  wsConnected = false;
  wsReconnecting = false;
  _wsRetries = 0;
}

/**
 * Get current connection status.
 */
export function isConnected(): boolean {
  return ws?.readyState === WebSocket.OPEN;
}

// ────────────────────────────────────────────────────────────
// Internal Implementation
// ────────────────────────────────────────────────────────────

function _connect(host: string): void {
  if (!browser || !host) return;
  
  const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const newWs = new WebSocket(`${proto}://${host}/ws`);
  ws = newWs;
  
  newWs.onopen = () => {
    wsConnected = true;
    wsReconnecting = false;
    _wsRetries = 0;
    if (_wsTimer) {
      clearTimeout(_wsTimer);
      _wsTimer = null;
    }
  };
  
  newWs.onmessage = (ev) => {
    try {
      const msg = JSON.parse(ev.data as string) as Record<string, unknown>;
      const type = msg.type as string | undefined;
      
      // Global handlers (always called)
      for (const handler of globalHandlers) {
        handler(msg);
      }
      
      // Type-specific handlers
      if (type) {
        const set = handlers.get(type);
        if (set) {
          for (const handler of set) {
            handler(msg);
          }
        }
      }
    } catch {
      // Non-JSON messages ignored
    }
  };
  
  newWs.onerror = () => {
    // Error details not available in browser WS API
  };
  
  newWs.onclose = () => {
    wsConnected = false;
    if (_wsRetries < MAX_RETRIES) {
      wsReconnecting = true;
      const delay = Math.min(1000 * 2 ** _wsRetries, MAX_BACKOFF);
      _wsRetries++;
      _wsTimer = setTimeout(() => _connect(host), delay);
    } else {
      wsReconnecting = false;
    }
  };
}
