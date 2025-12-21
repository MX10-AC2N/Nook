// frontend/src/lib/conversationStore.ts
import { writable, derived, get } from 'svelte/store';
import type { Writable } from 'svelte/store';
import { authStore } from './authStore';
import type { Conversation, Participant, Message } from '$lib/types';
import { connectionError } from './chatStore';

// Store pour les conversations
export const conversations: Writable<Conversation[]> = writable([]);

// Store pour la conversation active
export const activeConversationId = writable<string | null>(null);

// Store pour les participants d'une conversation
export const participants: Writable<Participant[]> = writable([]);

// Store pour les utilisateurs disponibles (pour créer des conversations)
export const availableUsers: Writable<Participant[]> = writable([]);

// Charger toutes les conversations de l'utilisateur
export async function loadConversations() {
  try {
    const response = await fetch('/api/conversations', {
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Impossible de charger les conversations');
    }
    
    const data = await response.json();
    conversations.set(data.conversations || []);
    connectionError.set(null);
    
    // Si aucune conversation active et il y en a au moins une, sélectionner la première
    if (!get(activeConversationId) && data.conversations.length > 0) {
      activeConversationId.set(data.conversations[0].id);
      await loadParticipants(data.conversations[0].id);
    }
  } catch (err) {
    connectionError.set('Erreur de chargement des conversations');
    console.error('Erreur chargement conversations:', err);
  }
}

// Charger les participants d'une conversation
export async function loadParticipants(conversationId: string) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}/participants`, {
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Impossible de charger les participants');
    }
    
    const data = await response.json();
    participants.set(data.participants || []);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des participants');
    console.error('Erreur chargement participants:', err);
  }
}

// Charger les utilisateurs disponibles pour créer une nouvelle conversation
export async function loadAvailableUsers() {
  try {
    const response = await fetch('/api/users/available', {
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Impossible de charger les utilisateurs disponibles');
    }
    
    const data = await response.json();
    availableUsers.set(data.users || []);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur de chargement des utilisateurs');
    console.error('Erreur chargement utilisateurs:', err);
  }
}

// Créer une nouvelle conversation
export async function createConversation(
  name: string | null, 
  participantIds: string[], 
  isGroup: boolean
) {
  try {
    const response = await fetch('/api/conversations', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ 
        name, 
        participant_ids: participantIds, 
        is_group: isGroup 
      })
    });
    
    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error || 'Erreur lors de la création de la conversation');
    }
    
    const data = await response.json();
    
    // Mettre à jour les conversations locales
    conversations.update(convs => {
      const exists = convs.some(c => c.id === data.conversation.id);
      if (exists) {
        return convs.map(c => c.id === data.conversation.id ? data.conversation : c);
      }
      return [...convs, data.conversation];
    });
    
    // Sélectionner la nouvelle conversation
    activeConversationId.set(data.conversation.id);
    await loadParticipants(data.conversation.id);
    
    connectionError.set(null);
    return data.conversation;
  } catch (err) {
    connectionError.set(err instanceof Error ? err.message : 'Erreur inconnue');
    console.error('Erreur création conversation:', err);
    throw err;
  }
}

// Ajouter un participant à une conversation de groupe
export async function addParticipantToConversation(
  conversationId: string, 
  userId: string
) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}/participants`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ user_id: userId })
    });
    
    if (!response.ok) {
      throw new Error('Erreur lors de l\'ajout du participant');
    }
    
    // Recharger les participants
    await loadParticipants(conversationId);
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de l\'ajout du participant');
    console.error('Erreur ajout participant:', err);
  }
}

