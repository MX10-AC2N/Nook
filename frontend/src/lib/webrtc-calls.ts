import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import type { CallSignal } from './types';
import { authStore } from './authStore';

// Types
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

// État initial
const createInitialState = (): CallState => ({
  isCalling: false,
  isAnswering: false,
  isInCall: false,
  callType: 'audio',
  localStream: null,
  remoteStreams: new Map<string, MediaStream>(),
  peerConnections: new Map<string, RTCPeerConnection>(),
  currentConversationId: null,
  error: null,
  isMuted: false,
  isVideoOff: false
});

// Store pour l'état des appels
export const callStore = writable<CallState>(createInitialState());

class WebRTCCallManager {
  private ws: WebSocket | null = null;
  private localStream: MediaStream | null = null;
  private conversationId: string = '';
  private userId: string = '';
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private remoteStreams: Map<string, MediaStream> = new Map();
  private currentState: CallState = createInitialState();

  constructor() {
    const user = get(authStore).user;
    this.userId = user?.id || 'anonymous';
  }

  private updateState(partial: Partial<CallState>) {
    this.currentState = { ...this.currentState, ...partial };
    callStore.set(this.currentState);
  }

  private getState(): CallState {
    return get(callStore);
  }

  private async setupLocalStream(type: 'audio' | 'video'): Promise<MediaStream> {
    if (this.localStream) this.localStream.getTracks().forEach(track => track.stop());
    try {
      const constraints = { 
        audio: true, 
        video: type === 'video' ? { width: 1280, height: 720, frameRate: 30 } : false 
      };
      this.localStream = await navigator.mediaDevices.getUserMedia(constraints);
      this.updateTrackStates();
      return this.localStream;
    } catch (err: any) {
      throw new Error(`Impossible d'accéder à la ${type === 'video' ? 'caméra' : 'microphone'}: ${err.message}`);
    }
  }

