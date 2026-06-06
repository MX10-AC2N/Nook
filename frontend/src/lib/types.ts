// src/lib/types.ts

/*=====================================================================
  USER & PARTICIPANT
=====================================================================*/
/**
 * Représente un utilisateur enregistré dans la base.
 */
export interface User {
  id: string;
  name: string;
  username: string;
  role: 'admin' | 'member';
  approved: boolean;
  createdAt: number;               // timestamp (seconds depuis epoch)
  publicKey?: string;              // clé publique (base64 ou hex)
  privateKeyEncrypted?: string;    // clé privée chiffrée (base64)
}

/**
 * Participant d’une conversation (peut être le même type que `User` mais
 * on le garde séparé pour éviter les dépendances circulaires).
 */
export interface Participant {
  id: string;
  name: string;
  username: string;
  role: 'admin' | 'member';
  approved: boolean;
  publicKey?: string;
}

/*=====================================================================
  CONVERSATION
=====================================================================*/
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

/*=====================================================================
  MESSAGE & REACTION
=====================================================================*/
export interface Reaction {
  /** emoji → nombre d’occurrences */
  [emoji: string]: number;
}

/**
 * Message stocké côté serveur (chiffré si besoin).
 */
export interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  sender_name: string;
  content: string;                                 // texte brut (chiffré côté client)
  encrypted_keys: Record<string, Uint8Array>;       // { recipientId: nonce+encryptedKey }
  nonce: string;                                   // base64 (nonce symétrique)
  media_type: 'text' | 'gif' | 'audio' | 'video' | null;
  media_url: string | null;
  duration: number | null;                         // en secondes (audio/video)
  timestamp: number;                               // epoch seconds
  reactions: Reaction;
}

/**
 * Message déchiffré côté client (extension de `Message`).
 */
export interface DecryptedMessage extends Message {
  decryptedContent: string;
  decryptedMediaUrl?: string;
}

/*=====================================================================
  ENCRYPTION HELPERS
=====================================================================*/
export interface EncryptedData {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>;
}

export interface KeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

/*=====================================================================
  WEBRTC SIGNALING & STATE
=====================================================================*/
export interface CallSignal {
  conversationId: string;
  from_user_id: string;
  /** `null` → appel broadcast (ex. appel de groupe) */
  to_user_id: string | null;
  type:
    | 'offer' | 'answer' | 'ice' | 'ice_candidate'
    | 'webrtc_offer' | 'webrtc_answer' | 'webrtc_ice_candidate'
    | 'join' | 'leave' | 'decline'
    | 'call_request' | 'call_accepted' | 'call_rejected'
    // SFU signals
    | 'sfu_join' | 'sfu_answer' | 'sfu_renegotiate_offer' | 'sfu_peers' | 'sfu_error'
    // File transfer P2P signals
    | 'file-transfer-offer' | 'file-transfer-answer' | 'file-transfer-ice';
  sdp: string | null;
  candidate: RTCIceCandidateInit | null;
  timestamp: number;
  // Champs optionnels pour call_request (sonnerie)
  from_user_name?: string;
  callType?: 'audio' | 'video';
  // SFU optional fields
  answer?: string;
  peers?: string[];
  renegotiate_offer?: string;
  offer?: string;
  error?: string;
}

/**
 * État complet d'un appel WebRTC.
 */
export interface CallState {
  isCalling: boolean;
  isAnswering: boolean;
  isInCall: boolean;
  callType: 'audio' | 'video';
  localStream: MediaStream | null;
  remoteStreams: Map<string, MediaStream>;
  peerConnections: Map<string, RTCPeerConnection>;
  fileDataChannels: Map<string, RTCDataChannel>;
  currentConversationId: string | null;
  error: string | null;
  isMuted: boolean;
  isVideoOff: boolean;
  localVideoElement: HTMLVideoElement | null;
  isScreenSharing: boolean;
  screenShareStream: MediaStream | null;
  screenShareLocalVideoElement: HTMLVideoElement | null;
  // Call quality monitoring
  callQuality: 'good' | 'fair' | 'poor' | 'unknown';
  currentBitrate: number;
  packetsLost: number;
  jitter: number;
  rtt: number;
  remoteResolution: string | null;
  remoteFps: number;
  // SFU state
  useSfu: boolean;
  sfuAnswer: string | null;
  sfuRenegotiateOffer: string | null;
  sfuPeers: string[];
  sfuPendingOffer: string | null;
}

/*=====================================================================
  STORED KEYS (IndexedDB)
=====================================================================*/
export interface StoredKeys {
  encryptedPrivateKey: string;   // base64
  publicKey: Uint8Array;
}

/*=====================================================================
  AUTH STORE STATE
=====================================================================*/
export interface AuthState {
  isAuthenticated: boolean;
  isAdmin: boolean;
  user: User | null;
  loading: boolean;
}

/*=====================================================================
  THEME STORE STATE
=====================================================================*/
export interface ThemeState {
  currentTheme: 'jardin-secret' | 'space-hub' | 'maison-chaleureuse';
  themes: {
    'jardin-secret': Record<string, string>;
    'space-hub': Record<string, string>;
    'maison-chaleureuse': Record<string, string>;
  };
}

/*=====================================================================
  CONNECTION ERROR
=====================================================================*/
export interface ConnectionError {
  message: string;
  timestamp: number;
}

/*=====================================================================
  MEDIA RECORDING & UPLOAD
=====================================================================*/
export interface RecordingState {
  isRecording: boolean;
  mediaType: 'audio' | 'video' | null;
  stream: MediaStream | null;
  recorder: MediaRecorder | null;
  chunks: Blob[];
  duration: number;   // en secondes
  startTime: number;  // epoch ms
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
  progress: number;   // 0‑100
  error?: string;
}

/*=====================================================================
  API RESPONSE WRAPPERS
=====================================================================*/
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
}
