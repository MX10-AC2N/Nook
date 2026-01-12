// src/lib/webrtc.ts
import Peer, { Instance as PeerInstance, Options as PeerOptions } from 'simple-peer';

/**
 * Wrapper autour de `simple-peer` qui simplifie la gestion d’un appel WebRTC.
 *
 * - `initiator` : true → crée l’offre (caller) ; false → répond à une offre (callee).
 * - Le flux local (`MediaStream`) est ajouté via `startCall` ou `answerCall`.
 * - Les callbacks `onSignal`, `onStreamReceived` et `onDataReceived` permettent
 *   d’interagir avec le signalling, le flux vidéo/audio et les messages de données.
 *
 * Toutes les méthodes sont typées pour éviter les erreurs de compilation.
 */
export class WebRtcCall {
  /** Instance interne de `simple-peer`. */
  private peer: PeerInstance | null = null;

  /** Callback appelé lorsqu’un flux distant est reçu. */
  private onStream: ((stream: MediaStream) => void) | null = null;

  /** Callback appelé lorsqu’une donnée (JSON) est reçue. */
  private onData: ((data: unknown) => void) | null = null;

  /**
   * Crée un nouveau wrapper WebRTC.
   *
   * @param initiator - true : le client crée l’offre (caller). false : il répond à une offre (callee).
   */
  constructor(initiator: boolean) {
    const options: PeerOptions = {
      initiator,
      trickle: false, // on préfère un signal complet (plus simple à gérer côté serveur)
    };

    this.peer = new Peer(options);

    // -------------------------------------------------------------
    // 1️⃣ Gestion du flux distant (vidéo / audio)
    // -------------------------------------------------------------
    this.peer.on('stream', (stream: MediaStream) => {
      if (this.onStream) this.onStream(stream);
    });

    // -------------------------------------------------------------
    // 2️⃣ Gestion des messages de données (JSON)
    // -------------------------------------------------------------
    this.peer.on('data', (data: Buffer) => {
      if (this.onData) {
        try {
          const parsed = JSON.parse(data.toString());
          this.onData(parsed);
        } catch {
          // Si le payload n’est pas du JSON valide, on transmet la donnée brute.
          this.onData(data);
        }
      }
    });
  }

  // -----------------------------------------------------------------
  // SIGNALING
  // -----------------------------------------------------------------
  /**
   * Applique le signal reçu du pair (offre, réponse ou ICE).
   *
   * @param data - Le payload de signalling (généralement un objet JSON).
   */
  signal(data: unknown): void {
    if (!this.peer) {
      throw new Error('Peer not initialized');
    }
    this.peer.signal(data);
  }

  /**
   * Enregistre un callback qui sera appelé chaque fois que le peer
   * génère un nouveau signal (offre, réponse ou ICE). Le callback reçoit
   * le payload à transmettre à l’autre participant (via votre serveur de signalling).
   *
   * @param callback - Fonction appelée avec le payload de signalling.
   */
  onSignal(callback: (data: unknown) => void): void {
    if (!this.peer) {
      throw new Error('Peer not initialized');
    }
    this.peer.on('signal', callback);
  }

  // -----------------------------------------------------------------
  // CALLBACKS FOURNIS PAR L’APPLICATION
  // -----------------------------------------------------------------
  /** Enregistre le callback déclenché lorsqu’un flux distant est reçu. */
  onStreamReceived(callback: (stream: MediaStream) => void): void {
    this.onStream = callback;
  }

  /** Enregistre le callback déclenché lorsqu’une donnée (JSON) est reçue. */
  onDataReceived(callback: (data: unknown) => void): void {
    this.onData = callback;
  }

  // -----------------------------------------------------------------
  // CONTRÔLE DE L’APPEL
  // -----------------------------------------------------------------
  /**
   * Démarre un appel en tant qu’initiateur (caller). Ajoute le flux local
   * au peer, puis retourne une promesse qui résout l’offre de signalling.
   *
   * @param stream - Flux local (caméra / micro) à partager.
   * @returns Une promesse résolue avec le payload d’offre à envoyer à l’autre pair.
   */
  async startCall(stream: MediaStream): Promise<unknown> {
    if (!this.peer) {
      throw new Error('Peer not initialized');
    }

    // Ajoute le flux local (audio/vidéo) au peer.
    this.peer.addStream(stream);

    // Retourne le signal (offre) dès qu’il est disponible.
    return new Promise<unknown>((resolve) => {
      this.peer!.once('signal', resolve);
    });
  }

  /**
   * Répond à une offre en tant que callee (receveur). Ajoute le flux local
   * au peer afin que le remote puisse recevoir votre audio/vidéo.
   *
   * @param stream - Flux local (caméra / micro) à partager.
   */
  answerCall(stream: MediaStream): void {
    if (!this.peer) {
      throw new Error('Peer not initialized');
    }
    this.peer.addStream(stream);
  }

  /**
   * Ferme proprement la connexion peer‑to‑peer.
   */
  close(): void {
    if (this.peer) {
      this.peer.destroy();
      this.peer = null;
    }
  }
}