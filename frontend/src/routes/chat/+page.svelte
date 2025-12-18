<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { initSodium, generateKeyPair, encryptMessage, decryptMessage } from '$lib/crypto';

  let input = $state('');
  let messages = $state([]);
  let myKeys = $state(null);
  let ws = null;
  let gifQuery = $state('');
  let gifResults = $state([]);
  let showGifs = $state(false);

  onMount(async () => {
    await initSodium();
    const stored = localStorage.getItem('nook-keys');
    if (stored) {
      myKeys = JSON.parse(stored);
    } else {
      myKeys = generateKeyPair();
      localStorage.setItem('nook-keys', JSON.stringify(myKeys));
    }

    ws = new WebSocket(`ws://${window.location.host}/ws`);
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'text' && myKeys) {
          const decrypted = decryptMessage(data.encrypted, data.senderPublicKey, myKeys.privateKey);
          messages = [...messages, {
            id: Date.now().toString(),
            content: decrypted,
            sender: data.sender,
            timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
            reactions: []
          }];
        }
      } catch (err) {
        console.error('Déchiffrement échoué:', err);
      }
    };
  });

  const sendMessage = () => {
    if (!input.trim() || !myKeys || !ws) return;
    const destPubKey = "DEST_PUBLIC_KEY"; // À remplacer par la vraie clé publique du groupe/famille
    const encrypted = encryptMessage(input, destPubKey, myKeys.privateKey);
    ws.send(JSON.stringify({
      type: 'text',
      encrypted,
      senderPublicKey: myKeys.publicKey,
      sender: localStorage.getItem('nook-name') || 'Vous'
    }));
    input = '';
  };

  const searchGifs = async () => {
    if (!gifQuery.trim()) return;
    const res = await fetch(`/api/gifs?q=${encodeURIComponent(gifQuery)}`);
    const data = await res.json();
    gifResults = data.results || [];
  };

  const sendGif = (url) => {
    if (!myKeys || !ws) return;
    const destPubKey = "DEST_PUBLIC_KEY";
    const encrypted = encryptMessage(url, destPubKey, myKeys.privateKey);
    ws.send(JSON.stringify({
      type: 'text',
      encrypted,
      senderPublicKey: myKeys.publicKey,
      sender: localStorage.getItem('nook-name') || 'Vous'
    }));
    showGifs = false;
    gifQuery = '';
  };

  const addReaction = (msgId, emoji) => {
    const msg = messages.find(m => m.id === msgId);
    if (msg) msg.reactions.push(emoji);
    messages = messages; // Trigger reactivity
  };
</script>

