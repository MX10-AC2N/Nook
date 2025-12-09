<script lang="ts">
  import { onMount } from 'svelte';
  import { initSodium, generateKeyPair, encryptMessage } from '$lib/crypto';

  let messages: { sender: string; text: string }[] = [];
  let input = '';
  let ws: WebSocket | null = null;
  let myKeys: { publicKey: string; privateKey: string } | null = null;

  onMount(async () => {
    await initSodium();
    myKeys = generateKeyPair();
    ws = new WebSocket('wss://nook.local/ws');
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      messages = [...messages, { sender: data.sender, text: data.text }];
    };
  });

  const send = () => {
    if (!input.trim() || !ws || !myKeys) return;
    const encrypted = encryptMessage(input, "DEST_PUBLIC_KEY", myKeys.privateKey);
    ws.send(JSON.stringify({ type: 'text', encrypted }));
    input = '';
  };
</script>

<div class="p-4">
  <div class="mb-4 h-96 overflow-y-auto">
    {#each messages as msg}
      <div class="mb-2">
        <strong>{msg.sender}:</strong> {msg.text}
      </div>
    {/each}
  </div>
  <input
    bind:value={input}
    on:keyup={(e: KeyboardEvent) => e.key === 'Enter' && send()}
    placeholder="Votre message..."
    class="w-full p-2 border rounded"
  />
  <button on:click={send} class="mt-2 bg-green-500 text-white p-2 rounded">
    Envoyer
  </button>
</div>