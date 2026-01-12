/**
 * Store et actions liées aux conversations.
 *
 * - Chargement des conversations, participants et utilisateurs disponibles.
 * - Création / mise à jour / suppression de conversations.
 * - Gestion des participants (ajout, départ, lecture des messages).
 * - Stores dérivés utiles pour l’UI (conversation active, tri, affichage du nom).
 * - Intégration d’un WebSocket simple pour recevoir les nouveaux messages.
 *
 * Toutes les fonctions sont typées, les erreurs sont capturées et le code
 * fonctionne correctement côté client (`browser`).  
 */

import { writable, derived, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './authStore';
import type { Conversation, Participant, Message } from './types';
import { connectionError } from './chatStore';

// -----------------------------------------------------------------
// 1️⃣ Stores principaux
// -----------------------------------------------------------------
export const conversations: Writable<Conversation[]> = writable([]);
export const activeConversationId: Writable<string | null> = writable(null);
export const participants: Writable<Participant[]> = writable([]);
export const availableUsers: Writable<Participant[]> = writable([]);

// -----------------------------------------------------------------
// 2️⃣ Chargement des données depuis le backend
// -----------------------------------------------------------------
export async function loadConversations(): Promise<void> {
  try {
    const resp = await fetch('/api/conversations', { credentials: 'include' });
    if (!resp.ok) throw new Error('Impossible de charger les conversations');

    const data = await resp.json();
    conversations.set(data.conversations ?? []);
    connectionError.set(null);

    // Sélectionner automatiquement la première conversation si aucune n’est active
    if (!get(activeConversationId) && data.conversations?.length) {
      const firstId = data.conversations[0].id;
      activeConversationId.set(firstId);
      await loadParticipants(firstId);
    }
  } catch (err) {
    connectionError.set('Erreur de chargement des conversations');
    console.error('Erreur chargement conversations :', err);
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
    participants.set(data.participants ?? []);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des participants');
    console.error('Erreur chargement participants :', err);
  }
}

export async function loadAvailableUsers(): Promise<void> {
  try {
    const resp = await fetch('/api/users/available', {
      credentials: 'include',
    });
    if (!resp.ok) throw new Error('Impossible de charger les utilisateurs disponibles');

    const data = await resp.json();
    availableUsers.set(data.users ?? []);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des utilisateurs');
    console.error('Erreur chargement utilisateurs :', err);
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

    // Mettre à jour le store local
    conversations.update((list) => {
      const exists = list.some((c) => c.id === newConv.id);
      return exists ? list.map((c) => (c.id === newConv.id ? newConv : c)) : [...list, newConv];
    });

    // Sélectionner la nouvelle conversation
    activeConversationId.set(newConv.id);
    await loadParticipants(newConv.id);
    connectionError.set(null);
    return newConv;
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Erreur inconnue';
    connectionError.set(msg);
    console.error('Erreur création conversation :', err);
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
    console.error('Erreur ajout participant :', err);
  }
}

export async function leaveConversation(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/leave`, {
      method: 'POST',
      credentials: 'include',
    });

    if (!resp.ok) throw new Error('Erreur lors du départ de la conversation');

    // Retirer la conversation du store
    conversations.update((list) => list.filter((c) => c.id !== conversationId));

    // Si c’était la conversation active, choisir une autre
    if (get(activeConversationId) === conversationId) {
      const remaining = get(conversations);
      const newActive = remaining.length ? remaining[0].id : null;
      activeConversationId.set(newActive);
      if (newActive) await loadParticipants(newActive);
    }

    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors du départ de la conversation');
    console.error('Erreur départ conversation :', err);
  }
}

export async function deleteConversation(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}`, {
      method: 'DELETE',
      credentials: 'include',
    });

    if (!resp.ok) throw new Error('Erreur lors de la suppression de la conversation');

    // Retirer du store
    conversations.update((list) => list.filter((c) => c.id !== conversationId));

    // Gestion de l’active
    if (get(activeConversationId) === conversationId) {
      const remaining = get(conversations);
      const newActive = remaining.length ? remaining[0].id : null;
      activeConversationId.set(newActive);
      if (newActive) await loadParticipants(newActive);
    }

    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de la suppression de la conversation');
    console.error('Erreur suppression conversation :', err);
  }
}

/**
 * Marque les messages d’une conversation comme lus côté serveur
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
    conversations.update((list) =>
      list.map((c) => (c.id === conversationId ? { ...c, unread_count: 0 } : c))
    );
  } catch (err) {
    console.error('Erreur marquage messages lus :', err);
  }
}

// -----------------------------------------------------------------
// 4️⃣ Stores dérivés (pratiques pour l’UI)
// -----------------------------------------------------------------
export const activeConversation = derived(
  [conversations, activeConversationId],
  ([$convs, $activeId]) => $convs.find((c) => c.id === $activeId) ?? null
);

export const sortedConversations = derived(conversations, ($list) => {
  return [...$list].sort((a, b) => {
    // Priorité aux conversations avec des messages non lus
    if (a.unread_count > 0 && b.unread_count === 0) return -1;
    if (a.unread_count === 0 && b.unread_count > 0) return 1;
    // Sinon, trier par date du dernier message (descendant)
    return new Date(b.last_message_at).getTime() - new Date(a.last_message_at).getTime();
  });
});

export const conversationDisplayName = derived(
  [activeConversation, participants, authStore],
  ([$conv, $parts, $auth]) => {
    if (!$conv) return 'Nouvelle conversation';

    if (!$conv.is_group) {
      // 1‑to‑1 : afficher le nom de l’autre participant
      const other = $parts.find((p) => p.id !== $auth.user?.id);
      return other?.name ?? 'Utilisateur inconnu';
    }

    // Groupe : afficher le nom ou un libellé par défaut
    return $conv.name ?? 'Groupe sans nom';
  }
);

// -----------------------------------------------------------------
// 5️⃣ Utilitaires
// -----------------------------------------------------------------
/**
 * Génère un identifiant de conversation unique basé sur les participants.
 * L’ordre des IDs n’influence pas le résultat (tri préalable).
 */
export function generateConversationId(participantIds: string[]): string {
  const sorted = [...participantIds].sort();
  return `conv_${sorted.join('_')}_${Date.now()}`;
}

// -----------------------------------------------------------------
// 6️⃣ Initialisation & écoute d’authentification
// -----------------------------------------------------------------
async function initConversations(): Promise<void> {
  const user = get(authStore).user;
  if (user) {
    await loadConversations();
    await loadAvailableUsers();
  }
}

// Réagir aux changements d’état d’authentification
authStore.subscribe(($store) => {
  if ($store.isAuthenticated && !$store.loading) {
    initConversations().catch(console.error);
  }
});

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
      conversations.update((list) =>
        list.map((conv) => {
          if (conv.id !== msg.conversation_id) return conv;

          const isActive = get(activeConversationId) === conv.id;
          return {
            ...conv,
            last_message_at: msg.timestamp,
            last_message_preview: '[Nouveau message]',
            unread_count: isActive ? 0 : conv.unread_count + 1,
          };
        })
      );

      // Si c’est la conversation active, rafraîchir les messages
      if (msg.conversation_id === get(activeConversationId)) {
        // La fonction `loadMessages` se trouve dans `chatStore.ts`
        // Nous l’importons dynamiquement pour éviter les cycles d’import.
        import('./chatStore').then(({ loadMessages }) => {
          loadMessages(msg.conversation_id).catch(console.error);
        });
      }
    } catch (e) {
      console.error('Erreur parsing WS message :', e);
    }
  };

  ws.onerror = (e) => {
    console.error('WebSocket error :', e);
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