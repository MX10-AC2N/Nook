/**
 * Store et actions liées aux conversations — Svelte 5 runes.
 *
 * Pattern : objet $state unique (jamais de let $state exporté réassignable).
 * Les exports nommés (activeConversationId, participants) sont des fonctions
 * getters pour assurer la compatibilité avec le code existant.
 */

import { browser } from '$app/environment';
import { authStore } from './authStore.svelte.js';
import { setConnectionError } from './chatStore.svelte.js';
import type { Conversation, Participant, Message } from './types';

// -----------------------------------------------------------------
// 1️⃣ Interface et état réactif principal
// -----------------------------------------------------------------
export interface ConversationState {
  conversations: Conversation[];
  activeConversationId: string | null;
  participants: Participant[];
  availableUsers: Participant[];
}

function createInitialState(): ConversationState {
  return {
    conversations: [],
    activeConversationId: null,
    participants: [],
    availableUsers: [],
  };
}

/**
 * État principal — objet $state muté directement, jamais réassigné.
 * C'est la seule façon valide d'exporter du state réactif en Svelte 5.
 */
export const conversationStore = $state<ConversationState>(createInitialState());

// -----------------------------------------------------------------
// 2️⃣ Accesseurs nommés (compatibilité avec imports existants)
// Les composants qui font `import { activeConversationId } from '...'`
// peuvent appeler activeConversationId() pour lire la valeur réactive.
// -----------------------------------------------------------------

/** Retourne l'ID de la conversation active. */
export function getActiveConversationId(): string | null {
  return conversationStore.activeConversationId;
}

/** Retourne les conversations chargées. */
export function getConversations(): Conversation[] {
  return conversationStore.conversations;
}

/** Retourne les participants de la conversation active. */
export function getParticipants(): Participant[] {
  return conversationStore.participants;
}

/** Retourne les utilisateurs disponibles pour créer une conversation. */
export function getAvailableUsers(): Participant[] {
  return conversationStore.availableUsers;
}

/**
 * Accesseurs compatibles avec l'ancien pattern `get(activeConversationId)`.
 * MediaRecorder.svelte fait `get(activeConversationId)` — on expose un objet
 * qui imite un readable Svelte store (propriété .value accessible).
 */
export const activeConversationId = {
  get value() { return conversationStore.activeConversationId; },
  // Imitation subscribe pour compatibilité get() de svelte/store
  subscribe(fn: (v: string | null) => void) {
    fn(conversationStore.activeConversationId);
    return () => {};
  },
};

export const participants = {
  get value() { return conversationStore.participants; },
  subscribe(fn: (v: Participant[]) => void) {
    fn(conversationStore.participants);
    return () => {};
  },
};

export const conversations = {
  get value() { return conversationStore.conversations; },
  subscribe(fn: (v: Conversation[]) => void) {
    fn(conversationStore.conversations);
    return () => {};
  },
};

// -----------------------------------------------------------------
// 3️⃣ Chargement des données depuis le backend
// -----------------------------------------------------------------
export async function loadConversations(): Promise<void> {
  try {
    const resp = await fetch('/api/conversations', { credentials: 'include' });
    if (!resp.ok) throw new Error('Impossible de charger les conversations');

    const data = await resp.json();
    conversationStore.conversations = data.conversations ?? [];
    setConnectionError(null);

    if (!conversationStore.activeConversationId && data.conversations?.length) {
      const firstId = data.conversations[0].id;
      conversationStore.activeConversationId = firstId;
      await loadParticipants(firstId);
    }
  } catch (err) {
    setConnectionError('Erreur de chargement des conversations');
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
    conversationStore.participants = data.participants ?? [];
    setConnectionError(null);
  } catch (err) {
    setConnectionError('Erreur de chargement des participants');
    console.error('Erreur chargement participants :', err);
  }
}

export async function loadAvailableUsers(): Promise<void> {
  try {
    const resp = await fetch('/api/users/available', { credentials: 'include' });
    if (!resp.ok) throw new Error('Impossible de charger les utilisateurs disponibles');

    const data = await resp.json();
    conversationStore.availableUsers = data.users ?? [];
    setConnectionError(null);
  } catch (err) {
    setConnectionError('Erreur de chargement des utilisateurs');
    console.error('Erreur chargement utilisateurs :', err);
  }
}

// -----------------------------------------------------------------
// 4️⃣ Actions sur les conversations
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
      body: JSON.stringify({ name, participant_ids: participantIds, is_group: isGroup }),
    });

    if (!resp.ok) {
      const errData = await resp.json();
      throw new Error(errData.error ?? 'Erreur lors de la création de la conversation');
    }

    const data = await resp.json();
    const newConv: Conversation = data.conversation;

    const exists = conversationStore.conversations.some((c) => c.id === newConv.id);
    if (exists) {
      conversationStore.conversations = conversationStore.conversations.map((c) =>
        c.id === newConv.id ? newConv : c
      );
    } else {
      conversationStore.conversations = [...conversationStore.conversations, newConv];
    }

    conversationStore.activeConversationId = newConv.id;
    await loadParticipants(newConv.id);
    setConnectionError(null);
    return newConv;
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Erreur inconnue';
    setConnectionError(msg);
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

    await loadParticipants(conversationId);
    setConnectionError(null);
  } catch (err) {
    setConnectionError("Erreur lors de l'ajout du participant");
    console.error('Erreur ajout participant :', err);
  }
}

