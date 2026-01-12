// src/lib/auth.js
import { goto } from '$app/navigation';
import { browser } from '$app/environment';

/**
 * @typedef {Object} AuthUser
 * @property {string} id
 * @property {string} username
 * @property {string} email
 * @property {string} role
 * @property {boolean} needs_password_change
 */

/**
 * @typedef {Object} AuthResult
 * @property {'authenticated'|'guest'} status
 * @property {AuthUser|null} user
 * @property {boolean} isAdmin
 * @property {boolean} needsPasswordChange
 */

/**
 * Vérifie la session courante auprès du backend.
 * Retourne toujours un objet au même format.
 *
 * @returns {Promise<AuthResult>}
 */
export async function checkAuth() {
  try {
    // 1️⃣ Tentative d’appel à l’endpoint `/api/auth/me`
    const response = await fetch('/api/auth/me', {
      credentials: 'include',
    });

    if (response.ok) {
      const data = await response.json();

      // Si le backend confirme que l’utilisateur est authentifié
      if (data.authenticated && data.user) {
        const isAdmin = data.user.role === 'admin';
        return {
          status: 'authenticated',
          user: data.user,
          isAdmin,
          needsPasswordChange: !!data.user.needs_password_change,
        };
      }
    }

    // -----------------------------------------------------------------
    // Gestion du token d’invitation (si présent dans l’URL)
    // -----------------------------------------------------------------
    if (browser) {
      const urlParams = new URLSearchParams(window.location.search);
      const inviteToken = urlParams.get('token');

      if (inviteToken) {
        // On garde le token en sessionStorage pour le récupérer plus tard
        sessionStorage.setItem('pending_invite_token', inviteToken);
        return {
          status: 'guest',
          user: null,
          isAdmin: false,
          needsPasswordChange: false,
        };
      }
    }

    // Cas par défaut : aucune session valide
    return {
      status: 'guest',
      user: null,
      isAdmin: false,
      needsPasswordChange: false,
    };
  } catch (error) {
    console.error('Auth check failed:', error);
    return {
      status: 'guest',
      user: null,
      isAdmin: false,
      needsPasswordChange: false,
    };
  }
}

/**
 * Effectue la connexion.
 *
 * @param {string} username
 * @param {string} password
 * @returns {Promise<AuthUser>}  L’objet utilisateur renvoyé par le backend.
 * @throws {Error}            Si le login échoue (mauvais identifiants ou serveur).
 */
export async function login(username, password) {
  const response = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
    credentials: 'include',
  });

  // Lecture du corps **une seule fois**
  const data = await response.json();

  // Si le statut HTTP n’est pas 2xx OU que le backend indique un échec
  if (!response.ok || !data.success) {
    // Le backend renvoie souvent `message` ; sinon on utilise un texte générique
    throw new Error(data.message ?? 'Identifiants incorrects ou erreur serveur');
  }

  // Retour du user (et éventuellement du token si besoin)
  return data.user;
}

/**
 * Déconnexion de l’utilisateur.
 * Redirige vers la page d’accueil après la requête.
 */
export async function logout() {
  try {
    await fetch('/api/auth/logout', {
      method: 'POST',
      credentials: 'include',
    });
  } catch (err) {
    console.error('Logout error:', err);
  }

  if (browser) goto('/');
}

/**
 * Récupère le token d’invitation stocké en sessionStorage (et le supprime).
 *
 * @returns {string|null}
 */
export function getPendingInviteToken() {
  if (!browser) return null;

  const token = sessionStorage.getItem('pending_invite_token');
  if (token) sessionStorage.removeItem('pending_invite_token');
  return token;
}

/**
 * Stocke un token d’invitation en sessionStorage.
 *
 * @param {string} token
 */
export function setPendingInviteToken(token) {
  if (browser) sessionStorage.setItem('pending_invite_token', token);
}