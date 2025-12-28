import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './authStore';
import { encryptForRecipients, decryptMessage, getStoredKeys, decryptPrivateKey } from './crypto';
import type { Message } from './types';

// Types
export interface ChatState {
  messages: Message[];
  connectionError: string | null;
  gifResults: any[];
  showGifs: boolean;
  gifLoading: boolean;
}

// État du chat avec stores Svelte
function createChatStore() {
  const initialState: ChatState = {
    messages: [],
    connectionError: null,
    gifResults: [],
    showGifs: false,
    gifLoading: false
  };

  const { subscribe, set, update } = writable<ChatState>(initialState);

  return {
    subscribe,
    setMessages: (messages: Message[]) => update(state => ({ ...state, messages })),
    addMessage: (message: Message) => update(state => ({
      ...state,
      messages: [...state.messages, message]
    })),
    setConnectionError: (error: string | null) => update(state => ({ ...state, connectionError: error })),
    setGifResults: (results: any[]) => update(state => ({ ...state, gifResults: results })),
    toggleGifs: () => update(state => {
      const newShowGifs = !state.showGifs;
      return { ...state, showGifs: newShowGifs, gifResults: newShowGifs ? state.gifResults : [] };
    }),
    setGifLoading: (loading: boolean) => update(state => ({ ...state, gifLoading: loading })),
    reset: () => set(initialState)
  };
}

export const chatStore = createChatStore();

// Stores individuels pour l'accès facile
export const messages = derived(chatStore, $store => $store.messages);
export const connectionError = derived(chatStore, $store => $store.connectionError);
export const gifResults = derived(chatStore, $store => $store.gifResults);
export const showGifs = derived(chatStore, $store => $store.showGifs);
export const gifLoading = derived(chatStore, $store => $store.gifLoading);

// Fonctions utilitaires
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
  return date.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
}

// Actions
export async function loadMessages(conversationId: string) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}/messages`, { credentials: 'include' });
    if (!response.ok) throw new Error('Impossible de charger les messages');
    const data = await response.json();
    chatStore.setMessages(data.messages || []);
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError('Erreur de chargement des messages');
    console.error('Erreur chargement messages:', err);
  }
}

export async function sendMessage(
  content: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
  try {
    const encrypted = await encryptForRecipients(content, recipientPublicKeys, senderPrivateKey);
    const messageData = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [userId, Array.from(key)])
      ),
      nonce: Array.from(encrypted.nonce),
      media_type: null,
      media_url: null
    };
    const response = await fetch('/api/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(messageData)
    });
    if (!response.ok) throw new Error("Erreur lors de l'envoi du message");
    await loadMessages(conversationId);
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError("Erreur lors de l'envoi du message");
    console.error('Erreur envoi message:', err);
  }
}

export async function sendGif(
  gifUrl: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
  try {
    const encrypted = await encryptForRecipients(`[GIF]${gifUrl}`, recipientPublicKeys, senderPrivateKey);
    const messageData = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [userId, Array.from(key)])
      ),
      nonce: Array.from(encrypted.nonce),
      media_type: 'gif',
      media_url: gifUrl
    };
    const response = await fetch('/api/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(messageData)
    });
    if (!response.ok) throw new Error("Erreur lors de l'envoi du GIF");
    await loadMessages(conversationId);
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError("Erreur lors de l'envoi du GIF");
    console.error('Erreur envoi GIF:', err);
  }
}

export async function searchGifs(query: string) {
  try {
    chatStore.setGifLoading(true);
    chatStore.setGifResults([]);
    const response = await fetch(
      `https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(query)}&key=LIVDSRZULELA&client_key=nook&limit=12`
    );
    if (!response.ok) throw new Error('Erreur lors de la recherche de GIFs');
    const data = await response.json();
    chatStore.setGifResults(data.results || []);
    chatStore.setGifLoading(false);
  } catch (err) {
    chatStore.setGifLoading(false);
    console.error('Erreur recherche GIFs:', err);
  }
}

export async function addReaction(messageId: string, emoji: string) {
  try {
    const response = await fetch('/api/messages/reaction', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ message_id: messageId, emoji })
    });
    if (!response.ok) throw new Error("Erreur lors de l'ajout de la réaction");
    chatStore.setMessages(
      get(messages).map(msg => {
        if (msg.id === messageId) {
          const reactions = { ...msg.reactions } || {};
          reactions[emoji] = (reactions[emoji] || 0) + 1;
          return { ...msg, reactions };
        }
        return msg;
      })
    );
  } catch (err) {
    console.error('Erreur ajout réaction:', err);
  }
}

export async function decryptMessageContent(
  message: Message,
  privateKey: Uint8Array,
  senderPublicKey: Uint8Array
): Promise<string> {
  try {
    if (message.media_type === 'gif') return `[GIF]${message.media_url}`;
    const encryptedContent = new Uint8Array(message.content);
    const userId = get(authStore).user?.id || '';
    const encryptedKeyData = new Uint8Array(message.encrypted_keys[userId] || []);
    const nonce = new Uint8Array(message.nonce);
    return await decryptMessage(encryptedContent, encryptedKeyData, senderPublicKey, privateKey, nonce);
  } catch (err) {
    console.error('Erreur déchiffrement message:', err);
    return '[Message illisible]';
  }
}

export async function initUserKeys() {
  const user = get(authStore).user;
  if (!user) return null;
  const storedKeys = await getStoredKeys(user.id);
  if (!storedKeys) return null;
  try {
    const password = user.password || (browser && prompt('Entrez votre mot de passe pour déchiffrer vos messages:'));
    if (!password) return null;
    const privateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
    return { privateKey, publicKey: storedKeys.publicKey };
  } catch (err) {
    chatStore.setConnectionError('Erreur de déchiffrement des clés - vérifiez votre mot de passe');
    console.error('Erreur déchiffrement clés:', err);
    return null;
  }
}
