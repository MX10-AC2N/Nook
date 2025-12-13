<script>
  import { onMount, onDestroy } from 'svelte';
  // Pas de TypeScript → supprime lang="ts"

  let localStream = null;
  let remoteStream = null;
  let call = null;
  let isCalling = false;
  let isAnswering = false;

  const startCall = async () => {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
      // Placeholder: appel WebRTC à implémenter
    } catch (err) {
      console.error('Erreur appel:', err);
    }
  };

  onMount(() => {
    // Écoute WebSocket pour les appels entrants
  });

  onDestroy(() => {
    if (call) call.close();
    if (localStream) {
      localStream.getTracks().forEach(track => track.stop());
    }
  });
</script>

<div class="flex flex-col h-screen bg-gray-900">
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
      <button on:click={startCall} class="bg-green-500 hover:bg-green-600 text-white p-6 rounded-full text-2xl">
        📞 Appeler
      </button>
    </div>
  {/if}
</div>