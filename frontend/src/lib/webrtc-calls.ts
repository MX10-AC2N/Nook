// src/lib/webrtc-calls.ts (Svelte 5 avec runes)
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import type { CallSignal } from './types';
import { authUser } from './authStore';

// -----------------------------------------------------------------
// 1️⃣ Types & état réactif (Svelte 5)
// -----------------------------------------------------------------
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
  localVideoElement: HTMLVideoElement | null; // Ajout pour Svelte 5
}

/** Crée un état vierge (utilisé au démarrage et lors du reset). */
function createInitialState(): CallState {
  return {
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
    isVideoOff: false,
    localVideoElement: null,
  };
}

/** État global réactif de l'appel (Svelte 5) */
export const callStore = $state<CallState>(createInitialState());

// -----------------------------------------------------------------
// 2️⃣ Classe de gestion WebRTC
// -----------------------------------------------------------------
class WebRTCCallManager {
  private ws: WebSocket | null = null;
  private conversationId: string = '';
  private userId: string = '';

  constructor() {
    // Accès direct à la rune authUser (Svelte 5)
    this.userId = authUser?.id ?? 'anonymous';
  }

  // -----------------------------------------------------------------
  // Helpers internes
  // -----------------------------------------------------------------
  private updateState(partial: Partial<CallState>) {
    // Mutation directe de l'état réactif (Svelte 5)
    Object.assign(callStore, partial);
  }

  private async setupLocalStream(type: 'audio' | 'video'): Promise<MediaStream> {
    // Arrêter le flux précédent s'il existe
    if (callStore.localStream) {
      callStore.localStream.getTracks().forEach((t) => t.stop());
    }

    const constraints: MediaStreamConstraints = {
      audio: true,
      video: type === 'video' ? { width: 1280, height: 720, frameRate: 30 } : false,
    };

    try {
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      
      // Mise à jour de l'état réactif
      callStore.localStream = stream;
      this.applyMuteVideoState(); // applique mute / videoOff selon l'état actuel
      
      // Si un élément vidéo existe, lui attribuer le flux
      if (callStore.localVideoElement) {
        callStore.localVideoElement.srcObject = stream;
      }
      
      return stream;
    } catch (err: any) {
      throw new Error(
        `Impossible d'accéder au ${
          type === 'video' ? 'caméra' : 'microphone'
        } : ${err.message}`
      );
    }
  }