// Quitter une conversation
export async function leaveConversation(conversationId: string) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}/leave`, {
      method: 'POST',
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Erreur lors du départ de la conversation');
    }
    
    // Mettre à jour les conversations locales
    conversations.update(convs => convs.filter(c => c.id !== conversationId));
    
    // Si c'était la conversation active, sélectionner une autre
    if (get(activeConversationId) === conversationId) {
      const remainingConvs = get(conversations);
      activeConversationId.set(remainingConvs.length > 0 ? remainingConvs[0].id : null);
      if (remainingConvs.length > 0) {
        await loadParticipants(remainingConvs[0].id);
      }
    }
    
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors du départ de la conversation');
    console.error('Erreur départ conversation:', err);
  }
}

// Supprimer une conversation (pour les groupes, seulement par l'admin)
export async function deleteConversation(conversationId: string) {
  try {
    const response = await fetch(`/api/conversations/${conversationId}`, {
      method: 'DELETE',
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Erreur lors de la suppression de la conversation');
    }
    
    // Mettre à jour les conversations locales
    conversations.update(convs => convs.filter(c => c.id !== conversationId));
    
    // Si c'était la conversation active, sélectionner une autre
    if (get(activeConversationId) === conversationId) {
      const remainingConvs = get(conversations);
      activeConversationId.set(remainingConvs.length > 0 ? remainingConvs[0].id : null);
      if (remainingConvs.length > 0) {
        await loadParticipants(remainingConvs[0].id);
      }
    }
    
    connectionError.set(null);
  } catch (err) {
    connectionError.set('Erreur lors de la suppression de la conversation');
    console.error('Erreur suppression conversation:', err);
  }
}

// Marquer les messages comme lus
export async function markMessagesAsRead(conversationId: string) {
  try {
    await fetch(`/api/conversations/${conversationId}/read`, {
      method: 'POST',
      credentials: 'include'
    });
    
    // Mettre à jour localement les conversations
    conversations.update(convs => 
      convs.map(conv => 
        conv.id === conversationId 
          ? { ...conv, unread_count: 0 } 
          : conv
      )
    );
  } catch (err) {
    console.error('Erreur marquage messages lus:', err);
  }
}

// Store dérivé pour la conversation active
export const activeConversation = derived(
  [conversations, activeConversationId],
  ([$conversations, $activeConversationId]) => {
    return $conversations.find(c => c.id === $activeConversationId) || null;
  }
);

// Store dérivé pour les conversations triées (non lus en premier, puis par date)
export const sortedConversations = derived(
  conversations,
  ($conversations) => {
    return [...$conversations].sort((a, b) => {
      // Messages non lus en premier
      if (a.unread_count > 0 && b.unread_count === 0) return -1;
      if (a.unread_count === 0 && b.unread_count > 0) return 1;
      
      // Puis par date de dernier message
      return new Date(b.last_message_at).getTime() - new Date(a.last_message_at).getTime();
    });
  }
);

// Store dérivé pour le nom affiché d'une conversation
export const conversationDisplayName = derived(
  [activeConversation, participants, authStore],
  ([$activeConversation, $participants, $authStore]) => {
    if (!$activeConversation) return 'Nouvelle conversation';
    
    if (!$activeConversation.is_group) {
      // Conversation 1:1 - trouver l'autre participant
      const otherParticipant = $participants.find(
        p => p.id !== $authStore.user?.id
      );
      return otherParticipant?.name || 'Utilisateur inconnu';
    }
    
    // Groupe - utiliser le nom ou "Groupe sans nom"
    return $activeConversation.name || 'Groupe sans nom';
  }
);

// Fonction utilitaire pour générer un ID de conversation unique basé sur les participants
export function generateConversationId(participantIds: string[]): string {
  // Tri pour que l'ordre n'importe pas
  const sortedIds = [...participantIds].sort();
  return `conv_${sortedIds.join('_')}_${Date.now()}`;
}

// Initialiser les conversations au démarrage
export async function initConversations() {
  const user = get(authStore).user;
  if (user) {
    await loadConversations();
    await loadAvailableUsers();
  }
}

// Écouter les changements d'authentification
authStore.subscribe(($authStore) => {
  if ($authStore.isAuthenticated && !$authStore.loading) {
    initConversations().catch(console.error);
  }
});

// Écouter les nouveaux messages via WebSocket
export function setupMessageWebSocket() {
  const ws = new WebSocket(`ws://${window.location.host}/ws/messages`);
  
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.type === 'new_message') {
      const message = data.message;
      
      // Mettre à jour la conversation concernée
      conversations.update(convs => {
        return convs.map(conv => {
          if (conv.id === message.conversation_id) {
            return {
              ...conv,
              last_message_at: message.timestamp,
              last_message_preview: '[Nouveau message]',
              unread_count: conv.id === get(activeConversationId) ? 0 : conv.unread_count + 1
            };
          }
          return conv;
        });
      });
      
      // Si c'est la conversation active, charger les nouveaux messages
      if (message.conversation_id === get(activeConversationId)) {
        loadMessages(message.conversation_id).catch(console.error);
      }
    }
  };
  
  ws.onerror = (error) => {
    console.error('WebSocket error:', error);
  };
  
  return () => ws.close();
}