<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { initSodium, generateKeyPair, encryptMessage, decryptMessage } from '$lib/crypto';

  // États
  let input = $state('');
  let messages = $state<Array<{ id: string; content: string; sender: string; timestamp: string; reactions: string[] }>>([]);
  let myKeys = $state<{ publicKey: string; privateKey: string } | null>(null);
  let ws: WebSocket | null = null;
  let gifQuery = $state('');
  let gifResults = $state<any[]>([]);
  let showGifs = $state(false);
  let isConnected = $state(false);

  // Initialisation
  onMount(async () => {
    await initSodium();
    const stored = localStorage.getItem('nook-keys');
    if (stored) {
      myKeys = JSON.parse(stored);
    } else {
      myKeys = generateKeyPair();
      localStorage.setItem('nook-keys', JSON.stringify(myKeys));
    }

    // Connexion WebSocket
    ws = new WebSocket(`ws://${window.location.host}/ws`);
    ws.onopen = () => isConnected = true;
    ws.onclose = () => isConnected = false;
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

  // Envoi de message
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

  // Recherche de GIFs
  const searchGifs = async () => {
    if (!gifQuery.trim()) return;
    const res = await fetch(`/api/gifs?q=${encodeURIComponent(gifQuery)}`);
    const data = await res.json();
    gifResults = data.results || [];
  };

  // Envoi de GIF
  const sendGif = (url: string) => {
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

  // Ajout de réaction
  const addReaction = (index: number, emoji: string) => {
    messages[index].reactions.push(emoji);
  };

  // Gestion clavier
  const handleKeyUp = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };
</script>

<div class="flex flex-col h-screen bg-gray-50">
  <!-- En-tête -->
  <div class="bg-white border-b p-4 shadow-sm">
    <h1 class="text-2xl font-bold">Nook — Chat</h1>
    <div class="text-sm mt-1">
      {#if isConnected}
        <span class="text-green-600">● Connecté</span>
      {:else}
        <span class="text-yellow-600">○ Hors ligne</span>
      {/if}
    </div>
  </div>

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg, i}
      <div class={`flex ${msg.sender === 'Vous' ? 'justify-end' : 'justify-start'}`}>
        <div class={`max-w-xs md:max-w-md p-3 rounded-lg ${msg.sender === 'Vous' ? 'bg-blue-500 text-white rounded-br-none' : 'bg-white border rounded-bl-none'}`}>
          <div class="text-xs opacity-80 mb-1">{msg.sender} — {msg.timestamp}</div>
          <div>{msg.content}</div>
          <div class="mt-2 flex gap-1">
            {#each msg.reactions as reaction}
              <span>{reaction}</span>
            {/each}
            <button onclick={() => addReaction(i, '👍')} class="text-xs">👍</button>
            <button onclick={() => addReaction(i, '❤️')} class="text-xs">❤️</button>
            <button onclick={() => addReaction(i, '😂')} class="text-xs">😂</button>
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
      <button onclick={sendMessage} class="bg-green-500 text-white p-2 rounded">Envoyer</button>
      <button onclick={() => showGifs = !showGifs} class="bg-purple-500 text-white p-2 rounded">GIF</button>
    </div>
  </div>
</div>