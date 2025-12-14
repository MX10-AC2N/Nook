<script>
  import { onMount, onDestroy } from 'svelte';
  import { WebRtcCall } from '$lib/webrtc-calls';

  let localStream = null;
  let remoteStream = null;
  let call = null;
  let isCalling = false;
  let isAnswering = false;

  const startCall = async () => {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
      call = new WebRtcCall(true);
      call.onStreamReceived(stream => remoteStream = stream);
      const offer = await call.startCall(localStream);
      // Envoyer via WebSocket
      const ws = new WebSocket(`wss://${window.location.host}/ws`);
      ws.send(JSON.stringify({ type: 'webrtc-offer', offer, to: 'DEST_ID' }));
      isCalling = true;
    } catch (err) {
      console.error('Erreur appel:', err);
    }
  };

  onMount(() => {
    const ws = new WebSocket(`wss://${window.location.host}/ws`);
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'webrtc-offer') {
        isAnswering = true;
        // Initialiser l'appel entrant
      }
    };
  });

  onDestroy(() => {
    if (call) call.close();
    if (localStream) localStream.getTracks().forEach(track => track.stop());
  });
</script>

<div class="flex flex-col h-screen bg-gray-900">
  {#if localStream}
    <video autoplay playsinline muted srcObject={localStream} class="w-32 h-32 absolute top-4 right-4 rounded border-2 border-white"></video>
  {/if}
  {#if remoteStream}
    <video autoplay playsinline srcObject={remoteStream} class="w-full h-full object-cover"></video>
  {:else if isCalling}
    <div class="flex-1 flex items-center justify-center text-white">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-16 w-16 border-b-2 border-white mb-4"></div>
        <p>Appel en cours...</p>
      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center">
      <button onclick={startCall} class="bg-green-500 hover:bg-green-600 text-white p-6 rounded-full text-2xl">
        📞 Appeler
      </button>
    </div>
  {/if}
</div>