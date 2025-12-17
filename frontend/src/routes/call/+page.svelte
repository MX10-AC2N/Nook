<script>
  import { onMount, onDestroy } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { WebRtcCall } from '$lib/webrtc-calls';

  let localStream = null;
  let remoteStream = null;
  let call = null;
  let isCalling = false;
  let isAnswering = false;
  let isMuted = false;
  let isVideoOff = false;

  const startCall = async () => {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
      call = new WebRtcCall(true);
      call.onStreamReceived(stream => remoteStream = stream);
      const offer = await call.startCall(localStream);
      // TODO: Envoyer l'offer via WebSocket signaling (à implémenter pleinement)
      const ws = new WebSocket(`ws://${window.location.host}/ws`);
      ws.onopen = () => {
        ws.send(JSON.stringify({ type: 'webrtc-offer', offer }));
      };
      isCalling = true;
    } catch (err) {
      console.error('Erreur démarrage appel:', err);
      alert('Impossible d\'accéder à la caméra/micro. Vérifiez les permissions.');
    }
  };

  const toggleMute = () => {
    if (localStream) {
      localStream.getAudioTracks()[0].enabled = !isMuted;
      isMuted = !isMuted;
    }
  };

  const toggleVideo = () => {
    if (localStream) {
      localStream.getVideoTracks()[0].enabled = !isVideoOff;
      isVideoOff = !isVideoOff;
    }
  };

  const hangUp = () => {
    if (call) call.close();
    if (localStream) localStream.getTracks().forEach(track => track.stop());
    localStream = null;
    remoteStream = null;
    isCalling = false;
    isAnswering = false;
  };

  onMount(() => {
    // Gestion des appels entrants (à compléter avec signaling réel)
    const ws = new WebSocket(`ws://${window.location.host}/ws`);
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'webrtc-offer' && !isCalling) {
        // Logique pour accepter l'appel entrant (ex: auto-accept pour famille)
        // Pour demo, on simule
        isAnswering = true;
      }
    };
  });

  onDestroy(hangUp);
</script>

<div class="relative w-full h-screen overflow-hidden">
  <!-- Vidéo distante full-screen -->
  {#if remoteStream}
    <video
      autoplay
      playsinline
      bind:this={remoteStream}
      class="absolute inset-0 w-full h-full object-cover"
    ><video/>
  {:else if isCalling || isAnswering}
    <!-- Écran d'attente immersif -->
    <div class="absolute inset-0 flex items-center justify-center">
      <div class="text-center">
        <div class="inline-block animate-pulse-rounded h-32 w-32 border-8 border-[var(--accent)/50] rounded-full mb-8"></div>
        <p class="text-3xl font-medium text-white opacity-90">
          {isCalling ? 'Appel en cours...' : 'Appel entrant...'}
        </p>
        <p class="mt-4 text-xl text-white/70">Connexion sécurisée P2P</p>
      </div>
    </div>
  {:else}
    <!-- Écran d'accueil appel -->
    <div class="absolute inset-0 flex items-center justify-center">
      <button
        onclick={startCall}
        class="p-12 bg-[var(--accent)] text-white rounded-full shadow-2xl hover:shadow-[0_0_60px_var(--accent)] hover:scale-110 transition-all duration-500 animate-pulse-slow"
      >
        <span class="text-6xl">
          {#if $currentTheme === 'jardin-secret'}
            🌿📞
          {:else if $currentTheme === 'space-hub'}
            🚀📞
          {:else}
            🏠📞
          {/if}
        </span>
      </button>
    </div>
  {/if}

  <!-- Vidéo locale PiP glass -->
  {#if localStream}
    <video
      autoplay
      playsinline
      muted
      bind:this={localStream}
      class="absolute top-6 right-6 w-64 h-64 md:w-80 md:h-80 object-cover rounded-3xl border-4 border-white/40 shadow-2xl backdrop-blur-md"
    ><video/>
  {/if}

  <!-- Barre de contrôles glass (visible en appel) -->
  {#if localStream || remoteStream}
    <div class="absolute bottom-8 left-1/2 -translate-x-1/2 flex gap-6 p-6 bg-white/10 dark:bg-black/10 backdrop-blur-2xl border border-white/20 rounded-full shadow-2xl">
      <button
        onclick={toggleMute}
        class={`p-5 rounded-full transition-all ${isMuted ? 'bg-red-500/70' : 'bg-white/20'} hover:scale-110`}
      >
        {isMuted ? '🔇' : '🎤'}
      </button>

      <button
        onclick={toggleVideo}
        class={`p-5 rounded-full transition-all ${isVideoOff ? 'bg-red-500/70' : 'bg-white/20'} hover:scale-110`}
      >
        {isVideoOff ? '📹⃠' : '📹'}
      </button>

      <button
        onclick={hangUp}
        class="p-5 bg-red-500/80 hover:bg-red-600 rounded-full hover:scale-110 transition-all"
      >
        📞
      </button>
    </div>
  {/if}
</div>

<style>
  @keyframes pulse-rounded {
    0%, 100% { box-shadow: 0 0 0 0 rgba(var(--accent-rgb), 0.4); }
    70% { box-shadow: 0 0 0 40px rgba(var(--accent-rgb), 0); }
  }

  @keyframes pulse-slow {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.05); }
  }

  .animate-pulse-rounded { animation: pulse-rounded 2s infinite; }
  .animate-pulse-slow { animation: pulse-slow 3s infinite ease-in-out; }

  /* Support CSS var pour accent si besoin */
  :global(body) { --accent-rgb: 74, 144, 226; } /* Ajuste selon tes thèmes */
</style>