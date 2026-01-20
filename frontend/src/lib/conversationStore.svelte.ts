/**
 * Store et actions liées aux conversations (Svelte 5 avec runes).
 *
 * - Chargement des conversations, participants et utilisateurs disponibles.
 * - Création / mise à jour / suppression de conversations.
 * - Gestion des participants (ajout, départ, lecture des messages).
 * - Variables dérivées utiles pour l'UI (conversation active, tri, affichage du nom).
 * - Intégration d'un WebSocket simple pour recevoir les nouveaux messages.
 *
 * Toutes les fonctions sont typées, les erreurs sont capturées et le code
 * fonctionne correctement côté client (`browser`).  
 */

import { browser } from '$app/environment';
import { authUser, isAuthenticated } from './authStore.svelte.js';
import { connectionError } from './chatStore.svelte.ts';
import type { Conversation, Participant, Message } from './types';

// -----------------------------------------------------------------
// 1️⃣ États réactifs (Svelte 5)
// -----------------------------------------------------------------
export let conversations = $state<Conversation[]>([]);
export let activeConversationId = $state<string | null>(null);
export let participants = $state<Participant[]>([]);
export let availableUsers = $state<Participant[]>([]);

// -----------------------------------------------------------------
// 2️⃣ Chargement des données depuis le backend
// -----------------------------------------------------------------
export async function loadConversations(): Promise<void> {
  try {
    const resp = await fetch('/api/conversations', { credentials: 'include' });
    if (!resp.ok) throw new Error('Impossible de charger les conversations');

    const data = await resp.json();
    conversations = data.conversations ?? [];
    connectionError.set(null);

    // Sélectionner automatiquement la première conversation si aucune n'est active
    if (!activeConversationId && data.conversations?.length) {
      const firstId = data.conversations[0].id;
      activeConversationId = firstId;
      await loadParticipants(firstId);
    }
  } catch (err) {
    connectionError.set('Erreur de chargement des conversations');
    console.error('Erreur chargement conversations :', err);
  }
}

export async function loadParticipants(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(
      `/api/conversations/${conversationId}/participants`,
      { credentials: 'include' }
    );
    if (!resp.ok) throw new Error('Impossible de charger les participants');

    const data = await resp.json();
    participants = data.participants ?? [];
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des participants');
    console.error('Erreur chargement participants :', err);
  }
}

export async function loadAvailableUsers(): Promise<void> {
  try {
    const resp = await fetch('/api/users/available', {
      credentials: 'include',
    });
    if (!resp.ok) throw new Error('Impossible de charger les utilisateurs disponibles');

    const data = await resp.json();
    availableUsers = data.users ?? [];
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des utilisateurs');
    console.error('Erreur chargement utilisateurs :', err);
  }
}

// -----------------------------------------------------------------
// 3️⃣ Actions sur les conversations
// -----------------------------------------------------------------
export async function createConversation(
  name: string | null,
  participantIds: string[],
  isGroup: boolean
): Promise<Conversation> {
  try {
    const resp = await fetch('/api/conversations', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({
        name,
        participant_ids: participantIds,
        is_group: isGroup,
      }),
    });

    if (!resp.ok) {
      const errData = await resp.json();
      throw new Error(errData.error ?? 'Erreur lors de la création de la conversation');
    }

    const data = await resp.json();
    const newConv: Conversation = data.conversation;

    // Mettre à jour l'état local
    const exists = conversations.some((c) => c.id === newConv.id);
    if (exists) {
      conversations = conversations.map((c) => (c.id === newConv.id ? newConv : c));
    } else {
      conversations = [...conversations, newConv];
    }

    // Sélectionner la nouvelle conversation
    activeConversationId = newConv.id;
    await loadParticipants(newConv.id);
    connectionError.set(null);
    return newConv;
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Erreur inconnue';
    connectionError.set(msg);
    console.error('Erreur création conversation :', err);
    throw err;
  }
}

export async function addParticipantToConversation(
  conversationId: string,
  userId: string
): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/participants`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ user_id: userId }),
    });

    if (!resp.ok) throw new Error("Erreur lors de l'ajout du participant");

    // Recharger la liste des participants
    await loadParticipants(conversationId);
    connectionError.set(null);
  } catch (err) {
    connectionError.set("Erreur lors de l'ajout du participant");
    console.error("Erreur ajout participant :", err);
  }
}

export async function leaveConversation(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/leave`, {
      method: 'POST',
      credentials: 'include',
    });

    if (!resp.ok) throw new Error('Erreur lors du départ de la conversation');

    // Retirer la conversation de l'état
    conversations = conversations.filter((c) => c.id !== conversationId);

    // Si c'était la conversation active, choisir une autre
    if (activeConversationId === conversationId) {
      const newActive = conversations.length ? conversations[0].id : null;
      activeConversationId = newActive;
      if (newActive) await loadParticipants(newActive);
    }

    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors du départ de la conversation');
    console.error('Erreur départ conversation :', err);
  }
}

