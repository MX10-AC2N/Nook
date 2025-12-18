// /frontend/src/lib/auth.js
import { goto } from '$app/navigation';

/**
 * Vérifie l'état d'authentification de l'utilisateur.
 * Effectue des appels API pour savoir qui il est.
 * @returns {Promise<{status: string, memberId: string|null}>}
 */
export async function checkAuth() {
  try {
    // 1. Vérifier si on a un cookie de session utilisateur (membre approuvé)
    const sessionResponse = await fetch('/api/validate-session', {
      credentials: 'include' // IMPORTANT : envoie les cookies
    });

    if (sessionResponse.ok) {
      // L'utilisateur a une session valide, il est "approved"
      const data = await sessionResponse.json();
      return { status: 'approved', memberId: data.member_id };
    }

    // 2. Vérifier si on a un cookie de session admin
    const adminResponse = await fetch('/api/admin/validate', {
      credentials: 'include'
    });
    if (adminResponse.ok) {
      return { status: 'admin', memberId: null };
    }

    // 3. Vérifier si on est sur une page avec un token d'invitation
    // (Cette logique pourrait être déplacée dans +page.svelte)
    const urlParams = new URLSearchParams(window.location.search);
    const inviteToken = urlParams.get('token');

    if (inviteToken) {
      // On pourrait stocker temporairement le token pour la suite
      sessionStorage.setItem('pending_invite_token', inviteToken);
      return { status: 'guest', memberId: null };
    }

    // 4. Par défaut : statut invité (pour demander à rejoindre)
    // (Dans un futur, on pourrait avoir une page publique sans invitation)
    return { status: 'guest', memberId: null };

  } catch (error) {
    console.error('Auth check failed:', error);
    throw error;
  }
}

/**
 * Effectue la connexion (login) pour un membre approuvé.
 * @param {string} memberId
 */
export async function login(memberId) {
  const response = await fetch('/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ member_id: memberId }),
    credentials: 'include'
  });

  if (!response.ok) {
    throw new Error('Login failed');
  }
  return response.json();
}

/**
 * Déconnexion (logout).
 */
export async function logout() {
  await fetch('/api/logout', {
    method: 'POST',
    credentials: 'include'
  });
  goto('/');
}

/**
 * Récupère la liste des membres (nécessite des droits admin).
 */
export async function getMembers() {
  const response = await fetch('/api/admin/members', {
    credentials: 'include'
  });
  if (!response.ok) throw new Error('Failed to fetch members');
  return response.json();
}