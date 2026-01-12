/**
 * Store et actions du chat.
 *
 * - Gestion du tableau de messages (chargement, ajout, réactions).
 * - Gestion de l’interface GIF (recherche, affichage, toggle).
 * - Chiffrement / déchiffrement des messages via les fonctions du module `crypto.ts`.
 * - Gestion des erreurs de connexion.
 *
 * Toutes les fonctions sont typées, les erreurs sont capturées et le code
 * fonctionne uniquement côté client (`browser`).  
 */

import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './authStore';
import {
  encryptForRecipients,
  decryptMessage,
  getStoredKeys,
  decryptPrivateKey,
} from './crypto';
import type { Message, EncryptedData, KeyPair } from './types';

// -----------------------------------------------------------------
// 1️⃣ Types & état initial du store
// -----------------------------------------------------------------
export interface ChatState {
  messages: Message[];
  connectionError: string | null;
  gifResults: any[];
  showGifs: boolean;
  gifLoading: boolean;
}

/** Valeur de départ du store. */
function createInitialState(): ChatState {
  return {
    messages: [],
    connectionError: null,
    gifResults: [],
    showGifs: false,
    gifLoading: false,
  };
}

/** Store principal contenant tout l’état du chat. */
function createChatStore() {
  const { subscribe, set, update } = writable<ChatState>(createInitialState());

  return {
    subscribe,

    /** Remplace complètement la liste de messages. */
    setMessages: (messages: Message[]) =>
      update((state) => ({ ...state, messages })),

    /** Ajoute un message à la fin du tableau. */
    addMessage: (message: Message) =>
      update((state) => ({
        ...state,
        messages: [...state.messages, message],
      })),

    /** Met à jour le champ d’erreur de connexion. */
    setConnectionError: (error: string | null) =>
      update((state) => ({ ...state, connectionError: error })),

    /** Remplace les résultats de recherche GIF. */
    setGifResults: (results: any[]) =>
      update((state) => ({ ...state, gifResults: results })),

    /** Active / désactive l’affichage du panel GIF. */
    toggleGifs: () =>
      update((state) => ({
        ...state,
        showGifs: !state.showGifs,
        // on vide les résultats quand on ferme le panel
        gifResults: state.showGifs ? [] : state.gifResults,
      })),

    /** Met à jour le flag de chargement des GIF. */
    setGifLoading: (loading: boolean) =>
      update((state) => ({ ...state, gifLoading: loading })),

    /** Réinitialise le store (ex. déconnexion). */
    reset: () => set(createInitialState()),
  };
}

export const chatStore = createChatStore();

// -----------------------------------------------------------------
// 2️⃣ Stores dérivés (accès facile depuis les composants)
// -----------------------------------------------------------------
export const messages = derived(chatStore, ($s) => $s.messages);
export const connectionError = derived(chatStore, ($s) => $s.connectionError);
export const gifResults = derived(chatStore, ($s) => $s.gifResults);
export const showGifs = derived(chatStore, ($s) => $s.showGifs);
export const gifLoading = derived(chatStore, ($s) => $s.gifLoading);

// -----------------------------------------------------------------
// 3️⃣ Helpers utilitaires
// -----------------------------------------------------------------
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();

  // Aujourd’hui → on n’affiche que l’heure
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString(undefined, {
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  // Même année → jour + mois
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
    });
  }

  // Autre année → jour + mois + année
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

// -----------------------------------------------------------------
// 4️⃣ Actions du chat (API)
// -----------------------------------------------------------------
/**
 * Charge les messages d’une conversation depuis le backend.
 */
export async function loadMessages(conversationId: string) {
  try {
    const response = await fetch(
      `/api/conversations/${conversationId}/messages`,
      { credentials: 'include' }
    );

    if (!response.ok) throw new Error('Impossible de charger les messages');

    const data = await response.json();
    chatStore.setMessages(data.messages ?? []);
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError('Erreur de chargement des messages');
    console.error('Erreur chargement messages :', err);
  }
}

/**
 * Envoie un message texte chiffré à tous les destinataires.
 *
 * @param content               Texte du message.
 * @param conversationId        ID de la conversation.
 * @param recipientPublicKeys   Tableau des clés publiques (Uint8Array) des participants.
 * @param senderPrivateKey      Clé privée (Uint8Array) de l’expéditeur.
 */
export async function sendMessage(
  content: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
  try {
    const encrypted = await encryptForRecipients(
      content,
      recipientPublicKeys,
      senderPrivateKey
    );

    // Le backend attend des tableaux d’octets (pas de Uint8Array directement)
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

    // Rafraîchir la liste après envoi
    await loadMessages(conversationId);
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError("Erreur lors de l'envoi du message");
    console.error('Erreur envoi message :', err);
  }
}

