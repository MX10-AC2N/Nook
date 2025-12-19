<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '$app/navigation';
  import { initSodium, generateKeyPair, encryptMessage, decryptMessage } from '$lib/crypto';

  let input = $state('');
  let messages = $state([]);
  let myKeys = $state(null);
  let ws = null;
  let gifQuery = $state('');
  let gifResults = $state([]);
  let showGifs = $state(false);
  let currentUser = $state({ id: null, name: null });
  let connectionError = $state('');

  onMount(async () => {
    // 1. Vérifier la session et obtenir les infos utilisateur
    try {
      const sessionRes = await fetch('/api/validate-session', { credentials: 'include' });
      if (!sessionRes.ok) {
        goto('/');
        return;
      }
      const sessionData = await sessionRes.json();
      currentUser = { id: sessionData.member_id, name: sessionData.member_name };
    } catch (err) {
      console.error("Erreur de session:", err);
      goto('/');
      return;
    }

    // 2. Initialiser la cryptographie
    await initSodium();
    const storedKeys = localStorage.getItem(`nook_keys_${currentUser.id}`);
    if (storedKeys) {
      myKeys = JSON.parse(storedKeys);
    } else {
      myKeys = generateKeyPair();
      localStorage.setItem(`nook_keys_${currentUser.id}`, JSON.stringify(myKeys));
    }

    // 3. Établir la connexion WebSocket
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
      console.log('WebSocket connecté avec authentification cookie.');
      connectionError = '';
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.from && data.content) {
          messages = [...messages, {
            id: `${data.from}_${Date.now()}`,
            content: data.content,
            sender: data.from_name,
            senderId: data.from,
            timestamp: new Date(data.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
            reactions: []
          }];
        }
      } catch (err) {
        console.error('Erreur traitement message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      connectionError = 'Problème de connexion au chat.';
    };

    ws.onclose = () => {
      console.log('WebSocket déconnecté.');
    };
  });

  const sendMessage = () => {
    if (!input.trim() || !ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(input);
    input = '';
  };

  const searchGifs = async () => {
    if (!gifQuery.trim()) return;
    const res = await fetch(`/api/gifs?q=${encodeURIComponent(gifQuery)}`);
    const data = await res.json();
    gifResults = data.results || [];
  };

  const sendGif = (url) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(`[GIF] ${url}`);
    showGifs = false;
    gifQuery = '';
  };

  const addReaction = (msgId, emoji) => {
    const msg = messages.find(m => m.id === msgId);
    if (msg) {
      msg.reactions.push(emoji);
      messages = messages;
    }
  };
</script>

<div class="flex flex-col h-screen relative">
  <!-- Header avec info utilisateur -->
  <div class="p-4 backdrop-blur-xl bg-white/10 dark:bg-black/10 border-b border-white/20 flex items-center justify-between">
    <div class="flex items-center gap-4">
      <div class="text-3xl">
        {#if $currentTheme === 'jardin-secret'}
          🌿
        {:else if $currentTheme === 'space-hub'}
          🚀
        {:else}
          🏠
        {/if}
      </div>
      <div>
        <h1 class="text-xl font-bold text-[var(--text-primary)]">Chat familial</h1>
        {#if currentUser.name}
          <p class="text-sm text-[var(--text-secondary)]">Connecté en tant que <strong>{currentUser.name}</strong></p>
        {/if}
      </div>
    </div>
    {#if connectionError}
      <div class="text-sm text-red-400 bg-red-900/20 px-3 py-1 rounded">
        {connectionError}
      </div>
    {/if}
  </div>

  <!-- Zone messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg (msg.id)}
      <div class={`flex ${msg.senderId === currentUser.id ? 'justify-end' : 'justify-start'}`}>
        <div class={`max-w-xs md:max-w-md rounded-2xl p-3 backdrop-blur-md ${msg.senderId === currentUser.id ? 'bg-[var(--accent)] text-white' : 'bg-white/20'}`}>
          <div class="flex justify-between items-baseline mb-1">
            <span class="text-sm font-semibold">{msg.sender}</span>
            <span class="text-xs opacity-70">{msg.timestamp}</span>
          </div>
          {#if msg.content.startsWith('[GIF]')}
            <img src={msg.content.slice(5)} alt="GIF" class="rounded-xl max-w-full" />
          {:else}
            <p class="break-words">{msg.content}</p>
          {/if}
          <div class="mt-2 flex gap-1">
            <button onclick={() => addReaction(msg.id, '👍')} class="text-lg hover:scale-125 transition">👍</button>
            <button onclick={() => addReaction(msg.id, '❤️')} class="text-lg hover:scale-125 transition">❤️</button>
            <button onclick={() => addReaction(msg.id, '😂')} class="text-lg hover:scale-125 transition">😂</button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Panel GIFs -->
  {#if showGifs}
    <div class="p-4 backdrop-blur-xl bg-white/10 border-t border-white/20">
      <input type="text" bind:value={gifQuery} placeholder="Rechercher un GIF..." onkeydown={(e) => e.key === 'Enter' && searchGifs()} class="w-full p-3 rounded-xl bg-white/20 border border-white/30 mb-3" />
      <div class="grid grid-cols-2 md:grid-cols-4 gap-2 max-h-48 overflow-y-auto">
        {#each gifResults as gif}
          <button onclick={() => sendGif(gif.media[0].gif.url)} class="rounded-xl overflow-hidden">
            <img src={gif.media[0].gif.url} alt="GIF" class="w-full h-auto" loading="lazy" />
          </button>
        {/each}
      </div>
      <button onclick={() => showGifs = false} class="mt-2 text-sm text-[var(--accent)]">Fermer</button>
    </div>
  {/if}

  <!-- Barre de saisie -->
  <div class="p-4 backdrop-blur-xl bg-white/10 border-t border-white/20">
    <div class="flex gap-2">
      <input type="text" bind:value={input} placeholder="Écrivez un message..." onkeydown={(e) => e.key === 'Enter' && sendMessage()} class="flex-1 p-3 rounded-xl bg-white/20 border border-white/30" />
      <button onclick={() => showGifs = !showGifs} class="p-3 rounded-xl bg-white/20 border border-white/30">GIF</button>
      <button onclick={sendMessage} class="p-3 bg-[var(--accent)] text-white rounded-xl">Envoyer</button>
    </div>
  </div>

  <!-- ThemeSwitcher -->
  <div class="absolute bottom-6 right-6">
    <ThemeSwitcher />
  </div>
</div>

<style>
  /* Conserve tes styles d'animation existants si tu les veux */
</style>