<div class="flex flex-col h-screen relative">
  <!-- Header glass -->
  <div class="p-5 backdrop-blur-xl bg-white/10 dark:bg-black/10 border-b border-white/20 dark:border-white/10 flex items-center justify-between">
    <div class="flex items-center gap-4">
      <div class="text-4xl">
        {#if $currentTheme === 'jardin-secret'}
          🌿
        {:else if $currentTheme === 'space-hub'}
          🚀
        {:else}
          🏠
        {/if}
      </div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">Chat familial</h1>
    </div>
  </div>

  <!-- Zone messages avec scroll smooth -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6 scrollbar-thin scrollbar-thumb-[var(--accent)/30]">
    {#each messages as msg (msg.id)}
      <div class={`flex ${msg.sender === localStorage.getItem('nook-name') || msg.sender === 'Vous' ? 'justify-end' : 'justify-start'} animate-fade-up`}>
        <div
          class={`max-w-xs md:max-w-md lg:max-w-lg rounded-3xl p-4 backdrop-blur-md shadow-lg transition-all hover:scale-[1.02] ${
            msg.sender === localStorage.getItem('nook-name') || msg.sender === 'Vous'
              ? 'bg-[var(--accent)/30] border border-[var(--accent)/40] text-white'
              : 'bg-white/20 dark:bg-black/20 border border-white/30'
          }`}
        >
          <div class="flex justify-between items-baseline mb-2">
            <span class="text-sm font-semibold opacity-80">{msg.sender}</span>
            <span class="text-xs opacity-60">{msg.timestamp}</span>
          </div>
          {#if msg.content.startsWith('https://media.tenor.com/')}
            <img src={msg.content} alt="GIF" class="rounded-xl max-w-full" />
          {:else}
            <p class="break-words">{msg.content}</p>
          {/if}
          <div class="mt-3 flex flex-wrap gap-2">
            {#each msg.reactions as reaction}
              <span class="inline-block animate-bounce-reaction text-lg">{reaction}</span>
            {/each}
            <div class="flex gap-1 ml-2">
              <button onclick={() => addReaction(msg.id, '👍')} class="text-lg opacity-60 hover:opacity-100 hover:scale-125 transition">👍</button>
              <button onclick={() => addReaction(msg.id, '❤️')} class="text-lg opacity-60 hover:opacity-100 hover:scale-125 transition">❤️</button>
              <button onclick={() => addReaction(msg.id, '😂')} class="text-lg opacity-60 hover:opacity-100 hover:scale-125 transition">😂</button>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Panel GIFs -->
  {#if showGifs}
    <div class="p-4 backdrop-blur-xl bg-white/10 dark:bg-black/10 border-t border-white/20">
      <input
        type="text"
        bind:value={gifQuery}
        placeholder="Rechercher un GIF..."
        class="w-full p-3 rounded-xl bg-white/20 dark:bg-black/20 border border-white/30 focus:outline-none focus:ring-2 focus:ring-[var(--accent)] mb-3"
        onkeydown={(e) => e.key === 'Enter' && searchGifs()}
      />
      <div class="grid grid-cols-3 md:grid-cols-4 gap-3 max-h-64 overflow-y-auto">
        {#each gifResults as gif}
          <button
            class="rounded-xl overflow-hidden shadow hover:shadow-xl hover:scale-105 transition-all"
            onclick={() => sendGif(gif.media[0].gif.url)}
          >
            <img src={gif.media[0].gif.url} alt="GIF" class="w-full h-auto" loading="lazy" />
          </button>
        {/each}
      </div>
      <button onclick={() => showGifs = false} class="mt-3 text-sm text-[var(--accent)]">Fermer</button>
    </div>
  {/if}

  <!-- Barre de saisie -->
  <div class="p-4 backdrop-blur-xl bg-white/10 dark:bg-black/10 border-t border-white/20">
    <div class="flex gap-3">
      <input
        type="text"
        bind:value={input}
        placeholder="Écrivez un message..."
        class="flex-1 p-4 rounded-2xl bg-white/20 dark:bg-black/20 border border-white/30 focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] transition-all"
        onkeydown={(e) => e.key === 'Enter' && !e.shiftKey && (e.preventDefault(), sendMessage())}
      />
      <button
        onclick={() => showGifs = !showGifs}
        aria-label="Rechercher un GIF"
        class="p-4 rounded-2xl bg-white/20 dark:bg-black/20 border border-white/30 hover:bg-white/30 transition"
      >
        GIF
      </button>
      <button
        onclick={sendMessage}
        aria-label="Envoyer le message"
        class="p-4 bg-[var(--accent)] text-white rounded-2xl hover:scale-105 transition"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      </button>
    </div>
  </div>

  <div class="absolute bottom-6 right-6">
    <ThemeSwitcher />
  </div>
</div>

<style>
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }
  @keyframes bounce-reaction {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.3); }
  }
  .animate-fade-up { animation: fade-up 0.5s ease-out; }
  .animate-bounce-reaction { animation: bounce-reaction 0.4s ease-out; }

  /* Scrollbar stylée */
  .scrollbar-thin::-webkit-scrollbar { width: 6px; }
  .scrollbar-thin::-webkit-scrollbar-thumb { background-color: rgba(255,255,255,0.3); border-radius: 3px; }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  }
</style>