/**
 * Envoie un GIF (URL) comme message chiffré.
 *
 * @param gifUrl                URL du GIF sélectionné.
 * @param conversationId        ID de la conversation.
 * @param recipientPublicKeys   Clés publiques des destinataires.
 * @param senderPrivateKey      Clé privée de l’expéditeur.
 */
export async function sendGif(
  gifUrl: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
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
    chatStore.setConnectionError(null);
  } catch (err) {
    chatStore.setConnectionError("Erreur lors de l'envoi du GIF");
    console.error('Erreur envoi GIF :', err);
  }
}

/**
 * Recherche des GIFs via l’API Tenor.
 *
 * @param query   Chaîne de recherche.
 */
export async function searchGifs(query: string) {
  try {
    chatStore.setGifLoading(true);
    chatStore.setGifResults([]);

    const response = await fetch(
      `https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(
        query
      )}&key=LIVDSRZULELA&client_key=nook&limit=12`
    );

    if (!response.ok) throw new Error('Erreur lors de la recherche de GIFs');

    const data = await response.json();
    chatStore.setGifResults(data.results ?? []);
  } catch (err) {
    console.error('Erreur recherche GIFs :', err);
  } finally {
    chatStore.setGifLoading(false);
  }
}

/**
 * Ajoute une réaction (emoji) à un message.
 *
 * @param messageId  ID du message ciblé.
 * @param emoji      Emoji à ajouter (ex. "👍").
 */
export async function addReaction(messageId: string, emoji: string) {
  try {
    const response = await fetch('/api/messages/reaction', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ message_id: messageId, emoji }),
    });

    if (!response.ok) throw new Error("Erreur lors de l'ajout de la réaction");

    // Met à jour le store localement (optimistic UI)
    const updated = get(messages).map((msg) => {
      if (msg.id !== messageId) return msg;

      const newReactions = { ...(msg.reactions ?? {}) };
      newReactions[emoji] = (newReactions[emoji] ?? 0) + 1;
      return { ...msg, reactions: newReactions };
    });

    chatStore.setMessages(updated);
  } catch (err) {
    console.error('Erreur ajout réaction :', err);
  }
}

/**
 * Déchiffre le contenu d’un message.
 *
 * @param message          Message brut reçu du serveur.
 * @param privateKey       Clé privée (Uint8Array) de l’utilisateur courant.
 * @param senderPublicKey  Clé publique (Uint8Array) de l’expéditeur.
 * @returns                Texte déchiffré (ou indication d’erreur).
 */
export async function decryptMessageContent(
  message: Message,
  privateKey: Uint8Array,
  senderPublicKey: Uint8Array
): Promise<string> {
  try {
    // Les GIFs sont stockés en clair sous forme de tag spécial
    if (message.media_type === 'gif') return `[GIF]${message.media_url}`;

    const encryptedContent = new Uint8Array(message.content);
    const userId = get(authStore).user?.id ?? '';
    const encryptedKeyData = new Uint8Array(
      message.encrypted_keys[userId] ?? []
    );
    const nonce = new Uint8Array(message.nonce);

    return await decryptMessage(
      encryptedContent,
      encryptedKeyData,
      senderPublicKey,
      privateKey,
      nonce
    );
  } catch (err) {
    console.error('Erreur déchiffrement message :', err);
    return '[Message illisible]';
  }
}

/**
 * Initialise les clés de l’utilisateur (déchiffrement de la clé privée stockée).
 *
 * @returns `{ privateKey, publicKey }` ou `null` si impossible.
 */
export async function initUserKeys(): Promise<
  { privateKey: Uint8Array; publicKey: Uint8Array } | null
> {
  const user = get(authStore).user;
  if (!user) return null;

  const stored = await getStoredKeys(user.id);
  if (!stored) return null;

  try {
    // Demander le mot de passe à l’utilisateur (si besoin)
    const password =
      user.password ??
      (browser && prompt('Entrez votre mot de passe pour déchiffrer vos messages :'));

    if (!password) return null;

    const privateKey = await decryptPrivateKey(
      stored.encryptedPrivateKey,
      password
    );

    return { privateKey, publicKey: stored.publicKey };
  } catch (err) {
    chatStore.setConnectionError(
      'Erreur de déchiffrement des clés – vérifiez votre mot de passe'
    );
    console.error('Erreur déchiffrement clés :', err);
    return null;
  }
}