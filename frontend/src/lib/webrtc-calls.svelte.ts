// src/lib/webrtc-calls.ts (Svelte 5 avec runes)
import { notifyCall } from '$lib/notificationStore.svelte';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import type { CallSignal } from './types';
import { authStore } from './authStore.svelte.js';

/** Génère les credentials TURN long-term (RFC 5389). */
async function generateTurnCredentials(secret: string, validityHours = 24): Promise<{ username: string; credential: string }> {
  const username = String(Math.floor(Date.now() / 1000) + (validityHours * 3600));
  const encoder = new TextEncoder();
  const keyData = encoder.encode(secret);
  const msgData = encoder.encode(username);
  const cryptoKey = await crypto.subtle.importKey('raw', keyData, { name: 'HMAC', hash: 'SHA-1' }, false, ['sign']);
  const signature = await crypto.subtle.sign('HMAC', cryptoKey, msgData);
  const credential = btoa(String.fromCharCode(...new Uint8Array(signature)));
  return { username, credential };
}

// ── Configuration TURN/STUN (100% self-hosted via turn-rs) ────
const TURN_SECRET = 'change_this_secret'; // TODO: read from env/config
const TURN_HOST = '192.168.1.100'; // TODO: read from env/config (or window.location.hostname)
const TURN_PORT = 3478;

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
  fileDataChannels: Map<string, RTCDataChannel>;
  currentConversationId: string | null;
  error: string | null;
  isMuted: boolean;
  isVideoOff: boolean;
  localVideoElement: HTMLVideoElement | null; // Ajout pour Svelte 5
  isScreenSharing: boolean;
  screenShareStream: MediaStream | null;
  screenShareLocalVideoElement: HTMLVideoElement | null;
  // Call quality monitoring
  callQuality: 'good' | 'fair' | 'poor' | 'unknown';
  currentBitrate: number; // kbps average
  packetsLost: number;
  jitter: number; // ms
  rtt: number; // round-trip ms
  remoteResolution: string | null; // e.g. "1280x720"
  remoteFps: number;
    // SFU state
    useSfu: false,
    sfuAnswer: null,
    sfuRenegotiateOffer: null,
    sfuPeers: [],
    sfuPendingOffer: null,
    // ═══ SFU state ═══
    useSfu: boolean;
    sfuAnswer: string | null;
    sfuRenegotiateOffer: string | null;
    sfuPeers: string[];
    sfuPendingOffer: string | null; // offer from SFU for renegotiation
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
    fileDataChannels: new Map<string, RTCDataChannel>(),
    currentConversationId: null,
    error: null,
    isMuted: false,
    isVideoOff: false,
    localVideoElement: null,
    isScreenSharing: false,
    screenShareStream: null,
    screenShareLocalVideoElement: null,
    // Call quality monitoring defaults
    callQuality: 'unknown',
    currentBitrate: 0,
    packetsLost: 0,
    jitter: 0,
    rtt: 0,
    remoteResolution: null,
    remoteFps: 0,
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
  // ── Sonnerie ────────────────────────────────────────────────
  private ringtoneCtx: AudioContext | null = null;
  private ringtoneInterval: ReturnType<typeof setInterval> | null = null;
  // ── Call quality monitoring ─────────────────────────────────
  private prevBytesSent = new Map<string, number>();
  private prevStatsTime = new Map<string, number>();
  private callQualityInterval: ReturnType<typeof setInterval> | null = null;

  // userId est un getter réactif — toujours synchronisé avec authStore
  private get userId(): string {
    return authStore.user?.id ?? 'anonymous';
  }

  // ── Sonnerie : tonalité synthétisée (pas de fichier externe) ─
  public startRingtone(): void {
    if (this.ringtoneInterval) return; // déjà en cours
    this._ringOnce();
    this.ringtoneInterval = setInterval(() => this._ringOnce(), 3000);
  }

  private _ringOnce(): void {
    try {
      if (!browser) return;
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      this.ringtoneCtx = ctx;
      // Deux tonalités (300ms + 300ms) séparées par 150ms
      const playTone = (freq: number, startSec: number, durSec: number) => {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.connect(gain); gain.connect(ctx.destination);
        osc.frequency.value = freq;
        osc.type = 'sine';
        gain.gain.setValueAtTime(0.3, ctx.currentTime + startSec);
        gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + startSec + durSec);
        osc.start(ctx.currentTime + startSec);
        osc.stop(ctx.currentTime + startSec + durSec);
      };
      playTone(880, 0, 0.3);    // La5
      playTone(1100, 0.45, 0.3); // Do#6
    } catch { /* AudioContext non disponible */ }
  }
  public stopRingtone() {
    if (this.ringtoneInterval) {
      clearInterval(this.ringtoneInterval);
      this.ringtoneInterval = null;
    }
    if (this.ringtoneCtx) {
      try { this.ringtoneCtx.close(); } catch { /* */ }
      this.ringtoneCtx = null;
    }
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
  private async createPeerConnection(remoteUserId: string): Promise<RTCPeerConnection> {
    // Génère des credentials TURN dynamiques (expirent après validityHours)
    const turnHost = (typeof window !== 'undefined' && window.location.hostname) || TURN_HOST;
    const creds = await generateTurnCredentials(TURN_SECRET);
    const iceServers: RTCIceServer[] = [
      { urls: `stun:${turnHost}:${TURN_PORT}` },
      { urls: `turn:${turnHost}:${TURN_PORT}?transport=udp`, username: creds.username, credential: creds.credential },
      { urls: `turn:${turnHost}:${TURN_PORT}?transport=tcp`, username: creds.username, credential: creds.credential },
    ];

    const pc = new RTCPeerConnection({
      iceServers,
      iceCandidatePoolSize: 10,
    });

    // ── File Transfer Data Channel ──
    const fileChan = pc.createDataChannel('file-transfer', {
      ordered: true,
      maxRetransmits: 3,
    });
    fileChan.binaryType = 'arraybuffer';
    this.fileDataChannels.set(remoteUserId, fileChan);
    
    fileChan.onmessage = (ev) => {
      import('./file-transfer.svelte.ts').then(({ handleFileTransferMessage }) => {
        handleFileTransferMessage(ev.data);
      }).catch(e => console.error('[FileChannel] Error:', e));
    };
    fileChan.onopen = () => {
      console.log(`[FileChannel] Open to ${remoteUserId}`);
    };
    fileChan.onclose = () => {
      this.fileDataChannels.delete(remoteUserId);
      console.log(`[FileChannel] Closed to ${remoteUserId}`);
    };

    // Handle incoming DataChannel requests from peer
    pc.ondatachannel = (event) => {
      const ch = event.channel;
      ch.binaryType = 'arraybuffer';
      ch.onmessage = (ev) => {
        import('./file-transfer.svelte.ts').then(({ handleFileTransferMessage }) => {
          handleFileTransferMessage(ev.data);
        }).catch(e => console.error('[FileChannel] Incoming error:', e));
      };
    };

    // Ajouter le flux local (audio/vidéo) à la connexion
    if (callStore.localStream) {
      callStore.localStream.getTracks().forEach((track) => pc.addTrack(track, callStore.localStream!));
    }

    // Si le partage d'ecran est actif, ajouter egalement la piste d'ecran
    if (callStore.isScreenSharing && callStore.screenShareStream) {
      const screenTrack = callStore.screenShareStream.getVideoTracks()[0];
      if (screenTrack) {
        pc.addTrack(screenTrack.clone(), callStore.screenShareStream);
      }
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
      case 'webrtc_offer':
        await this.handleOffer(signal);
        break;
      case 'answer':
      case 'webrtc_answer':
        await this.handleAnswer(signal);
        break;
      case 'ice':
      case 'ice_candidate':
      case 'webrtc_ice_candidate':
        await this.handleIceCandidate(signal);
        break;
      case 'join':
        await this.handleJoin(signal);
        break;
      case 'leave':
        this.handleLeave(signal);
        break;
      case 'decline':
      case 'call_rejected':
        this.stopRingtone();
        this.handleDecline(signal);
        break;
      // ── Sonnerie : appel entrant ──────────────────────────────
      case 'call_request':
        this.stopRingtone();
        this.playRingtone();
        if (browser) {
          notifyCall(signal.from_user_name || 'Quelqu un');
        window.dispatchEvent(new CustomEvent('incoming-call', {
            detail: {
              from_user_id: signal.from_user_id,
              from_user_name: signal.from_user_name ?? signal.from_user_id,
              conversationId: signal.conversationId,
              callType: signal.callType ?? 'audio',
            }
          }));
        }
        break;
      // ━━━ SFU Signalisation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      case 'sfu_answer':
        if (signal.answer) {
          this.handleSfuJoinResponse({
            answer: signal.answer as string,
            peers: signal.peers as string[] || [],
            renegotiate_offer: signal.renegotiate_offer as string,
          });
        }
        break;
      case 'sfu_renegotiate_offer':
        if (signal.offer) {
          this.handleSfuRenegotiateOffer(signal.offer as string);
        }
        break;
      case 'sfu_peers':
        if (signal.peers) {
          this.handleSfuPeers({ peers: signal.peers as string[] });
        }
        break;
      case 'sfu_error':
        callStore.error = (signal.error as string) || 'SFU error';
        callStore.isCalling = false;
        break;
      case 'call_accepted':
        this.stopRingtone();
        callStore.isCalling = false;
        callStore.isInCall = true;
        break;
      default:
        console.warn('Signal inconnu reçu :', signal.type);
    }
  }

  private async handleOffer(signal: CallSignal) {
    if (!signal.sdp || !signal.from_user_id) return;

    // BUG-CALL-5 FIX : l'appelé peut recevoir une offer sans avoir encore de stream local
    // (il arrive sur la page call sans avoir cliqué "Démarrer").
    // → Setup automatique du stream avant de répondre.
    if (!callStore.localStream) {
      try {
        const hasVideo = signal.sdp.includes('m=video');
        await this.setupLocalStream(hasVideo ? 'video' : 'audio');
        callStore.isAnswering = true;
      } catch (err: any) {
        callStore.error = `Impossible d'accéder au micro/caméra : ${err.message}`;
        return;
      }
    }

    const pc = await this.createPeerConnection(signal.from_user_id);
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

      // Arrêter la sonnerie si elle joue (appel accepté automatiquement)
      this.stopRingtone();
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
    // Auto-switch to SFU for 3+ participants
    if (participantIds.length >= 3) {
      callStore.useSfu = true;
      return this.startSfuCall(conversationId, participantIds, type);
    }

      callStore.isCalling = true;
      callStore.callType = type;
      callStore.currentConversationId = conversationId;
      callStore.error = null;

      this.conversationId = conversationId;
      await this.setupLocalStream(type);
      callStore.isCalling = true;

      // --- WebSocket (signalling) ---
      // Protocole correct : ws en HTTP, wss en HTTPS — même logique que chatStore
      const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
      // BUG-CALL-1 FIX : /ws/call n'existe pas côté backend → utiliser /ws
      const wsUrl = `${proto}://${window.location.host}/ws`;
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

        // Envoyer call_request à chaque destinataire → déclenche la sonnerie chez eux
        const callerName = (authStore.user as any)?.name ?? (authStore.user as any)?.username ?? this.userId;
        targets.forEach((uid) => {
          this.sendSignal({
            type: 'call_request',
            to_user_id: uid,
            from_user_name: callerName,
            callType: type,
          });
        });

        // Puis lancer les offres WebRTC
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
    const pc = await this.createPeerConnection(remoteUserId);
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

  /** Envoie un signal call_rejected à l'appelant (refus de l'appel entrant). */
  public sendReject(convId: string, toUserId: string): void {
    // Ouvrir un WS temporaire si nécessaire, ou utiliser le WS existant
    if (this.ws?.readyState === WebSocket.OPEN) {
      const signal = {
        type: 'call_rejected',
        conversationId: convId,
        from_user_id: this.userId,
        to_user_id: toUserId,
        sdp: null,
        candidate: null,
        timestamp: Date.now(),
      };
      this.ws.send(JSON.stringify(signal));
    }
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

  /** Demarre le partage d'ecran via getDisplayMedia. */
  public async startScreenShare(): Promise<void> {
    if (callStore.isScreenSharing) return;

    const displayStream = await navigator.mediaDevices.getDisplayMedia({
      video: { cursor: 'always', displaySurface: 'monitor' },
      audio: false, // On ne capture pas l'audio systeme pour un appel familial
    });

    // Remplacer la piste video des peerConnections existantes
    const videoTrack = displayStream.getVideoTracks()[0];
    if (!videoTrack) return;

    // Ecouter l'arret du partage (bouton navigateur)
    videoTrack.onended = () => {
      this.stopScreenShare();
    };

    // Remplacer la piste dans chaque RTCPeerConnection
    callStore.peerConnections.forEach(async (pc, remoteUserId) => {
      const senders = pc.getSenders();
      const videoSender = senders.find((s) => s.track?.kind === 'video');
      if (videoSender) {
        await videoSender.replaceTrack(videoTrack.clone());
      }
    });

    callStore.isScreenSharing = true;
    callStore.screenShareStream = displayStream;

    if (callStore.screenShareLocalVideoElement && displayStream) {
      callStore.screenShareLocalVideoElement.srcObject = displayStream;
    }
  }

  /** Arrete le partage d'ecran et restaure le flux camera local. */
  public stopScreenShare(): void {
    if (!callStore.isScreenSharing) return;

    // Stopper les tracks du display
    if (callStore.screenShareStream) {
      callStore.screenShareStream.getTracks().forEach((t) => t.stop());
      callStore.screenShareStream = null;
    }

    // Remettre le flux local (camera) dans les peerConnections
    if (callStore.localStream && callStore.localStream.getVideoTracks().length > 0) {
      const localVideoTrack = callStore.localStream.getVideoTracks()[0];
      callStore.peerConnections.forEach(async (pc) => {
        const senders = pc.getSenders();
        const videoSender = senders.find((s) => s.track?.kind === 'video');
        if (videoSender) {
          await videoSender.replaceTrack(localVideoTrack);
        }
      });
    }

    if (callStore.screenShareLocalVideoElement) {
      callStore.screenShareLocalVideoElement.srcObject = null;
    }

    callStore.isScreenSharing = false;
  }

  /** Toggle partage d'ecran. */
  public toggleScreenShare(): void {
    if (callStore.isScreenSharing) {
      this.stopScreenShare();
    } else {
      this.startScreenShare().catch((err) => {
        callStore.error = `Impossible de partager l'ecran: ${err.message}`;
      });
    }
  }

  /** Collect and update call quality stats every 2 seconds. */
  public async updateCallQuality(): Promise<void> {
    if (!callStore.isInCall || callStore.peerConnections.size === 0) return;

    let totalPacketsLost = 0;
    let maxJitter = 0;
    let maxRtt = 0;
    let totalBitrate = 0;
    let resolution: string | null = null;
    let fps = 0;

    for (const [remoteUserId, pc] of callStore.peerConnections.entries()) {
      try {
        const stats = await pc.getStats();
        
        for (const report of stats.values()) {
          // Inbound RTP (receiving remote media)
          if (report.type === 'inbound-rtp' && report.kind === 'video') {
            totalPacketsLost += (report.packetsLost ?? 0);
            maxJitter = Math.max(maxJitter, report.jitter ?? 0);
            if (report.frameWidth && report.frameHeight) {
              resolution = `${report.frameWidth}x${report.frameHeight}`;
              fps = report.framesPerSecond ?? 0;
            }
          }
          
          // Candidate pair (network quality)
          if (report.type === 'candidate-pair' && report.state === 'succeeded') {
            maxRtt = Math.max(maxRtt, report.currentRoundTripTime ?? 0);
          }
        }
      } catch (err) {
        // Ignore stats errors
      }
    }

    // Jitter in ms (WebRTC reports in seconds)
    const jitterMs = maxJitter * 1000;
    
    // Determine quality level
    let quality: 'good' | 'fair' | 'poor' = 'good';
    if (maxRtt > 400 || jitterMs > 100 || totalPacketsLost > 100) {
      quality = 'poor';
    } else if (maxRtt > 200 || jitterMs > 30 || totalPacketsLost > 10) {
      quality = 'fair';
    }

    callStore.callQuality = quality;
    callStore.packetsLost = totalPacketsLost;
    callStore.jitter = Math.round(jitterMs * 10) / 10;
    callStore.rtt = Math.round(maxRtt * 1000);
    callStore.remoteResolution = resolution;
    callStore.remoteFps = fps;
  }

  /** Start call quality monitoring interval. */
  private startQualityMonitoring(): void {
    if (this.callQualityInterval) return;
    this.callQualityInterval = setInterval(() => {
      this.updateCallQuality();
    }, 2000);
    // Initial call
    this.updateCallQuality();
  }

  /** Stop call quality monitoring interval. */
  private stopQualityMonitoring(): void {
    if (this.callQualityInterval) {
      clearInterval(this.callQualityInterval);
      this.callQualityInterval = null;
    }
  }

  /** Applique les états `isMuted` et `isVideoOff` sur le flux local. */
  private applyMuteVideoState() {
    if (!callStore.localStream) return;
    callStore.localStream.getAudioTracks().forEach((t) => (t.enabled = !callStore.isMuted));
    callStore.localStream.getVideoTracks().forEach((t) => (t.enabled = !callStore.isVideoOff));
  }

  /** Termine l'appel en cours et nettoie toutes les ressources. */
  public endCall(): void {
    // Arrêter la sonnerie (cas où on raccroche pendant que ça sonne)
    this.stopRingtone();

    // Arreter le partage d'ecran si actif
    if (callStore.isScreenSharing) {
      this.stopScreenShare();
    }

    // Prévenir les pairs qu'on part
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.sendSignal({ type: 'leave' });
    }

    // Fermer toutes les connexions peer-to-peer
    callStore.peerConnections.forEach((pc) => pc.close());
    callStore.peerConnections = new Map();

    // Arrêter le flux local
    if (callStore.localStream) {
      callStore.localStream.getTracks().forEach((t) => t.stop());
      callStore.localStream = null;
      if (callStore.localVideoElement) {
        callStore.localVideoElement.srcObject = null;
      }
    }

    // Fermer le WebSocket
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    // Stop quality monitoring
    this.stopQualityMonitoring();

    // Reset de l'état réactif
  }
  // ══════════════════════════════════════════════════════════
  // SFU CALLS — via backend SFU (rustrtc)
  // ══════════════════════════════════════════════════════════

  /** Demarre un appel SFU pour une conversation (3+ participants). */
  public async startSfuCall(conversationId: string, participantIds: string[], type: 'audio' | 'video'): Promise<void> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN || !this.conversationId) return;

    callStore.isCalling = true;
    callStore.callType = type;
    callStore.currentConversationId = conversationId;
    callStore.useSfu = true;

    // Obtenir le flux local
    await this.setupLocalStream(type);

    // Creer une PeerConnection locale pour le join SFU
    await this.createPeerConnection(this.userId);
    this.sendSignal({
      type: 'sfu_join',
      conversation_id: conversationId,
      sdp: 'offer',
    });
  }

  /** Le backend repond avec answer SDP + peers + renegotiate_offer. */
  public handleSfuJoinResponse(data: { answer: string; peers: string[]; renegotiate_offer?: string }): void {
    callStore.sfuAnswer = data.answer;
    callStore.sfuPeers = data.peers;
    callStore.sfuRenegotiateOffer = data.renegotiate_offer || null;
    callStore.isCalling = false;
    callStore.isInCall = true;
  }

  /** Le backend envoie une offre de renegotiation (nouvelles tracks). */
  public handleSfuRenegotiateOffer(offer: string): void {
    callStore.sfuPendingOffer = offer;
    // On confirme la reception
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.sendSignal({
        type: 'sfu_answer',
        conversation_id: callStore.currentConversationId || '',
        sdp: 'answer',
      });
    }
  }

  /** Le backend informe sur les peers actuels. */
  public handleSfuPeers(data: { peers: string[] }): void {
    callStore.sfuPeers = data.peers;
  }

  /** Arrete le mode SFU et retourne en P2P mesh. */
  public async stopSfuMode(): Promise<void> {
    callStore.useSfu = false;
    callStore.sfuAnswer = null;
    callStore.sfuRenegotiateOffer = null;
    callStore.sfuPeers = [];
    callStore.sfuPendingOffer = null;
  }

  /** Reinitialise l'etat d'appel. */
  resetState(): void {
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

/** Reset call quality monitoring state to defaults. */
export function resetCallState(): void {
  callStore.callQuality = 'unknown';
  callStore.currentBitrate = 0;
  callStore.packetsLost = 0;
  callStore.jitter = 0;
  callStore.rtt = 0;
  callStore.remoteResolution = null;
  callStore.remoteFps = 0;
}