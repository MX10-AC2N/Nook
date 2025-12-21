// frontend/src/lib/types.ts
export interface CallSignal {
  conversationId: string;
  from_user_id: string;
  to_user_id: string | null;
  type: string; // 'offer' | 'answer' | 'ice' | 'join' | 'leave'
  sdp: string | null;
  candidate: any | null; // RTCIceCandidateInit
}