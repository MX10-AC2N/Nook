<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
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
            timestamp: new Date().toLocaleTimeString(),
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
    const destPubKey = "DEST_PUBLIC_KEY";
    const encrypted = encryptMessage(input, destPubKey, myKeys.privateKey);
    ws.send(JSON.stringify({
      type: 'text',
      encrypted,
      senderPublicKey: myKeys.publicKey,
      sender: 'Vous'
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
      sender: 'Vous'
    }));
    showGifs = false;
  };

  const addReaction = (index, emoji) => {
    messages[index].reactions.push(emoji);
  };
</script>

<div class="flex flex-col h-screen bg-[var(--bg-primary)]">
  <!-- En-tête -->
  <div class="p-4 bg-[var(--bg-secondary)] border-b border-[var(--border)]">
    <h1 class="text-2xl font-bold text-[var(--text-primary)]">Chat — Nook</h1>
  </div>

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg, i}
      <div class={`flex ${msg.sender === 'Vous' ? 'justify-end' : 'justify-start'}`}>
        <div
          class={`max-w-xs md:max-w-md lg:max-w-lg rounded-2xl p-4 transition-all duration-300 {msg.sender === 'Vous' ? 'bg-[var(--chat-mine)] text-white' : 'bg-[var(--chat-theirs)] text-[var(--text-primary)]'} shadow-[var(--depth)]`}
        >
          <div class="flex justify-between items-baseline mb-1">
            <span class={`text-sm font-medium {msg.sender === 'Vous' ? 'text-[var(--chat-mine)]/80' : 'text-[var(--text-secondary)]'}`}>
              {msg.sender}
            </span>
            <span class={`text-xs ml-2 {msg.sender === 'Vous' ? 'text-[var(--chat-mine)]/60' : 'text-[var(--text-secondary)]'}`}>
              {msg.timestamp}
            </span>
          </div>
          <p class="break-words">{msg.content}</p>
          <div class="mt-2 flex gap-1">
            {#each msg.reactions as reaction}
              <span class="text-sm">{reaction}</span>
            {/each}
            <button onclick={() => addReaction(i, '👍')} class="text-xs opacity-70 hover:opacity-100">👍</button>
            <button onclick={() => addReaction(i, '❤️')} class="text-xs opacity-70 hover:opacity-100">❤️</button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- GIFs -->
  {#if showGifs}
    <div class="p-4 border-t border-[var(--border)] bg-[var(--bg-secondary)]">
      <input
        type="text"
        bind:value={gifQuery}
        placeholder="Rechercher un GIF..."
        class="w-full p-2 rounded-lg border border-[var(--border)] bg-[var(--input-bg)] text-[var(--text-primary)] mb-2"
        onkeydown={(e) => e.key === 'Enter' && searchGifs()}
      />
      <div class="grid grid-cols-3 gap-2">
        {#each gifResults as gif}
          <button
            class="p-0 bg-transparent border-0 cursor-pointer rounded-lg overflow-hidden shadow hover:shadow-lg transition-all"
            onclick={() => sendGif(gif.media[0].gif.url)}
          >
            <img src={gif.media[0].gif.url} alt="GIF" class="w-full h-auto" />
          </button>
        {/each}
      </div>
      <button
        onclick={() => showGifs = false}
        class="mt-2 text-sm text-[var(--accent)]"
      >
        Fermer
      </button>
    </div>
  {/if}

  <!-- Saisie -->
  <div class="p-4 border-t border-[var(--border)] bg-[var(--bg-secondary)]">
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={input}
        placeholder="Votre message..."
        class="flex-1 p-3 rounded-xl border border-[var(--border)] bg-[var(--input-bg)] text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
        onkeydown={(e) => e.key === 'Enter' && !e.shiftKey && (e.preventDefault(), sendMessage())}
      />
      <button
        onclick={sendMessage}
        class="p-3 bg-[var(--accent)] text-white rounded-xl hover:bg-[var(--button-hover)] transition-colors"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      </button>
      <button
        onclick={() => showGifs = !showGifs}
        class="p-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-xl hover:bg-[var(--bg-secondary)] transition-colors"
      >
        GIF
      </button>
    </div>
  </div>

  <ThemeSwitcher />
</div>