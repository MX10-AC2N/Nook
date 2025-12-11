<script lang="ts">
  import { onMount } from 'svelte';
  import { initSodium, generateKeyPair, encryptMessage, decryptMessage } from '$lib/crypto';

  let messages: { sender: string; text: string; timestamp: Date }[] = $state([]);
  let input = $state('');
  let ws: WebSocket | null = null;
  let myKeys: { publicKey: string; privateKey: string } | null = $state(null);

  onMount(async () => {
    await initSodium();
    const storedKeys = localStorage.getItem('nook-keys');
    if (storedKeys) {
      myKeys = JSON.parse(storedKeys);
    } else {
      myKeys = generateKeyPair();
      localStorage.setItem('nook-keys', JSON.stringify(myKeys));
    }

    ws = new WebSocket('ws://localhost:3000/ws'); // Remplace par ton URL
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'text') {
        const decrypted = decryptMessage(data.encrypted, data.senderPublicKey, myKeys!.privateKey);
        messages = [...messages, { sender: data.sender, text: decrypted, timestamp: new Date() }];
      }
    };
  });

  const send = () => {
    if (!input.trim() || !ws || !myKeys) return;
    // Pour cet exemple, on suppose que la clé publique du destinataire est connue
    const recipientPublicKey = "DEST_PUBLIC_KEY";
    const encrypted = encryptMessage(input, recipientPublicKey, myKeys.privateKey);
    ws.send(JSON.stringify({ type: 'text', encrypted, senderPublicKey: myKeys.publicKey }));
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
    onkeyup={(e: KeyboardEvent) => e.key === 'Enter' && send()}
    placeholder="Votre message..."
    class="w-full p-2 border rounded"
  />
  <button onclick={send} class="mt-2 bg-green-500 text-white p-2 rounded">
    Envoyer
  </button>
</div>