  /** Crée (ou récupère) une RTCPeerConnection pour un participant distant. */
  private createPeerConnection(remoteUserId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun.relay.metered.ca:80' },
      ],
      iceCandidatePoolSize: 10,
    });

    // Ajouter le flux local (audio/vidéo) à la connexion
    if (callStore.localStream) {
      callStore.localStream.getTracks().forEach((track) => pc.addTrack(track, callStore.localStream!));
    }

    // -------------------------------------------------------------
    // Gestion des ICE candidates
    // -------------------------------------------------------------
    pc.onicecandidate = (event) => {
      if (event.candidate && this.ws?.readyState === WebSocket.OPEN) {
        this.sendSignal({
          type: 'ice',
          to_user_id: remoteUserId,
          candidate: event.candidate,
        });
      }
    };

    // -------------------------------------------------------------
    // Gestion du flux distant (remote stream)
    // -------------------------------------------------------------
    pc.ontrack = (event) => {
      const stream = event.streams[0];
      
      // Mise à jour réactive du Map remoteStreams
      const newRemoteStreams = new Map(callStore.remoteStreams);
      newRemoteStreams.set(remoteUserId, stream);
      callStore.remoteStreams = newRemoteStreams;
    };

    // -------------------------------------------------------------
    // Nettoyage en cas de perte de connexion
    // -------------------------------------------------------------
    pc.onconnectionstatechange = () => {
      if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected') {
        this.endCallForUser(remoteUserId);
      }
    };

    // Mise à jour réactive du Map peerConnections
    const newPeerConnections = new Map(callStore.peerConnections);
    newPeerConnections.set(remoteUserId, pc);
    callStore.peerConnections = newPeerConnections;
    
    return pc;
  }

  /** Envoie un signal via le WebSocket (signalling). */
  private sendSignal(signal: Partial<CallSignal>) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN || !this.conversationId) return;

    const fullSignal: CallSignal = {
      conversationId: this.conversationId,
      from_user_id: this.userId,
      to_user_id: signal.to_user_id ?? null,
      type: signal.type ?? 'unknown',
      sdp: signal.sdp ?? null,
      candidate: signal.candidate ?? null,
      timestamp: Date.now(),
    };

    this.ws.send(JSON.stringify(fullSignal));
  }

  // -----------------------------------------------------------------
  // Signal handling (receiving)
  // -----------------------------------------------------------------
  private async handleSignal(signal: CallSignal) {
    // Ignorer les signaux provenant de soi-même
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
      default:
        console.warn('Signal inconnu reçu :', signal);
    }
  }

  private async handleOffer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id || !callStore.localStream) return;

    const pc = this.createPeerConnection(signal.from_user_id);
    try {
      await pc.setRemoteDescription(
        new RTCSessionDescription({ type: 'offer', sdp: signal.sdp })
      );

      const answer = await pc.createAnswer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: signal.sdp.includes('m=video'),
      });
      await pc.setLocalDescription(answer);

      this.sendSignal({ type: 'answer', to_user_id: signal.from_user_id, sdp: answer.sdp });

      callStore.isAnswering = false;
      callStore.isInCall = true;
    } catch (err) {
      console.error('Erreur lors du traitement de l\'offre :', err);
      this.endCallForUser(signal.from_user_id);
    }
  }

  private async handleAnswer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;

    const pc = callStore.peerConnections.get(signal.from_user_id);
    if (!pc) return;

    try {
      await pc.setRemoteDescription(
        new RTCSessionDescription({ type: 'answer', sdp: signal.sdp })
      );
    } catch (err) {
      console.error('Erreur lors du traitement de la réponse :', err);
      this.endCallForUser(signal.from_user_id);
    }
  }

  private async handleIceCandidate(signal: CallSignal) {
    if (!signal.candidate || !signal.from_user_id) return;

    const pc = callStore.peerConnections.get(signal.from_user_id);
    if (!pc) return;

    try {
      await pc.addIceCandidate(new RTCIceCandidate(signal.candidate));
    } catch (err) {
      console.error('Erreur ICE candidate :', err);
    }
  }

  private async handleJoin(signal: CallSignal) {
    if (signal.from_user_id === this.userId || !callStore.localStream) return;

    // Si on est déjà en appel, initier une connexion vers le nouveau participant
    if (callStore.isInCall && !callStore.peerConnections.has(signal.from_user_id)) {
      await this.initiateCallWithUser(signal.from_user_id);
    }
  }

  private handleLeave(signal: CallSignal) {
    this.endCallForUser(signal.from_user_id);
  }

  private handleDecline(signal: CallSignal) {
    if (signal.from_user_id !== this.userId) {
      this.endCallForUser(signal.from_user_id);
    }
  }

  // -----------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------
  /** Démarre un appel de groupe (caller). */
  public async startCall(
    conversationId: string,
    participantIds: string[],
    type: 'audio' | 'video'
  ): Promise<void> {
    try {
      // Met à jour l'état réactif
      callStore.isCalling = true;
      callStore.callType = type;
      callStore.currentConversationId = conversationId;
      callStore.error = null;

      this.conversationId = conversationId;
      await this.setupLocalStream(type);
      callStore.isCalling = true;

      // --- WebSocket (signalling) ---
      const wsUrl = `wss://${browser ? window.location.host : 'localhost:3000'}/ws/call?conv=${conversationId}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        // Annonce notre présence dans la conversation
        this.sendSignal({ type: 'join' });

        // Initier les appels vers chaque participant (hors self)
        const targets = participantIds.filter((id) => id !== this.userId);
        if (targets.length === 0) {
          callStore.isCalling = false;
          callStore.isInCall = true;
          callStore.error = 'Aucun participant à appeler';
          return;
        }

        targets.forEach((uid) => {
          this.initiateCallWithUser(uid).catch((err) => {
            console.error(`Erreur appel vers ${uid} :`, err);
            callStore.error = `Impossible d'appeler ${uid}`;
          });
        });
      };

      this.ws.onmessage = (ev) => {
        try {
          const signal = JSON.parse(ev.data) as CallSignal;
          this.handleSignal(signal);
        } catch (err) {
          console.error('Erreur parsing signal :', err);
        }
      };

      this.ws.onerror = () => {
        callStore.error = 'Erreur de connexion WebSocket';
      };

      this.ws.onclose = () => {
        console.log('WebSocket fermé');
        if (callStore.isInCall) this.endCall();
      };
    } catch (err: any) {
      callStore.error = err instanceof Error ? err.message : 'Erreur inconnue';
      callStore.isCalling = false;
      this.endCall();
      throw err;
    }
  }

  /** Initialise un appel (offer) vers un utilisateur distant. */
  private async initiateCallWithUser(remoteUserId: string): Promise<void> {
    const pc = this.createPeerConnection(remoteUserId);
    try {
      const offer = await pc.createOffer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: callStore.localStream?.getVideoTracks().length! > 0,
      });
      await pc.setLocalDescription(offer);
      this.sendSignal({ type: 'offer', to_user_id: remoteUserId, sdp: offer.sdp });
    } catch (err) {
      console.error('Erreur création offre :', err);
      this.endCallForUser(remoteUserId);
    }
  }

  /** Répond à une offre (callee). */
  public async answerCall(conversationId: string, participantIds: string[], type: 'audio' | 'video') {
    // Cette méthode peut être appelée depuis l'interface si on veut répondre manuellement.
    // Pour le moment, on réutilise `startCall` avec `isAnswering` à true.
    callStore.isAnswering = true;
    callStore.currentConversationId = conversationId;
    await this.startCall(conversationId, participantIds, type);
  }

  /** Bascule le mute du microphone. */
  public toggleMute(): void {
    callStore.isMuted = !callStore.isMuted;
    this.applyMuteVideoState();
  }

  /** Bascule la vidéo (on/off). */
  public toggleVideo(): void {
    callStore.isVideoOff = !callStore.isVideoOff;
    this.applyMuteVideoState();
  }

  /** Applique les états `isMuted` et `isVideoOff` sur le flux local. */
  private applyMuteVideoState() {
    if (!callStore.localStream) return;
    callStore.localStream.getAudioTracks().forEach((t) => (t.enabled = !callStore.isMuted));
    callStore.localStream.getVideoTracks().forEach((t) => (t.enabled = !callStore.isVideoOff));
  }

  /** Termine l'appel en cours et nettoie toutes les ressources. */
  public endCall(): void {
    // Fermer toutes les connexions peer-to-peer
    callStore.peerConnections.forEach((pc) => pc.close());
    callStore.peerConnections = new Map();

    // Arrêter le flux local
    if (callStore.localStream) {
      callStore.localStream.getTracks().forEach((t) => t.stop());
      callStore.localStream = null;
      
      // Nettoyer l'élément vidéo
      if (callStore.localVideoElement) {
        callStore.localVideoElement.srcObject = null;
      }
    }

    // Fermer le WebSocket
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    // Reset de l'état réactif
    Object.assign(callStore, createInitialState());
  }

  /** Termine l'appel avec un participant précis (déconnexion ou erreur). */
  private endCallForUser(remoteUserId: string): void {
    const pc = callStore.peerConnections.get(remoteUserId);
    if (pc) {
      pc.close();
      const newPeerConnections = new Map(callStore.peerConnections);
      newPeerConnections.delete(remoteUserId);
      callStore.peerConnections = newPeerConnections;
    }

    // Retirer le flux distant
    const newRemoteStreams = new Map(callStore.remoteStreams);
    newRemoteStreams.delete(remoteUserId);
    callStore.remoteStreams = newRemoteStreams;

    // Si plus aucun participant, on termine l'appel complet
    if (callStore.peerConnections.size === 0 && callStore.isInCall) {
      this.endCall();
    }
  }
}

// -----------------------------------------------------------------
// 3️⃣ Export des fonctions utilitaires
// -----------------------------------------------------------------
export const callManager = new WebRTCCallManager();

/**
 * Démarre un appel de groupe (caller) pour la conversation donnée.
 *
 * @param conversationId   Identifiant de la conversation (ex. `/call/[id]`).
 * @param participantIds   Tableau d'identifiants des participants (exclut l'appelant).
 * @param type             `audio` ou `video` (défaut : `audio`).
 */
export async function startGroupCall(
  conversationId: string,
  participantIds: string[],
  type: 'audio' | 'video' = 'audio'
): Promise<void> {
  await callManager.startCall(conversationId, participantIds, type);
}

/** Termine l'appel en cours (appelé depuis l'UI). */
export function endCurrentCall(): void {
  callManager.endCall();
}

/** Retourne l'état actuel (alias pour compatibilité). */
export function getCallState(): CallState {
  return callStore;
}