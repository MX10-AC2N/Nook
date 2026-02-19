/**
 * Store et actions du chat — Svelte 5 runes.
 *
 * - Gestion du tableau de messages (chargement, ajout, réactions).
 * - Gestion de l'interface GIF (recherche, affichage, toggle).
 * - Chiffrement / déchiffrement des messages via crypto.ts.
 * - Gestion des erreurs de connexion.
 */

import { browser } from '$app/environment';
import { authStore } from './authStore.svelte.js';
import {
  encryptForRecipients,
  decryptMessage,
  getStoredKeys,
  decryptPrivateKey,
} from './crypto';
import type { Message } from './types';

// -----------------------------------------------------------------
// 1️⃣ Interface et état réactif principal
// -----------------------------------------------------------------
export interface ChatState {
  messages: Message[];
  connectionError: string | null;
  gifResults: any[];
  showGifs: boolean;
  gifLoading: boolean;
}

function createInitialState(): ChatState {
  return {
    messages: [],
    connectionError: null,
    gifResults: [],
    showGifs: false,
    gifLoading: false,
  };
}

/** État principal du chat — objet $state unique, jamais réassigné */
export const chatStore = $state<ChatState>(createInitialState());

// -----------------------------------------------------------------
// 2️⃣ Mutateurs d'état
// -----------------------------------------------------------------
export function setMessages(msgs: Message[]): void {
  chatStore.messages = msgs;
}

export function addMessage(message: Message): void {
  chatStore.messages = [...chatStore.messages, message];
}

export function setConnectionError(error: string | null): void {
  chatStore.connectionError = error;
}

export function setGifResults(results: any[]): void {
  chatStore.gifResults = results;
}

export function toggleGifs(): void {
  chatStore.showGifs = !chatStore.showGifs;
  if (!chatStore.showGifs) chatStore.gifResults = [];
}

export function setGifLoading(loading: boolean): void {
  chatStore.gifLoading = loading;
}

export function resetChat(): void {
  Object.assign(chatStore, createInitialState());
}

// -----------------------------------------------------------------
// 3️⃣ Accesseurs nommés — compatibilité avec chat/+page.svelte
//
// chat/+page.svelte importe : messages, showGifs, gifResults, gifLoading
// et les utilise comme : `messages.length`, `{#if $showGifs}`, etc.
//
// On expose des objets avec .subscribe() pour que $showGifs fonctionne
// dans les templates Svelte (auto-subscription).
// -----------------------------------------------------------------

/** Liste des messages — utilisable directement : messages.length, {#each messages} */
export const messages = {
  get length() { return chatStore.messages.length; },
  [Symbol.iterator]() { return chatStore.messages[Symbol.iterator](); },
  subscribe(fn: (v: Message[]) => void) {
    fn(chatStore.messages);
    return () => {};
  },
  get value() { return chatStore.messages; },
};

/** Panel GIF visible — `$showGifs` dans le template */
export const showGifs = {
  subscribe(fn: (v: boolean) => void) {
    fn(chatStore.showGifs);
    return () => {};
  },
  get value() { return chatStore.showGifs; },
};

/** Résultats GIF — `$gifResults` dans le template */
export const gifResults = {
  subscribe(fn: (v: any[]) => void) {
    fn(chatStore.gifResults);
    return () => {};
  },
  get value() { return chatStore.gifResults; },
};

/** Loading GIF — `$gifLoading` dans le template */
export const gifLoading = {
  subscribe(fn: (v: boolean) => void) {
    fn(chatStore.gifLoading);
    return () => {};
  },
  get value() { return chatStore.gifLoading; },
};

// -----------------------------------------------------------------
// 4️⃣ Getters fonctionnels
// -----------------------------------------------------------------
export function getMessages(): Message[] { return chatStore.messages; }
export function getConnectionError(): string | null { return chatStore.connectionError; }
export function getGifResults(): any[] { return chatStore.gifResults; }
export function getShowGifs(): boolean { return chatStore.showGifs; }
export function getGifLoading(): boolean { return chatStore.gifLoading; }

// -----------------------------------------------------------------
// 5️⃣ Formatage
// -----------------------------------------------------------------
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();

  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }
  return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

