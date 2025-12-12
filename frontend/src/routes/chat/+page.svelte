<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  // États réactifs avec Svelte 5
  let input = $state('');
  let messages = $state<Array<{id: string; content: string; sender: string; timestamp: string}>>([]);
  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let isConnected = $state(false);
  
  // Pour les connexions WebSocket (optionnel)
  let socket: WebSocket | null = null;

  // Charger les messages au démarrage
  const loadMessages = async () => {
    try {
      isLoading = true;
      error = null;
      const res = await fetch('/api/messages');
      
      if (!res.ok) {
        throw new Error(`Erreur HTTP: ${res.status}`);
      }
      
      const data = await res.json();
      messages = data.messages || [];
    } catch (err) {
      error = `Erreur de chargement: ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur loadMessages:', err);
    } finally {
      isLoading = false;
    }
  };

  // Envoyer un message
  const sendMessage = async () => {
    if (!input.trim()) return;
    
    const content = input.trim();
    const tempId = `temp-${Date.now()}`;
    
    // Ajouter temporairement le message
    const tempMessage = {
      id: tempId,
      content,
      sender: 'Vous',
      timestamp: new Date().toLocaleTimeString()
    };
    
    messages = [...messages, tempMessage];
    input = '';
    
    try {
      error = null;
      const res = await fetch('/api/messages', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content })
      });
      
      if (!res.ok) {
        throw new Error(`Erreur HTTP: ${res.status}`);
      }
      
      const data = await res.json();
      
      // Remplacer le message temporaire par le message réel
      messages = messages.map(msg => 
        msg.id === tempId ? { ...data.message, sender: 'Vous' } : msg
      );
      
    } catch (err) {
      error = `Erreur d'envoi: ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur sendMessage:', err);
      
      // Supprimer le message temporaire en cas d'erreur
      messages = messages.filter(msg => msg.id !== tempId);
      input = content; // Restaurer le texte
    }
  };

  // Gestion de l'événement Enter
  const handleKeyUp = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  // Initialisation WebSocket (optionnel - pour le temps réel)
  const initWebSocket = () => {
    try {
      socket = new WebSocket(`ws://${window.location.host}/ws`);
      
      socket.onopen = () => {
        console.log('WebSocket connecté');
        isConnected = true;
      };
      
      socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.type === 'new_message') {
            messages = [...messages, data.message];
          }
        } catch (err) {
          console.error('Erreur parsing WebSocket message:', err);
        }
      };
      
      socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        isConnected = false;
      };
      
      socket.onclose = () => {
        console.log('WebSocket déconnecté');
        isConnected = false;
        // Tentative de reconnexion après 5 secondes
        setTimeout(initWebSocket, 5000);
      };
    } catch (err) {
      console.error('Erreur initialisation WebSocket:', err);
    }
  };

  // Scroll automatique vers le dernier message
  const scrollToBottom = () => {
    const container = document.querySelector('.messages-container');
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  };

  // Effet pour scroller quand les messages changent
  $effect(() => {
    if (messages.length > 0) {
      scrollToBottom();
    }
  });

  // Initialisation au montage
  onMount(() => {
    loadMessages();
    // Décommentez si vous utilisez WebSocket :
    // initWebSocket();
  });

  // Nettoyage à la destruction
  onDestroy(() => {
    if (socket) {
      socket.close();
    }
  });
</script>

<div class="flex flex-col h-screen bg-gray-50">
  <!-- En-tête -->
  <div class="bg-white border-b p-4 shadow-sm">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-800">Chat — Nook</h1>
      <div class="flex items-center gap-4">
        {#if isConnected}
          <div class="flex items-center">
            <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
            <span class="text-sm text-gray-600">En ligne</span>
          </div>
        {:else}
          <span class="text-sm text-yellow-600">Hors ligne</span>
        {/if}
        <button 
          onclick={loadMessages} 
          disabled={isLoading}
          class="text-sm bg-gray-100 hover:bg-gray-200 px-3 py-1 rounded transition-colors disabled:opacity-50"
        >
          Actualiser
        </button>
      </div>
    </div>
  </div>

  <!-- Messages d'erreur -->
  {#if error}
    <div class="mx-4 mt-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded flex justify-between items-center">
      <span>{error}</span>
      <button onclick={() => error = null} class="text-red-800 font-bold text-lg">
        ×
      </button>
    </div>
  {/if}

  <!-- Zone de messages -->
  <div class="messages-container flex-1 overflow-y-auto p-4 space-y-4">
    {#if isLoading && messages.length === 0}
      <div class="flex justify-center items-center h-full">
        <div class="text-center">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900 mb-2"></div>
          <p class="text-gray-500">Chargement des messages...</p>
        </div>
      </div>
    {:else if messages.length === 0}
      <div class="flex justify-center items-center h-full">
        <div class="text-center text-gray-500">
          <svg class="w-16 h-16 mx-auto mb-4 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <p class="text-lg">Aucun message</p>
          <p class="text-sm">Soyez le premier à envoyer un message !</p>
        </div>
      </div>
    {:else}
      {#each messages as message (message.id)}
        <div class={`flex ${message.sender === 'Vous' ? 'justify-end' : 'justify-start'}`}>
          <div class={`max-w-xs md:max-w-md lg:max-w-lg rounded-lg p-3 ${message.sender === 'Vous' ? 'bg-blue-500 text-white rounded-br-none' : 'bg-white border rounded-bl-none'}`}>
            <div class="flex justify-between items-baseline mb-1">
              <span class={`text-sm font-medium ${message.sender === 'Vous' ? 'text-blue-100' : 'text-gray-700'}`}>
                {message.sender}
              </span>
              <span class={`text-xs ml-2 ${message.sender === 'Vous' ? 'text-blue-200' : 'text-gray-500'}`}>
                {message.timestamp}
              </span>
            </div>
            <p class="break-words">{message.content}</p>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Zone de saisie -->
  <div class="border-t bg-white p-4">
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={input}
        onkeyup={handleKeyUp}
        placeholder="Tapez votre message..."
        class="flex-1 p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        disabled={isLoading}
      />
      <button
  onclick={sendMessage}
  disabled={isLoading || !input.trim()}
  class="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center gap-2"
>
  {#if isLoading}
    <div class="flex items-center gap-2">
      <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
      <span>Envoi...</span>
    </div>
  {:else}
    <div class="flex items-center gap-2">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
      </svg>
      <span>Envoyer</span>
    </div>
  {/if}
</button>
    </div>
    <div class="mt-2 text-xs text-gray-500 flex justify-between">
      <div>
        Appuyez sur <kbd class="px-1 py-0.5 bg-gray-100 rounded">Enter</kbd> pour envoyer
      </div>
      <div>{input.length}/500 caractères</div>
    </div>
  </div>
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
  
  .messages-container::-webkit-scrollbar-thumb:hover {
    background-color: #a0aec0;
  }
  
  /* Animation pour les nouveaux messages */
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .messages-container > div {
    animation: fadeIn 0.3s ease-out;
  }
</style>