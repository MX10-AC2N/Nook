// frontend/src/lib/webrtc-calls.ts
import { writable } from 'svelte/store';
import type { CallSignal } from '$lib/types';

interface CallState {
  isCalling: boolean;
  isInCall: boolean;
  callType: 'audio' | 'video';
  localStream: MediaStream | null;
  remoteStreams: Map<string, MediaStream>; // user_id -> stream
  peerConnections: Map<string, RTCPeerConnection>;
  currentConversationId: string | null;
  error: string | null;
}

export const callStore = writable<CallState>({
  isCalling: false,
  isInCall: false,
  callType: 'audio',
  localStream: null,
  remoteStreams: new Map(),
  peerConnections: new Map(),
  currentConversationId: null,
  error: null
});

class WebRTCCallManager {
  private ws: WebSocket | null = null;
  private localStream: MediaStream | null = null;
  private conversationId: string = '';
  private userId: string = '';

  constructor() {
    const auth = (window as any).authStore?.user;
    this.userId = auth?.id || 'anonymous';
  }

  private async setupLocalStream(type: 'audio' | 'video'): Promise<MediaStream> {
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
    }

    this.localStream = await navigator.mediaDevices.getUserMedia({
      audio: true,
      video: type === 'video'
    });

    return this.localStream;
  }

  private createPeerConnection(remoteUserId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
    });

    // Ajouter les tracks locaux
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => {
        pc.addTrack(track, this.localStream!);
      });
    }

    // Gérer les ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate && this.ws?.readyState === WebSocket.OPEN) {
        this.sendSignal({
          type: 'ice',
          to_user_id: remoteUserId,
          candidate: event.candidate
        });
      }
    };

    // Gérer les flux entrants
    pc.ontrack = (event) => {
      callStore.update(state => {
        const newStreams = new Map(state.remoteStreams);
        newStreams.set(remoteUserId, event.streams[0]);
        return { ...state, remoteStreams: newStreams };
      });
    };

    // Gestion des erreurs
    pc.onconnectionstatechange = () => {
      if (pc.connectionState === 'failed') {
        this.endCallForUser(remoteUserId);
      }
    };

    return pc;
  }

  private sendSignal(signal: Partial<CallSignal>) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;

    const fullSignal: CallSignal = {
      conversationId: this.conversationId,
      from_user_id: this.userId,
      to_user_id: signal.to_user_id || null,
      type: signal.type || 'unknown',
      sdp: signal.sdp || null,
      candidate: signal.candidate || null,
      ...signal
    };

    this.ws.send(JSON.stringify(fullSignal));
  }

  private async handleSignal(signal: CallSignal) {
    if (signal.from_user_id === this.userId) return;

    switch (signal.type) {
      case 'offer':
        await this.handleOffer(signal);
        break;
      case 'answer':
        await this.handleAnswer(signal);
        break;
      case 'ice':
        await this.handleIceCandidate(signal);
        break;
      case 'join':
        if (this.callStoreValue.isInCall && signal.from_user_id !== this.userId) {
          this.initiateCallWithUser(signal.from_user_id);
        }
        break;
      case 'leave':
        this.endCallForUser(signal.from_user_id);
        break;
    }
  }

  private async handleOffer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;

    const pc = this.createPeerConnection(signal.from_user_id);
    
    await pc.setRemoteDescription(new RTCSessionDescription({
      type: 'offer',
      sdp: signal.sdp
    }));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    this.sendSignal({
      type: 'answer',
      to_user_id: signal.from_user_id,
      sdp: answer.sdp
    });

    callStore.update(state => {
      const newPCs = new Map(state.peerConnections);
      newPCs.set(signal.from_user_id, pc);
      return { ...state, peerConnections: newPCs };
    });
  }

  private async handleAnswer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;

    const pc = this.peerConnections.get(signal.from_user_id);
    if (!pc) return;

    await pc.setRemoteDescription(new RTCSessionDescription({
      type: 'answer',
      sdp: signal.sdp
    }));
  }

  private async handleIceCandidate(signal: CallSignal) {
    if (!signal.candidate || !signal.from_user_id) return;

    const pc = this.peerConnections.get(signal.from_user_id);
    if (!pc) return;

    try {
      await pc.addIceCandidate(new RTCIceCandidate(signal.candidate as RTCIceCandidateInit));
    } catch (err) {
      console.error('Erreur ICE candidate:', err);
    }
  }

  private get peerConnections(): Map<string, RTCPeerConnection> {
    let pcs = new Map<string, RTCPeerConnection>();
    callStore.update(state => {
      pcs = state.peerConnections;
      return state;
    });
    return pcs;
  }

  private get callStoreValue(): CallState {
    let state: CallState;
    callStore.subscribe(s => state = s)();
    return state;
  }

  public async startCall(conversationId: string, type: 'audio' | 'video', participants: string[]) {
    try {
      this.conversationId = conversationId;
      const stream = await this.setupLocalStream(type);

      callStore.set({
        isCalling: true,
        isInCall: true,
        callType: type,
        localStream: stream,
        remoteStreams: new Map(),
        peerConnections: new Map(),
        currentConversationId: conversationId,
        error: null
      });

      // Connexion WebSocket
      const wsUrl = `ws://${window.location.host}/ws/call?conv=${conversationId}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        // Envoyer un signal de join explicite
        this.sendSignal({ type: 'join' });
        
        // Initier les appels avec tous les participants
        participants
          .filter(id => id !== this.userId)
          .forEach(userId => this.initiateCallWithUser(userId));
      };

      this.ws.onmessage = (event) => {
        try {
          const signal = JSON.parse(event.data) as CallSignal;
          this.handleSignal(signal);
        } catch (err) {
          console.error('Erreur parsing signal:', err);
        }
      };

      this.ws.onerror = (error) => {
        callStore.update(state => ({
          ...state,
          error: 'Erreur de connexion WebRTC'
        }));
      };

      this.ws.onclose = () => {
        this.endCall();
      };

    } catch (err) {
      callStore.update(state => ({
        ...state,
        error: err instanceof Error ? err.message : 'Erreur inconnue'
      }));
      this.endCall();
    }
  }

  private async initiateCallWithUser(remoteUserId: string) {
    const pc = this.createPeerConnection(remoteUserId);
    
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);

    this.sendSignal({
      type: 'offer',
      to_user_id: remoteUserId,
      sdp: offer.sdp
    });

    callStore.update(state => {
      const newPCs = new Map(state.peerConnections);
      newPCs.set(remoteUserId, pc);
      return { ...state, peerConnections: newPCs };
    });
  }

  public endCallForUser(userId: string) {
    const pcs = this.peerConnections;
    const pc = pcs.get(userId);

    if (pc) {
      pc.close();
      pcs.delete(userId);
    }

    callStore.update(state => {
      const newStreams = new Map(state.remoteStreams);
      const newPCs = new Map(state.peerConnections);
      
      newStreams.delete(userId);
      newPCs.delete(userId);
      
      // Si plus personne dans l'appel, terminer complètement
      if (newStreams.size === 0 && newPCs.size === 0) {
        this.endCall();
      }
      
      return { 
        ...state, 
        remoteStreams: newStreams,
        peerConnections: newPCs 
      };
    });
  }

  public endCall() {
    // Fermer toutes les connexions
    this.peerConnections.forEach(pc => pc.close());
    
    // Arrêter le stream local
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }
    
    // Fermer WebSocket
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    
    // Réinitialiser l'état
    callStore.set({
      isCalling: false,
      isInCall: false,
      callType: 'audio',
      localStream: null,
      remoteStreams: new Map(),
      peerConnections: new Map(),
      currentConversationId: null,
      error: null
    });
  }
}

// Singleton
export const callManager = new WebRTCCallManager();

// Exposer des fonctions simples pour les composants
export async function startGroupCall(conversationId: string, participants: string[], type: 'audio' | 'video' = 'audio') {
  await callManager.startCall(conversationId, type, participants);
}

export function endCurrentCall() {
  callManager.endCall();
}