// -----------------------------------------------------------------
// 6️⃣ Actions API
// -----------------------------------------------------------------
export async function loadMessages(conversationId: string): Promise<void> {
  try {
    const response = await fetch(
      `/api/conversations/${conversationId}/messages`,
      { credentials: 'include' }
    );
    if (!response.ok) throw new Error('Impossible de charger les messages');

    const data = await response.json();
    setMessages(data.messages ?? []);
    setConnectionError(null);
  } catch (err) {
    setConnectionError('Erreur de chargement des messages');
    console.error('Erreur chargement messages :', err);
  }
}

export async function sendMessage(
  content: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
): Promise<void> {
  try {
    const encrypted = await encryptForRecipients(content, recipientPublicKeys, senderPrivateKey);

    const messagePayload = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [
          userId,
          Array.from(key),
        ])
      ),
      nonce: Array.from(encrypted.nonce),
      media_type: null,
      media_url: null,
    };

    const response = await fetch('/api/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(messagePayload),
    });

    if (!response.ok) throw new Error("Erreur lors de l'envoi du message");

    await loadMessages(conversationId);
    setConnectionError(null);
  } catch (err) {
    setConnectionError("Erreur lors de l'envoi du message");
    console.error('Erreur envoi message :', err);
  }
}

export async function sendGif(
  gifUrl: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
): Promise<void> {
  try {
    const encrypted = await encryptForRecipients(
      `[GIF]${gifUrl}`,
      recipientPublicKeys,
      senderPrivateKey
    );

    const payload = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [
          userId,
          Array.from(key),
        ])
      ),
      nonce: Array.from(encrypted.nonce),
      media_type: 'gif',
      media_url: gifUrl,
    };

    const response = await fetch('/api/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(payload),
    });

    if (!response.ok) throw new Error("Erreur lors de l'envoi du GIF");

    await loadMessages(conversationId);
    setConnectionError(null);
  } catch (err) {
    setConnectionError("Erreur lors de l'envoi du GIF");
    console.error('Erreur envoi GIF :', err);
  }
}

export async function searchGifs(query: string): Promise<void> {
  try {
    setGifLoading(true);
    setGifResults([]);

    const response = await fetch(
      `https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(query)}&key=LIVDSRZULELA&client_key=nook&limit=12`
    );
    if (!response.ok) throw new Error('Erreur lors de la recherche de GIFs');

    const data = await response.json();
    setGifResults(data.results ?? []);
  } catch (err) {
    console.error('Erreur recherche GIFs :', err);
  } finally {
    setGifLoading(false);
  }
}

export async function addReaction(messageId: string, emoji: string): Promise<void> {
  try {
    const response = await fetch('/api/messages/reaction', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ message_id: messageId, emoji }),
    });
    if (!response.ok) throw new Error("Erreur lors de l'ajout de la réaction");

    chatStore.messages = chatStore.messages.map((msg) => {
      if (msg.id !== messageId) return msg;
      const newReactions = { ...(msg.reactions ?? {}) };
      newReactions[emoji] = (newReactions[emoji] ?? 0) + 1;
      return { ...msg, reactions: newReactions };
    });
  } catch (err) {
    console.error('Erreur ajout réaction :', err);
  }
}

export async function decryptMessageContent(
  message: Message,
  privateKey: Uint8Array,
  senderPublicKey: Uint8Array
): Promise<string> {
  try {
    if (message.media_type === 'gif') return `[GIF]${message.media_url}`;

    const encryptedContent = new Uint8Array(message.content as unknown as number[]);
    const userId = authStore.user?.id ?? '';
    const encryptedKeyData = new Uint8Array(
      (message.encrypted_keys[userId] as unknown as number[]) ?? []
    );
    const nonce = new Uint8Array(message.nonce as unknown as number[]);

    return await decryptMessage(encryptedContent, encryptedKeyData, senderPublicKey, privateKey, nonce);
  } catch (err) {
    console.error('Erreur déchiffrement message :', err);
    return '[Message illisible]';
  }
}

export async function initUserKeys(): Promise<
  { privateKey: Uint8Array; publicKey: Uint8Array } | null
> {
  const user = authStore.user;
  if (!user) return null;

  const stored = await getStoredKeys(user.id);
  if (!stored) return null;

  try {
    const password =
      (user as any).password ??
      (browser && prompt('Entrez votre mot de passe pour déchiffrer vos messages :'));

    if (!password) return null;

    const privateKey = await decryptPrivateKey(stored.encryptedPrivateKey, password);
    return { privateKey, publicKey: stored.publicKey };
  } catch (err) {
    setConnectionError('Erreur de déchiffrement des clés – vérifiez votre mot de passe');
    console.error('Erreur déchiffrement clés :', err);
    return null;
  }
}
