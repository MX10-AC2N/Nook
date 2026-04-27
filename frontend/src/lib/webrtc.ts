// src/lib/webrtc.ts
// Utilise l'API WebRTC native (RTCPeerConnection) au lieu de simple-peer (obsolète).
// Complétement réécrit pour supprimer la dépendance simple-peer (PR #28).

export class WebRtcCall {
  private pc: RTCPeerConnection | null = null;
  private dataChannel: RTCDataChannel | null = null;
  private localStream: MediaStream | null = null;
  
  /** Callback appelé lorsqu'un flux distant est reçu. */
  private onStreamCallback: ((stream: MediaStream) => void) | null = null;
  
  /** Callback appelé lorsqu'une donnée (JSON) est reçue. */
  private onDataCallback: ((data: unknown) => void) | null = null;
  
  /** Callback appelé pour envoyer un signal à l'autre pair. */
  private onSignalCallback: ((data: unknown) => void) | null = null;

  /**
   * Crée un nouveau wrapper WebRTC.
   * @param initiator - true: le client crée l'offre (caller). false: il répond à une offre (callee).
   */
  constructor(private initiator: boolean) {
    this.pc = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        // Les TURN servers seront ajoutés dynamiquement via le serveur
      ]
    });

    // Gestion des candidats ICE
    this.pc.onicecandidate = ({ candidate }) => {
      if (candidate) {
        this.emitSignal({ type: 'ice', candidate });
      }
    };

    // Gestion du flux distant
    this.pc.ontrack = (event) => {
      const [remoteStream] = event.streams;
      if (remoteStream && this.onStreamCallback) {
        this.onStreamCallback(remoteStream);
      }
    };

    // Gestion des DataChannels
    if (this.initiator) {
      // Le caller crée le DataChannel
      this.dataChannel = this.pc.createDataChannel('data');
      this.setupDataChannel(this.dataChannel);
    } else {
      // Le callee écoute les DataChannels entrants
      this.pc.ondatachannel = (event) => {
        this.dataChannel = event.channel;
        this.setupDataChannel(this.dataChannel);
      };
    }
  }

  /** Configure les événements du DataChannel. */
  private setupDataChannel(channel: RTCDataChannel): void {
    channel.onmessage = (event) => {
      if (this.onDataCallback) {
        try {
          const parsed = JSON.parse(event.data);
          this.onDataCallback(parsed);
        } catch {
          // Si le payload n'est pas du JSON valide, on transmet la donnée brute
          this.onDataCallback(event.data);
        }
      }
    };

    channel.onopen = () => {
      console.log('[WebRTC] DataChannel ouvert');
    };

    channel.onclose = () => {
      console.log('[WebRTC] DataChannel fermé');
    };
  }

  // -------------------------------------------------------------
  // SIGNALING
  // -------------------------------------------------------------

  /**
   * Applique le signal reçu du pair (offre, réponse ou ICE).
   */
  async signal(data: unknown): Promise<void> {
    if (!this.pc) throw new Error('Peer not initialized');

    const signal = data as { type: string; [key: string]: unknown };

    try {
      if (signal.type === 'offer') {
        await this.pc.setRemoteDescription(new RTCSessionDescription(signal as RTCSessionDescriptionInit));
        const answer = await this.pc.createAnswer();
        await this.pc.setLocalDescription(answer);
        this.emitSignal(this.pc.localDescription);
      } else if (signal.type === 'answer') {
        await this.pc.setRemoteDescription(new RTCSessionDescription(signal as RTCSessionDescriptionInit));
      } else if (signal.type === 'ice' && signal.candidate) {
        await this.pc.addIceCandidate(new RTCIceCandidate(signal.candidate as RTCIceCandidateInit));
      }
    } catch (err) {
      console.error('[WebRTC] Erreur signal:', err);
      throw err;
    }
  }

  /**
   * Enregistre un callback qui sera appelé chaque fois que le peer
   * génère un nouveau signal (offre, réponse ou ICE).
   */
  onSignal(callback: (data: unknown) => void): void {
    this.onSignalCallback = callback;
  }

  /** Émet un signal vers l'autre pair via le callback. */
  private emitSignal(description: RTCSessionDescription | RTCIceCandidate | null): void {
    if (!description) return;
    if (this.onSignalCallback) {
      this.onSignalCallback(description.toJSON());
    }
  }

  // -------------------------------------------------------------
  // CALLBACKS FOURNIS PAR L'APPLICATION
  // -------------------------------------------------------------

  /** Enregistre le callback déclenché lorsqu'un flux distant est reçu. */
  onStreamReceived(callback: (stream: MediaStream) => void): void {
    this.onStreamCallback = callback;
  }

  /** Enregistre le callback déclenché lorsqu'une donnée (JSON) est reçue. */
  onDataReceived(callback: (data: unknown) => void): void {
    this.onDataCallback = callback;
  }

  // -------------------------------------------------------------
  // CONTRÔLE DE L'APPEL
  // -------------------------------------------------------------

  /**
   * Démarre un appel en tant qu'initiateur (caller).
   * Ajoute le flux local au peer, puis retourne une promesse qui résout l'offre de signalling.
   *
   * @param stream - Flux local (caméra / micro) à partager.
   * @returns Une promesse résolue avec le payload d'offre à envoyer à l'autre pair.
   */
  async startCall(stream: MediaStream): Promise<unknown> {
    if (!this.pc) throw new Error('Peer not initialized');
    this.localStream = stream;

    // Ajouter les tracks locales
    stream.getTracks().forEach(track => {
      this.pc!.addTrack(track, stream);
    });

    // Créer l'offre
    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);

    // Attendre que l'ICE gathering soit complet
    return new Promise((resolve) => {
      const checkState = () => {
        if (this.pc!.iceGatheringState === 'complete') {
          resolve(this.pc!.localDescription!.toJSON());
        } else {
          setTimeout(checkState, 100);
        }
      };
      checkState();
    });
  }

  /**
   * Répond à une offre en tant que callee (receveur).
   * Ajoute le flux local au peer afin que le remote puisse recevoir votre audio/vidéo.
   *
   * @param stream - Flux local (caméra / micro) à partager.
   */
  async answerCall(stream: MediaStream): Promise<void> {
    if (!this.pc) throw new Error('Peer not initialized');
    this.localStream = stream;

    // Ajouter les tracks locales
    stream.getTracks().forEach(track => {
      this.pc!.addTrack(track, stream);
    });
  }

  /**
   * Envoie des données via le DataChannel.
   * @param data - Données à envoyer (sera converti en JSON).
   */
  sendData(data: unknown): void {
    if (this.dataChannel && this.dataChannel.readyState === 'open') {
      this.dataChannel.send(JSON.stringify(data));
    }
  }

  /**
   * Ferme proprement la connexion peer-to-peer.
   */
  close(): void {
    if (this.dataChannel) {
      this.dataChannel.close();
      this.dataChannel = null;
    }
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }
    if (this.pc) {
      this.pc.close();
      this.pc = null;
    }
    this.onStreamCallback = null;
    this.onDataCallback = null;
    this.onSignalCallback = null;
  }
}