export async function deleteConversation(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}`, {
      method: 'DELETE',
      credentials: 'include',
    });

    if (!resp.ok) throw new Error('Erreur lors de la suppression de la conversation');

    // Retirer de l'état
    conversations = conversations.filter((c) => c.id !== conversationId);

    // Gestion de l'active
    if (activeConversationId === conversationId) {
      const newActive = conversations.length ? conversations[0].id : null;
      activeConversationId = newActive;
      if (newActive) await loadParticipants(newActive);
    }

    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de la suppression de la conversation');
    console.error('Erreur suppression conversation :', err);
  }
}

/**
 * Marque les messages d'une conversation comme lus côté serveur
 * et met à jour le compteur `unread_count` localement.
 */
export async function markMessagesAsRead(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/read`, {
      method: 'POST',
      credentials: 'include',
    });

    if (!resp.ok) throw new Error('Erreur lors du marquage comme lu');

    // Mettre à jour le compteur localement
    conversations = conversations.map((c) => 
      c.id === conversationId ? { ...c, unread_count: 0 } : c
    );
  } catch (err) {
    console.error('Erreur marquage messages lus :', err);
  }
}

// -----------------------------------------------------------------
// 4️⃣ Variables dérivées (pratiques pour l'UI - Svelte 5)
// -----------------------------------------------------------------
export const activeConversation = $derived(() => 
  conversations.find((c) => c.id === activeConversationId) ?? null
);

export const sortedConversations = $derived(() => {
  return [...conversations].sort((a, b) => {
    // Priorité aux conversations avec des messages non lus
    if (a.unread_count > 0 && b.unread_count === 0) return -1;
    if (a.unread_count === 0 && b.unread_count > 0) return 1;
    // Sinon, trier par date du dernier message (descendant)
    return new Date(b.last_message_at).getTime() - new Date(a.last_message_at).getTime();
  });
});

export const conversationDisplayName = $derived(() => {
  if (!activeConversation) return 'Nouvelle conversation';

  if (!activeConversation.is_group) {
    // 1‑to‑1 : afficher le nom de l'autre participant
    const other = participants.find((p) => p.id !== authUser?.id);
    return other?.name ?? 'Utilisateur inconnu';
  }

  // Groupe : afficher le nom ou un libellé par défaut
  return activeConversation.name ?? 'Groupe sans nom';
});

// -----------------------------------------------------------------
// 5️⃣ Utilitaires
// -----------------------------------------------------------------
/**
 * Génère un identifiant de conversation unique basé sur les participants.
 * L'ordre des IDs n'influence pas le résultat (tri préalable).
 */
export function generateConversationId(participantIds: string[]): string {
  const sorted = [...participantIds].sort();
  return `conv_${sorted.join('_')}_${Date.now()}`;
}

// -----------------------------------------------------------------
// 6️⃣ Initialisation & réaction à l'authentification (Svelte 5)
// -----------------------------------------------------------------
async function initConversations(): Promise<void> {
  if (authUser) {
    await loadConversations();
    await loadAvailableUsers();
  }
}

// Réagir aux changements d'état d'authentification
// En Svelte 5, nous pouvons utiliser un effet pour cela
// Mais dans un module, nous ne pouvons pas utiliser $effect directement
// Nous allons donc exposer une fonction d'initialisation que les composants appelleront
let initialized = false;

/**
 * Initialise le store de conversations. Doit être appelé dans onMount d'un composant.
 */
export async function initConversationStore(): Promise<void> {
  if (initialized || !browser) return;
  initialized = true;

  // Initialisation au démarrage
  if (isAuthenticated) {
    await initConversations();
  }

  // Surveiller les changements d'authentification
  // En Svelte 5, nous pourrions utiliser un effet, mais ici nous utilisons une approche plus simple
  // Les composants peuvent réagir à `isAuthenticated` directement
}

// -----------------------------------------------------------------
// 7️⃣ WebSocket pour les nouveaux messages (client‑side only)
// -----------------------------------------------------------------
let ws: WebSocket | null = null;

/**
 * Initialise le WebSocket qui écoute les nouveaux messages.
 * Retourne une fonction de nettoyage à appeler lors du `onDestroy`.
 */
export function setupMessageWebSocket(): () => void {
  if (!browser) return () => {};

  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const url = `${protocol}://${window.location.host}/ws/messages`;

  ws = new WebSocket(url);

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.type !== 'new_message') return;

      const msg: Message = data.message;

      // Mettre à jour la conversation concernée
      conversations = conversations.map((conv) => {
        if (conv.id !== msg.conversation_id) return conv;

        const isActive = activeConversationId === conv.id;
        return {
          ...conv,
          last_message_at: msg.timestamp,
          last_message_preview: '[Nouveau message]',
          unread_count: isActive ? 0 : conv.unread_count + 1,
        };
      });

      // Si c'est la conversation active, rafraîchir les messages
      if (msg.conversation_id === activeConversationId) {
        // La fonction `loadMessages` se trouve dans `chatStore.ts`
        // Nous l'importons dynamiquement pour éviter les cycles d'import.
        import('./chatStore').then(({ loadMessages }) => {
          loadMessages(msg.conversation_id).catch(console.error);
        });
      }
    } catch (e) {
      console.error('Erreur parsing WS message :', e);
    }
  };

  ws.onerror = (e) => {
    console.error('WebSocket error :', e);
  };

  ws.onclose = () => {
    ws = null;
    console.log('WebSocket closed');
  };

  // Fonction de nettoyage
  return () => {
    if (ws) ws.close();
    ws = null;
  };
}

// -----------------------------------------------------------------
// 8️⃣ Initialisation au chargement du module (client‑side only)
// -----------------------------------------------------------------
if (browser && isAuthenticated) {
  // Initialisation automatique si déjà authentifié
  initConversations().catch(console.error);
}