export async function leaveConversation(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/leave`, {
      method: 'POST',
      credentials: 'include',
    });
    if (!resp.ok) throw new Error('Erreur lors du départ de la conversation');

    conversationStore.conversations = conversationStore.conversations.filter(
      (c) => c.id !== conversationId
    );

    if (conversationStore.activeConversationId === conversationId) {
      const newActive = conversationStore.conversations.length
        ? conversationStore.conversations[0].id
        : null;
      conversationStore.activeConversationId = newActive;
      if (newActive) await loadParticipants(newActive);
    }
    setConnectionError(null);
  } catch (err) {
    setConnectionError('Erreur lors du départ de la conversation');
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

    conversationStore.conversations = conversationStore.conversations.filter(
      (c) => c.id !== conversationId
    );

    if (conversationStore.activeConversationId === conversationId) {
      const newActive = conversationStore.conversations.length
        ? conversationStore.conversations[0].id
        : null;
      conversationStore.activeConversationId = newActive;
      if (newActive) await loadParticipants(newActive);
    }
    setConnectionError(null);
  } catch (err) {
    setConnectionError('Erreur lors de la suppression de la conversation');
    console.error('Erreur suppression conversation :', err);
  }
}

export async function markMessagesAsRead(conversationId: string): Promise<void> {
  try {
    const resp = await fetch(`/api/conversations/${conversationId}/read`, {
      method: 'POST',
      credentials: 'include',
    });
    if (!resp.ok) throw new Error('Erreur lors du marquage comme lu');

    conversationStore.conversations = conversationStore.conversations.map((c) =>
      c.id === conversationId ? { ...c, unread_count: 0 } : c
    );
  } catch (err) {
    console.error('Erreur marquage messages lus :', err);
  }
}

// -----------------------------------------------------------------
// 5️⃣ Getters dérivés (fonctions pures — pas de $derived en module)
// -----------------------------------------------------------------
export function getActiveConversation(): Conversation | null {
  return (
    conversationStore.conversations.find(
      (c) => c.id === conversationStore.activeConversationId
    ) ?? null
  );
}

export function getSortedConversations(): Conversation[] {
  return [...conversationStore.conversations].sort((a, b) => {
    if (a.unread_count > 0 && b.unread_count === 0) return -1;
    if (a.unread_count === 0 && b.unread_count > 0) return 1;
    return new Date(b.last_message_at).getTime() - new Date(a.last_message_at).getTime();
  });
}

export function getConversationDisplayName(): string {
  const activeConv = getActiveConversation();
  if (!activeConv) return 'Nouvelle conversation';

  if (!activeConv.is_group) {
    const other = conversationStore.participants.find(
      (p) => p.id !== authStore.user?.id
    );
    return other?.name ?? 'Utilisateur inconnu';
  }
  return activeConv.name ?? 'Groupe sans nom';
}

// -----------------------------------------------------------------
// 6️⃣ Utilitaires
// -----------------------------------------------------------------
export function generateConversationId(participantIds: string[]): string {
  const sorted = [...participantIds].sort();
  return `conv_${sorted.join('_')}_${Date.now()}`;
}

export function setActiveConversation(id: string | null): void {
  conversationStore.activeConversationId = id;
}

export function resetConversationStore(): void {
  Object.assign(conversationStore, createInitialState());
}

// -----------------------------------------------------------------
// 7️⃣ Initialisation
// -----------------------------------------------------------------
export async function initConversationStore(): Promise<void> {
  if (!browser || !authStore.isAuthenticated) return;
  await loadConversations();
  await loadAvailableUsers();
}

// -----------------------------------------------------------------
// 8️⃣ WebSocket pour les nouveaux messages
// -----------------------------------------------------------------
let ws: WebSocket | null = null;

export function setupMessageWebSocket(): () => void {
  if (!browser) return () => {};

  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const url = `${protocol}://${window.location.host}/webrtc/ws`;
  ws = new WebSocket(url);

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.type !== 'new_message') return;

      const msg: Message = data.message;

      conversationStore.conversations = conversationStore.conversations.map((conv) => {
        if (conv.id !== msg.conversation_id) return conv;
        const isActive = conversationStore.activeConversationId === conv.id;
        return {
          ...conv,
          last_message_at: msg.timestamp,
          last_message_preview: '[Nouveau message]',
          unread_count: isActive ? 0 : conv.unread_count + 1,
        };
      });

      if (msg.conversation_id === conversationStore.activeConversationId) {
        import('./chatStore.svelte.js').then(({ loadMessages }) => {
          loadMessages(msg.conversation_id).catch(console.error);
        });
      }
    } catch (e) {
      console.error('Erreur parsing WS message :', e);
    }
  };

  ws.onerror = (e) => console.error('WebSocket error :', e);
  ws.onclose = () => { ws = null; };

  return () => { if (ws) ws.close(); ws = null; };
}

// Auto-init si déjà authentifié au chargement du module
if (browser && authStore.isAuthenticated) {
  initConversationStore().catch(console.error);
}