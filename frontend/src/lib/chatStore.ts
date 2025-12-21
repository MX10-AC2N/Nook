// frontend/src/lib/chatStore.ts
import { writable, derived, get } from 'svelte/store';
import type { Writable } from 'svelte/store';
import { authStore } from './authStore';
import { 
  encryptForRecipients, 
  decryptMessage, 
  getStoredKeys,
  decryptPrivateKey
} from './crypto';
import type { Message, Reaction, DecryptedMessage, EncryptedMessage } from '$lib/types';

// Store pour l'utilisateur courant
export const currentUser = writable<{
  id: string;
  name: string;
  username: string;
  publicKey?: Uint8Array;
} | null>(null);

// Store pour les messages
export const messages: Writable<Message[]> = writable([]);

// Store pour les erreurs de connexion
export const connectionError = writable<string | null>(null);

// Store pour la recherche de GIFs
export const gifResults = writable<any[]>([]);
export const showGifs = writable(false);
export const gifLoading = writable(false);

// Charger les messages depuis le backend
export async function loadMessages(conversationId: string) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}/messages`, {
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Impossible de charger les messages');
    }
    
    const data = await response.json();
    messages.set(data.messages || []);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des messages');
    console.error('Erreur chargement messages:', err);
  }
}

// Envoyer un message chiffré
export async function sendMessage(
  content: string, 
  conversationId: string, 
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
  try {
    // Chiffrer le message pour tous les destinataires
    const encrypted = await encryptForRecipients(
      content, 
      recipientPublicKeys, 
      senderPrivateKey
    );
    
    // Préparer les données pour l'API
    const messageData = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [
          userId, 
          Array.from(key)
        ])
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
    
    if (!response.ok) {
      throw new Error('Erreur lors de l\'envoi du message');
    }
    
    // Recharger les messages après envoi
    await loadMessages(conversationId);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de l\'envoi du message');
    console.error('Erreur envoi message:', err);
  }
}

// Envoyer un GIF chiffré
export async function sendGif(
  gifUrl: string,
  conversationId: string,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
) {
  try {
    // Chiffrer l'URL du GIF
    const encrypted = await encryptForRecipients(
      `[GIF]${gifUrl}`, 
      recipientPublicKeys, 
      senderPrivateKey
    );
    
    const messageData = {
      conversation_id: conversationId,
      content: Array.from(encrypted.encryptedContent),
      encrypted_keys: Object.fromEntries(
        Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [
          userId, 
          Array.from(key)
        ])
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
    
    if (!response.ok) {
      throw new Error('Erreur lors de l\'envoi du GIF');
    }
    
    await loadMessages(conversationId);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de l\'envoi du GIF');
    console.error('Erreur envoi GIF:', err);
  }
}

// Rechercher des GIFs sur Tenor (anonyme)
export async function searchGifs(query: string) {
  try {
    gifLoading.set(true);
    gifResults.set([]);
    
    // Appel anonyme à Tenor (pas de clé API = limité mais fonctionnel)
    const response = await fetch(`https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(query)}&key=LIVDSRZULELA&client_key=nook&limit=12`);
    
    if (!response.ok) {
      throw new Error('Erreur lors de la recherche de GIFs');
    }
    
    const data = await response.json();
    gifResults.set(data.results || []);
    gifLoading.set(false);
  } catch (err) {
    gifLoading.set(false);
    console.error('Erreur recherche GIFs:', err);
  }
}

// Ajouter une réaction à un message
export async function addReaction(messageId: string, emoji: string) {
  try {
    const response = await fetch('/api/messages/reaction', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ 
        message_id: messageId, 
        emoji 
      })
    });
    
    if (!response.ok) {
      throw new Error('Erreur lors de l\'ajout de la réaction');
    }
    
    // Mettre à jour localement les messages
    messages.update(msgs => {
      return msgs.map(msg => {
        if (msg.id === messageId) {
          const reactions = msg.reactions || {};
          reactions[emoji] = (reactions[emoji] || 0) + 1;
          return { ...msg, reactions };
        }
        return msg;
      });
    });
  } catch (err) {
    console.error('Erreur ajout réaction:', err);
  }
}

// Déchiffrer un message pour l'affichage
export async function decryptMessageContent(
  message: Message, 
  privateKey: Uint8Array,
  senderPublicKey: Uint8Array
): Promise<string> {
  try {
    // Vérifier si c'est un GIF
    if (message.media_type === 'gif') {
      return `[GIF]${message.media_url}`;
    }
    
    // Extraire les données chiffrées
    const encryptedContent = new Uint8Array(message.content);
    const encryptedKeyData = new Uint8Array(
      message.encrypted_keys[get(authStore).user?.id || '']
    );
    const nonce = new Uint8Array(message.nonce);
    
    // Déchiffrer le message
    return await decryptMessage(
      encryptedContent,
      encryptedKeyData,
      senderPublicKey,
      privateKey,
      nonce
    );
  } catch (err) {
    console.error('Erreur déchiffrement message:', err);
    return '[Message illisible]';
  }
}

// Toggle la recherche de GIFs
export function toggleGifs() {
  showGifs.update(value => !value);
  if (!get(showGifs)) {
    gifResults.set([]);
  }
}

// Initialiser les clés de l'utilisateur
export async function initUserKeys() {
  const user = get(authStore).user;
  if (!user) return null;
  
  const storedKeys = await getStoredKeys(user.id);
  if (!storedKeys) return null;
  
  try {
    // Dans une vraie implémentation, demander le mot de passe ou le récupérer sécurisé
    const password = user.password || prompt('Entrez votre mot de passe pour déchiffrer vos messages:');
    if (!password) return null;
    
    const privateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
    return {
      privateKey,
      publicKey: storedKeys.publicKey
    };
  } catch (err) {
    connectionError.set('Erreur de déchiffrement des clés - vérifiez votre mot de passe');
    console.error('Erreur déchiffrement clés:', err);
    return null;
  }
}

// Store dérivé pour les messages déchiffrés
export const decryptedMessages = derived(
  [messages, authStore],
  async ([$messages, $authStore]) => {
    if (!$authStore.user || $messages.length === 0) return [];
    
    const keys = await initUserKeys();
    if (!keys) return [];
    
    const decryptedMsgs = await Promise.all(
      $messages.map(async (msg) => {
        try {
          const sender = msg.sender_id === $authStore.user?.id 
            ? $authStore.user 
            : { id: msg.sender_id, name: msg.sender_name };
          
          // Trouver la clé publique de l'expéditeur
          let senderPublicKey = keys.publicKey;
          if (msg.sender_id !== $authStore.user?.id) {
            // Dans une vraie implémentation, récupérer la clé publique du destinataire depuis le backend
            senderPublicKey = keys.publicKey; // Placeholder - à remplacer par la vraie logique
          }
          
          const decryptedContent = await decryptMessageContent(
            msg,
            keys.privateKey,
            senderPublicKey
          );
          
          return {
            ...msg,
            decryptedContent,
            senderName: sender.name,
            isCurrentUser: msg.sender_id === $authStore.user?.id
          };
        } catch (err) {
          console.error('Erreur déchiffrement message individuel:', err);
          return {
            ...msg,
            decryptedContent: '[Message illisible]',
            senderName: msg.sender_name,
            isCurrentUser: msg.sender_id === $authStore.user?.id
          };
        }
      })
    );
    
    return decryptedMsgs.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
  }
);

// Fonction utilitaire pour formater les timestamps
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  
  // Si aujourd'hui
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  
  // Si cette année
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
  
  // Année différente
  return date.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
}