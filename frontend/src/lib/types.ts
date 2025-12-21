// frontend/src/lib/types.ts

// Types pour les conversations et messages
export interface Participant {
  id: string;
  name: string;
  username: string;
  role: 'admin' | 'member';
  approved: boolean;
  public_key?: string;
}

export interface Conversation {
  id: string;
  name: string | null;
  is_group: boolean;
  created_at: string;
  last_message_at: string;
  last_message_preview: string;
  unread_count: number;
  participants: Participant[];
}

export interface Reaction {
  [emoji: string]: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  sender_name: string;
  content: number[]; // Tableau d'octets pour le contenu chiffré
  encrypted_keys: Record<string, number[]>; // { user_id: clé chiffrée }
  nonce: number[];
  media_type: 'gif' | 'audio' | 'video' | null;
  media_url: string | null;
  timestamp: string;
  reactions: Reaction;
}

export interface DecryptedMessage extends Message {
  decryptedContent: string;
  senderName: string;
  isCurrentUser: boolean;
}

export interface EncryptedMessage {
  encryptedContent: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>;
  nonce: Uint8Array;
}

// Types pour WebRTC
export interface CallSignal {
  conversationId: string;
  from_user_id: string;
  to_user_id: string | null;
  type: string; // 'offer' | 'answer' | 'ice' | 'join' | 'leave'
  sdp: string | null;
  candidate: any | null; // RTCIceCandidateInit
}

export interface CallState {
  isCalling: boolean;
  isInCall: boolean;
  isAnswering: boolean;
  callType: 'audio' | 'video';
  localStream: MediaStream | null;
  remoteStreams: Map<string, MediaStream>;
  peerConnections: Map<string, RTCPeerConnection>;
  currentConversationId: string | null;
  error: string | null;
  isMuted: boolean;
  isVideoOff: boolean;
}