  private createPeerConnection(remoteUserId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:stun.relay.metered.ca:80' }],
      iceCandidatePoolSize: 10
    });

    if (this.localStream) {
      this.localStream.getTracks().forEach(track => { pc.addTrack(track, this.localStream!); });
    }

    pc.onicecandidate = (event) => {
      if (event.candidate && this.ws?.readyState === WebSocket.OPEN) {
        this.sendSignal({ type: 'ice', to_user_id: remoteUserId, candidate: event.candidate });
      }
    };

    pc.ontrack = (event) => {
      const stream = event.streams[0];
      this.remoteStreams.set(remoteUserId, stream);
      const state = this.getState();
      const newStreams = new Map(state.remoteStreams);
      newStreams.set(remoteUserId, stream);
      this.updateState({ remoteStreams: newStreams });
    };

    pc.onconnectionstatechange = () => {
      if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected') {
        this.endCallForUser(remoteUserId);
      }
    };

    this.peerConnections.set(remoteUserId, pc);
    return pc;
  }

  private sendSignal(signal: Partial<CallSignal>) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN || !this.conversationId) return;
    const fullSignal: CallSignal = {
      conversationId: this.conversationId,
      from_user_id: this.userId,
      to_user_id: signal.to_user_id || null,
      type: signal.type || 'unknown',
      sdp: signal.sdp || null,
      candidate: signal.candidate || null,
      timestamp: Date.now()
    };
    this.ws.send(JSON.stringify(fullSignal));
  }

  private async handleSignal(signal: CallSignal) {
    if (signal.from_user_id === this.userId) return;
    switch (signal.type) {
      case 'offer': await this.handleOffer(signal); break;
      case 'answer': await this.handleAnswer(signal); break;
      case 'ice': await this.handleIceCandidate(signal); break;
      case 'join': await this.handleJoin(signal); break;
      case 'leave': this.handleLeave(signal); break;
      case 'decline': this.handleDecline(signal); break;
    }
  }

  private async handleOffer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id || !this.localStream) return;
    const pc = this.createPeerConnection(signal.from_user_id);
    try {
      await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp: signal.sdp }));
      const answer = await pc.createAnswer({ offerToReceiveAudio: true, offerToReceiveVideo: signal.sdp.includes('m=video') });
      await pc.setLocalDescription(answer);
      this.sendSignal({ type: 'answer', to_user_id: signal.from_user_id, sdp: answer.sdp });
      this.updateState({ isAnswering: false, isInCall: true });
    } catch (err) {
      console.error("Erreur lors de la gestion de l'offre:", err);
      this.endCallForUser(signal.from_user_id);
    }
  }

  private async handleAnswer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;
    const pc = this.peerConnections.get(signal.from_user_id);
    if (!pc) return;
    try {
      await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: signal.sdp }));
    } catch (err) {
      console.error("Erreur lors de la gestion de la réponse:", err);
      this.endCallForUser(signal.from_user_id);
    }
  }

  private async handleIceCandidate(signal: CallSignal) {
    if (!signal.candidate || !signal.from_user_id) return;
    const pc = this.peerConnections.get(signal.from_user_id);
    if (!pc) return;
    try {
      await pc.addIceCandidate(new RTCIceCandidate(signal.candidate));
    } catch (err) {
      console.error('Erreur ICE candidate:', err);
    }
  }

  private async handleJoin(signal: CallSignal) {
    const state = this.getState();
    if (signal.from_user_id === this.userId || !this.localStream) return;
    if (state.isInCall && !this.peerConnections.has(signal.from_user_id)) {
      await this.initiateCallWithUser(signal.from_user_id);
    }
  }

  private handleLeave(signal: CallSignal) { this.endCallForUser(signal.from_user_id); }
  private handleDecline(signal: CallSignal) { if (signal.from_user_id !== this.userId) this.endCallForUser(signal.from_user_id); }

  private updateTrackStates() {
    const state = this.getState();
    if (!this.localStream) return;
    this.localStream.getAudioTracks().forEach(track => { track.enabled = !state.isMuted; });
    this.localStream.getVideoTracks().forEach(track => { track.enabled = !state.isVideoOff; });
  }

  public async startCall(conversationId: string, participantIds: string[], type: 'audio' | 'video') {
    try {
      this.updateState({ isCalling: true, callType: type, currentConversationId: conversationId, error: null });
      this.conversationId = conversationId;
      this.localStream = await this.setupLocalStream(type);
      this.updateState({ localStream: this.localStream, isCalling: true });

      const wsUrl = `wss://${browser ? window.location.host : 'localhost:3000'}/ws/call?conv=${conversationId}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        this.sendSignal({ type: 'join' });
        const targets = participantIds.filter(id => id !== this.userId);
        if (targets.length === 0) {
          this.updateState({ isCalling: false, isInCall: true, error: 'Aucun participant à appeler' });
          return;
        }
        targets.forEach(userId => {
          this.initiateCallWithUser(userId).catch(err => {
            console.error(`Erreur appel à ${userId}:`, err);
            this.updateState({ error: `Impossible d'appeler ${userId}` });
          });
        });
      };

      this.ws.onmessage = (event) => {
        try {
          const signal = JSON.parse(event.data) as CallSignal;
          this.handleSignal(signal);
        } catch (err) {
          console.error('Erreur parsing signal:', err);
        }
      };

      this.ws.onerror = () => { this.updateState({ error: 'Erreur de connexion WebRTC' }); };
      this.ws.onclose = () => {
        console.log('WebSocket fermé');
        const state = this.getState();
        if (state.isInCall) this.endCall();
      };

    } catch (err: any) {
      this.updateState({ error: err instanceof Error ? err.message : 'Erreur inconnue', isCalling: false });
      this.endCall();
      throw err;
    }
  }

  private async initiateCallWithUser(remoteUserId: string) {
    const pc = this.createPeerConnection(remoteUserId);
    try {
      const offer = await pc.createOffer({ offerToReceiveAudio: true, offerToReceiveVideo: this.localStream?.getVideoTracks().length > 0 });
      await pc.setLocalDescription(offer);
      this.sendSignal({ type: 'offer', to_user_id: remoteUserId, sdp: offer.sdp });
    } catch (err) {
      console.error('Erreur création offre:', err);
      this.endCallForUser(remoteUserId);
    }
  }

  public async toggleMute() {
    const state = this.getState();
    this.updateState({ isMuted: !state.isMuted });
    const newState = this.getState();
    if (this.localStream) {
      this.localStream.getAudioTracks().forEach(track => { track.enabled = !newState.isMuted; });
    }
  }

  public async toggleVideo() {
    const state = this.getState();
    this.updateState({ isVideoOff: !state.isVideoOff });
    const newState = this.getState();
    if (this.localStream) {
      this.localStream.getVideoTracks().forEach(track => { track.enabled = !newState.isVideoOff; });
    }
  }

  public endCall() {
    this.peerConnections.forEach(pc => pc.close());
    this.peerConnections.clear();

    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.currentState = createInitialState();
    callStore.set(this.currentState);
  }

  private endCallForUser(remoteUserId: string) {
    const pc = this.peerConnections.get(remoteUserId);
    if (pc) {
      pc.close();
      this.peerConnections.delete(remoteUserId);
    }
    const state = this.getState();
    const newRemoteStreams = new Map(state.remoteStreams);
    newRemoteStreams.delete(remoteUserId);
    this.updateState({ remoteStreams: newRemoteStreams });

    if (this.peerConnections.size === 0 && state.isInCall) {
      this.endCall();
    }
  }

}

export const callManager = new WebRTCCallManager();

export async function startGroupCall(conversationId: string, participantIds: string[], type: 'audio' | 'video' = 'audio') {
  await callManager.startCall(conversationId, participantIds, type);
}

export function endCurrentCall() { callManager.endCall(); }

export function getCallState(): CallState { return get(callStore); }
