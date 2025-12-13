import Peer from 'simple-peer';

export class WebRtcCall {
  private peer: Peer.Instance | null = null;
  private onStream: ((stream: MediaStream) => void) | null = null;
  private onData: ((data: any) => void) | null = null;

  constructor(initiator: boolean) {
    this.peer = new Peer({ initiator, trickle: false });
    this.peer.on('stream', (stream) => {
      if (this.onStream) this.onStream(stream);
    });
    this.peer.on('data', (data) => {
      if (this.onData) this.onData(JSON.parse(data.toString()));
    });
  }

  signal( any) {
    if (this.peer) this.peer.signal(data);
  }

  onSignal(callback: (data: any) => void) {
    if (this.peer) this.peer.on('signal', callback);
  }

  onStreamReceived(callback: (stream: MediaStream) => void) {
    this.onStream = callback;
  }

  onDataReceived(callback: ( any) => void) {
    this.onData = callback;
  }

  async startCall(stream: MediaStream) {
    if (this.peer) {
      this.peer.addStream(stream);
      const offer = await new Promise((resolve) => {
        this.peer!.once('signal', resolve);
      });
      return offer;
    }
  }

  answerCall(stream: MediaStream) {
    if (this.peer) this.peer.addStream(stream);
  }

  close() {
    if (this.peer) this.peer.destroy();
  }
}