import Peer from 'simple-peer';

export class WebRtcManager {
  private peer: Peer.Instance | null = null;
  private onReceive: (( Uint8Array) => void) | null = null;

  constructor(initiator: boolean) {
    this.peer = new Peer({ initiator, trickle: false });
  }

  signal( any) {
    if (this.peer) this.peer.signal(data);
  }

  onSignal(callback: (data: any) => void) {
    if (this.peer) this.peer.on('signal', callback);
  }

  onData(callback: (data: Uint8Array) => void) {
    this.onReceive = callback;
    if (this.peer) this.peer.on('data', callback);
  }

  send( Uint8Array) {
    if (this.peer) this.peer.send(data);
  }

  destroy() {
    if (this.peer) this.peer.destroy();
  }
}