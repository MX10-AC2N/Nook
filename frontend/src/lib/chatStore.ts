// Types communs pour le chat — exportés depuis un module .ts pur
// (évite les problèmes de résolution de module avec .svelte.ts)

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
  /** WebSocket instance for direct access */
  ws: WebSocket | null;
  /** Participants cache per conversation */
  participants: Map<string, Participant[]>;
}

export interface Participant {
  id: string;
  username: string;
  name: string | null;
  role: string;
}

// Ré-export depuis le store Svelte (runtime)
export {
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
} from './chatStore.svelte';