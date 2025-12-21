// frontend/src/lib/types.ts

// Types pour les utilisateurs
export interface User {
  id: string;
  name: string;
  username: string;
  role: 'admin' | 'member';
  approved: boolean;
  createdAt: number;
  publicKey?: string;
  privateKeyEncrypted?: string;
}

// Types pour les conversations
export interface Participant {
  id: string;
  name: string;
  username: string;
  role: 'admin' | 'member';
  approved: boolean;
  publicKey?: string;
}

export interface Conversation {
  id: string;
  name: string | null;
  is_group: boolean;
  created_at: number;
  last_message_at: number;
  last_message_preview: string;
  unread_count: number;
  participants: Participant[];
}

// Types pour les messages
export interface Reaction {
  [emoji: string]: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  sender_name: string;
  content: string; // Contenu chiffré (base64)
  encrypted_keys: Record<string, string>; // { user_id: clé_chiffrée_base64 }
  nonce: string; // base64
  media_type: 'text' | 'gif' | 'audio' | 'video' | null;
  media_url: string | null;
  duration: number | null; // secondes
  timestamp: number;
  reactions: Reaction;
}

export interface DecryptedMessage extends Message {
  decryptedContent: string;
  decryptedMediaUrl?: string;
}

// Types pour le chiffrement
export interface EncryptedData {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>;
}

export interface KeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

// Types pour WebRTC
export interface CallSignal {
  conversationId: string;
  from_user_id: string;
  to_user_id: string | null;
  type: 'offer' | 'answer' | 'ice' | 'join' | 'leave' | 'decline';
  sdp: string | null;
  candidate: RTCIceCandidateInit | null;
  timestamp: number;
}

export interface CallState {
  isCalling: boolean;
  isAnswering: boolean;
  isInCall: boolean;
  callType: 'audio' | 'video';
  localStream: MediaStream | null;
  remoteStreams: Map<string, MediaStream>;
  peerConnections: Map<string, RTCPeerConnection>;
  currentConversationId: string | null;
  error: string | null;
  isMuted: boolean;
  isVideoOff: boolean;
}

// Types pour le stockage
export interface StoredKeys {
  encryptedPrivateKey: string; // base64
  publicKey: Uint8Array;
}

// Types pour les stores Svelte
export interface AuthState {
  isAuthenticated: boolean;
  isAdmin: boolean;
  user: User | null;
  loading: boolean;
}

export interface ThemeState {
  currentTheme: 'jardin-secret' | 'space-hub' | 'maison-chaleureuse';
  themes: {
    'jardin-secret': Record<string, string>;
    'space-hub': Record<string, string>;
    'maison-chaleureuse': Record<string, string>;
  };
}

export interface ConnectionError {
  message: string;
  timestamp: number;
}

// Types pour les médias
export interface RecordingState {
  isRecording: boolean;
  mediaType: 'audio' | 'video' | null;
  stream: MediaStream | null;
  recorder: MediaRecorder | null;
  chunks: Blob[];
  duration: number;
  startTime: number;
}

export interface MediaUpload {
  id: string;
  conversationId: string;
  mediaType: 'audio' | 'video';
  file: Blob;
  duration: number;
  recipientPublicKeys: Uint8Array[];
  senderPrivateKey: Uint8Array;
  status: 'pending' | 'uploading' | 'success' | 'error';
  progress: number;
  error?: string;
}

// Types pour les API responses
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: number;
}

export interface LoginResponse {
  success: boolean;
  user?: User;
  token?: string;
}

export interface ConversationResponse {
  success: boolean;
  conversation?: Conversation;
  error?: string;
}

export interface MessageResponse {
  success: boolean;
  message?: Message;
  error?: string;
}

// Types utilitaires
export type DeepPartial<T> = {
  [P in keyof T]?: DeepPartial<T[P]>;
};

export type WritableDeep<T> = {
  [P in keyof T]: T[P] extends object ? WritableDeep<T[P]> : T[P];
};

export type ThemeVariables = {
  [key: string]: string;
};

// Constantes
export const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50 Mo
export const MAX_DURATION = 60 * 10; // 10 minutes
export const MAX_PARTICIPANTS = 6; // Limite raisonnable pour appels de groupe

// Enums pour les états
export enum CallStatus {
  IDLE = 'idle',
  CALLING = 'calling',
  IN_CALL = 'in_call',
  ANSWERING = 'answering',
  ERROR = 'error'
}

export enum MessageType {
  TEXT = 'text',
  GIF = 'gif',
  AUDIO = 'audio',
  VIDEO = 'video',
  REACTION = 'reaction'
}

export enum ThemeName {
  JARDIN_SECRET = 'jardin-secret',
  SPACE_HUB = 'space-hub',
  MAISON_CHALEUREUSE = 'maison-chaleureuse'
}

// Types pour les événements personnalisés
export interface CustomEventMap {
  'incoming-call': CustomEvent<{
    from_user_id: string;
    conversation_id: string;
  }>;
  'message-sent': CustomEvent<Message>;
  'message-received': CustomEvent<Message>;
  'call-ended': CustomEvent<void>;
  'theme-changed': CustomEvent<{
    theme: ThemeName;
    variables: ThemeVariables;
  }>;
}

// Extension de Window pour les événements personnalisés
declare global {
  interface WindowEventMap extends CustomEventMap {}
}

// Export par défaut pour la compatibilité
export default {
  User,
  Participant,
  Conversation,
  Message,
  Reaction,
  DecryptedMessage,
  EncryptedData,
  KeyPair,
  CallSignal,
  CallState,
  StoredKeys,
  AuthState,
  ThemeState,
  ConnectionError,
  RecordingState,
  MediaUpload,
  ApiResponse,
  LoginResponse,
  ConversationResponse,
  MessageResponse,
  MAX_FILE_SIZE,
  MAX_DURATION,
  MAX_PARTICIPANTS,
  CallStatus,
  MessageType,
  ThemeName
};