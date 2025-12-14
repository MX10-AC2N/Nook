<script>
  import { onMount, onDestroy } from 'svelte';
  import { initSodium, generateKeyPair, encryptMessage, decryptMessage } from '$lib/crypto';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';

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

  onDestroy(() => {
    if (ws) ws.close();
  });

  const sendMessage = () => {
    if (!input.trim() || !myKeys || !ws) return;
    const destPubKey = "DEST_PUBLIC_KEY"; // À remplacer par la clé du destinataire
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

  const handleKeyUp = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };
</script>

<div class="flex flex-col h-screen bg-gray-50">
  <!-- En-tête -->
  <div class="bg-white border-b p-4 shadow-sm">
    <h1 class="text-2xl font-bold text-gray-800">Chat — Nook</h1>
  </div>

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg, i}
      <div class={`flex ${msg.sender === 'Vous' ? 'justify-end' : 'justify-start'}`}>
        <div class={`max-w-xs md:max-w-md lg:max-w-lg rounded-lg p-3 ${msg.sender === 'Vous' ? 'bg-blue-500 text-white rounded-br-none' : 'bg-white border rounded-bl-none'}`}>
          <div class="flex justify-between items-baseline mb-1">
            <span class={`text-sm font-medium ${msg.sender === 'Vous' ? 'text-blue-100' : 'text-gray-700'}`}>
              {msg.sender}
            </span>
            <span class={`text-xs ml-2 ${msg.sender === 'Vous' ? 'text-blue-200' : 'text-gray-500'}`}>
              {msg.timestamp}
            </span>
          </div>
          <p class="break-words">{msg.content}</p>
          <div class="mt-2 flex gap-1">
            {#each msg.reactions as reaction}
              <span>{reaction}</span>
            {/each}
            <button onclick={() => addReaction(i, '👍')} class="text-xs">👍</button>
            <button onclick={() => addReaction(i, '❤️')} class="text-xs">❤️</button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Zone de saisie -->
  <div class="border-t bg-white p-4">
    {#if showGifs}
      <div class="mb-2">
        <input
          type="text"
          bind:value={gifQuery}
          onkeyup={(e) => e.key === 'Enter' && searchGifs()}
          placeholder="Rechercher un GIF..."
          class="w-full p-2 border rounded mb-2"
        />
        <div class="grid grid-cols-3 gap-2">
          {#each gifResults as gif}
            <img
              src={gif.media[0].gif.url}
              alt="GIF"
              onclick={() => sendGif(gif.media[0].gif.url)}
              class="cursor-pointer rounded"
            />
          {/each}
        </div>
        <button onclick={() => showGifs = false} class="mt-2 text-sm text-blue-500">Fermer les GIFs</button>
      </div>
    {/if}

    <div class="flex gap-2">
      <input
        type="text"
        bind:value={input}
        onkeyup={handleKeyUp}
        placeholder="Votre message..."
        class="flex-1 p-2 border rounded"
      />
      <button onclick={sendMessage} class="bg-green-500 text-white p-2 rounded">
        Envoyer
      </button>
      <button onclick={() => showGifs = !showGifs} class="bg-purple-500 text-white p-2 rounded">
        GIF
      </button>
    </div>
  </div>

  <!-- Sélecteur de thème -->
  <ThemeSwitcher />
</div>

<style>
  .messages-container {
    scrollbar-width: thin;
    scrollbar-color: #cbd5e0 #f7fafc;
  }
  
  .messages-container::-webkit-scrollbar {
    width: 8px;
  }
  
  .messages-container::-webkit-scrollbar-track {
    background: #f7fafc;
  }
  
  .messages-container::-webkit-scrollbar-thumb {
    background-color: #cbd5e0;
    border-radius: 4px;
  }
</style>