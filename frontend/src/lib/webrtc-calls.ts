// frontend/src/lib/webrtc-calls.ts
import { writable, get } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { CallSignal } from './types';
import { authStore } from './authStore';
import { participants } from './conversationStore';

// État d'appel WebRTC
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

// Store pour l'état d'appel
export const callStore: Writable<CallState> = writable({
  isCalling: false,
  isAnswering: false,
  isInCall: false,
  callType: 'audio',
  localStream: null,
  remoteStreams: new Map(),
  peerConnections: new Map(),
  currentConversationId: null,
  error: null,
  isMuted: false,
  isVideoOff: false
});

class WebRTCCallManager {
  private ws: WebSocket | null = null;
  private localStream: MediaStream | null = null;
  private conversationId: string = '';
  private userId: string = '';
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private remoteStreams: Map<string, MediaStream> = new Map();

  constructor() {
    const user = get(authStore).user;
    this.userId = user?.id || 'anonymous';
  }

  private async setupLocalStream(type: 'audio' | 'video'): Promise<MediaStream> {
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
    }

    try {
      const constraints = {
        audio: true,
        video: type === 'video' ? { width: 1280, height: 720, frameRate: 30 } : false
      };

      this.localStream = await navigator.mediaDevices.getUserMedia(constraints);
      
      // Appliquer l'état mute/vidéo off si nécessaire
      this.updateTrackStates();
      
      return this.localStream;
    } catch (err) {
      throw new Error(`Impossible d'accéder à la ${type === 'video' ? 'caméra' : 'microphone'}: ${err.message}`);
    }
  }

  private createPeerConnection(remoteUserId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun.relay.metered.ca:80' }
      ],
      iceCandidatePoolSize: 10
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
      const stream = event.streams[0];
      this.remoteStreams.set(remoteUserId, stream);
      
      callStore.update(state => {
        const newStreams = new Map(state.remoteStreams);
        newStreams.set(remoteUserId, stream);
        return { ...state, remoteStreams: newStreams };
      });
    };

    // Gestion des erreurs
    pc.onconnectionstatechange = () => {
      const state = pc.connectionState;
      if (state === 'failed' || state === 'disconnected') {
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
        await this.handleJoin(signal);
        break;
      case 'leave':
        this.handleLeave(signal);
        break;
      case 'decline':
        this.handleDecline(signal);
        break;
    }
  }

  private async handleOffer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id || !this.localStream) return;

    // Créer une connexion pour cet utilisateur
    const pc = this.createPeerConnection(signal.from_user_id);
    
    try {
      await pc.setRemoteDescription(new RTCSessionDescription({
        type: 'offer',
        sdp: signal.sdp
      }));

      const answer = await pc.createAnswer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: signal.sdp.includes('m=video') // Vérifier si l'offre contient de la vidéo
      });

      await pc.setLocalDescription(answer);

      this.sendSignal({
        type: 'answer',
        to_user_id: signal.from_user_id,
        sdp: answer.sdp
      });

      callStore.update(state => ({
        ...state,
        isAnswering: false,
        isInCall: true
      }));
    } catch (err) {
      console.error('Erreur lors de la gestion de l\'offre:', err);
      this.endCallForUser(signal.from_user_id);
    }
  }

  private async handleAnswer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;

    const pc = this.peerConnections.get(signal.from_user_id);
    if (!pc) return;

    try {
      await pc.setRemoteDescription(new RTCSessionDescription({
        type: 'answer',
        sdp: signal.sdp
      }));
    } catch (err) {
      console.error('Erreur lors de la gestion de la réponse:', err);
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
    if (signal.from_user_id === this.userId || !this.localStream) return;

    // Si nous sommes déjà en appel, inviter le nouveau participant
    if (get(callStore).isInCall && !this.peerConnections.has(signal.from_user_id)) {
      await this.initiateCallWithUser(signal.from_user_id);
    }
  }

  private handleLeave(signal: CallSignal) {
    this.endCallForUser(signal.from_user_id);
  }

  private handleDecline(signal: CallSignal) {
    if (signal.from_user_id === this.userId) return;
    this.endCallForUser(signal.from_user_id);
  }

  private updateTrackStates() {
    if (!this.localStream) return;

    const audioTracks = this.localStream.getAudioTracks();
    const videoTracks = this.localStream.getVideoTracks();

    const state = get(callStore);
    
    audioTracks.forEach(track => {
      track.enabled = !state.isMuted;
    });

    videoTracks.forEach(track => {
      track.enabled = !state.isVideoOff;
    });
  }

  public async startCall(conversationId: string, participantIds: string[], type: 'audio' | 'video') {
    try {
      callStore.update(state => ({
        ...state,
        isCalling: true,
        callType: type,
        currentConversationId: conversationId,
        error: null
      }));

      this.conversationId = conversationId;
      this.localStream = await this.setupLocalStream(type);

      callStore.update(state => ({
        ...state,
        localStream: this.localStream,
        isCalling: true
      }));

      // Connexion WebSocket pour le signaling
      const wsUrl = `wss://${window.location.host}/ws/call?conv=${conversationId}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        // Envoyer un signal de join
        this.sendSignal({ type: 'join' });
        
        // Initier les appels avec tous les participants (sauf soi-même)
        const targets = participantIds.filter(id => id !== this.userId);
        
        if (targets.length === 0) {
          callStore.update(state => ({
            ...state,
            isCalling: false,
            isInCall: true,
            error: 'Aucun participant à appeler'
          }));
          return;
        }

        targets.forEach(userId => {
          this.initiateCallWithUser(userId).catch(err => {
            console.error(`Erreur appel à ${userId}:`, err);
            callStore.update(state => ({
              ...state,
              error: `Impossible d'appeler ${userId}`
            }));
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

      this.ws.onerror = (error) => {
        callStore.update(state => ({
          ...state,
          error: 'Erreur de connexion WebRTC'
        }));
      };

      this.ws.onclose = () => {
        console.log('WebSocket fermé');
        if (get(callStore).isInCall) {
          this.endCall();
        }
      };

    } catch (err) {
      callStore.update(state => ({
        ...state,
        error: err instanceof Error ? err.message : 'Erreur inconnue',
        isCalling: false
      }));
      this.endCall();
      throw err;
    }
  }

  private async initiateCallWithUser(remoteUserId: string) {
    const pc = this.createPeerConnection(remoteUserId);
    
    try {
      const offer = await pc.createOffer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: this.localStream?.getVideoTracks().length > 0
      });
      
      await pc.setLocalDescription(offer);

      this.sendSignal({
        type: 'offer',
        to_user_id: remoteUserId,
        sdp: offer.sdp
      });
    } catch (err) {
      console.error('Erreur création offre:', err);
      this.endCallForUser(remoteUserId);
    }
  }

  public async toggleMute() {
    callStore.update(state => {
      const newState = { ...state, isMuted: !state.isMuted };
      return newState;
    });
    
    // Mettre à jour les tracks audio
    if (this.localStream) {
      const audioTracks = this.localStream.getAudioTracks();
      const muted = get(callStore).isMuted;
      audioTracks.forEach(track => {
        track.enabled = !muted;
      });
    }
  }

  public async toggleVideo() {
    callStore.update(state => {
      const newState = { ...state, isVideoOff: !state.isVideoOff };
      return newState;
    });
    
    // Mettre à jour les tracks vidéo
    if (this.localStream) {
      const videoTracks = this.localStream.getVideoTracks();
      const videoOff = get(callStore).isVideoOff;
      videoTracks.forEach(track => {
        track.enabled = !videoOff;
      });
    }
  }

  public sendDeclineSignal(fromUserId: string, conversationId: string) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    
    const signal: CallSignal = {
      conversationId: conversationId,
      from_user_id: this.userId,
      to_user_id: fromUserId,
      type: 'decline',
      sdp: null,
      candidate: null,
      timestamp: Date.now()
    };
    
    this.ws.send(JSON.stringify(signal));
  }

  private endCallForUser(userId: string) {
    const pc = this.peerConnections.get(userId);
    if (pc) {
      pc.close();
      this.peerConnections.delete(userId);
    }

    const stream = this.remoteStreams.get(userId);
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      this.remoteStreams.delete(userId);
    }

    callStore.update(state => {
      const newStreams = new Map(state.remoteStreams);
      const newPCs = new Map(state.peerConnections);
      
      newStreams.delete(userId);
      newPCs.delete(userId);
      
      // Si plus personne dans l'appel, terminer complètement
      if (newStreams.size === 0 && newPCs.size === 0 && !state.isCalling) {
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
    this.peerConnections.clear();
    
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
      isAnswering: false,
      isInCall: false,
      callType: 'audio',
      localStream: null,
      remoteStreams: new Map(),
      peerConnections: new Map(),
      currentConversationId: null,
      error: null,
      isMuted: false,
      isVideoOff: false
    });
  }
}

// Singleton
export const callManager = new WebRTCCallManager();

// Fonctions utilitaires pour les composants
export async function startGroupCall(conversationId: string, participantIds: string[], type: 'audio' | 'video' = 'audio') {
  await callManager.startCall(conversationId, participantIds, type);
}

export function endCurrentCall() {
  callManager.endCall();
}

// Émettre un événement d'appel entrant pour le composant UI
export function emitIncomingCall(fromUserId: string, conversationId: string) {
  const event = new CustomEvent('incoming-call', {
    detail: { from_user_id: fromUserId, conversation_id: conversationId }
  });
  window.dispatchEvent(event);
}