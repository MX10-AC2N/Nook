// src/lib/authStore.svelte.js
// Store d'authentification — Svelte 5 runes
// Pattern : classe avec $state + exports de compatibilité pour tout le projet

import { browser } from '$app/environment';

// =====================================================================
// CLASSE AuthStore — état réactif principal
// =====================================================================
class AuthStore {
  user = $state(null);
  token = $state(null);

  isAuthenticated = $derived(this.user !== null && this.token !== null);
  isAdmin = $derived(this.user?.role === 'admin');
  authHeaders = $derived(
    this.token ? { Authorization: `Bearer ${this.token}` } : {}
  );

  constructor() {
    if (!browser) return;
    try {
      const savedUser = localStorage.getItem('nook_user');
      const savedToken = localStorage.getItem('nook_token');
      if (savedUser) this.user = JSON.parse(savedUser);
      if (savedToken) this.token = savedToken;
    } catch (e) {
      console.error('[AuthStore] Erreur lecture localStorage :', e);
    }
  }

  login(userData, token) {
    this.user = userData;
    this.token = token;
    if (browser) {
      localStorage.setItem('nook_user', JSON.stringify(userData));
      localStorage.setItem('nook_token', token);
    }
  }

  logout() {
    this.user = null;
    this.token = null;
    if (browser) {
      localStorage.removeItem('nook_user');
      localStorage.removeItem('nook_token');
    }
  }
}

// =====================================================================
// SINGLETON exporté
// =====================================================================
export const authStore = new AuthStore();

// =====================================================================
// ÉTAT DE CHARGEMENT (init async au mount)
// =====================================================================
export let authLoading = $state(true);

// =====================================================================
// EXPORTS DE COMPATIBILITÉ
// Tous les fichiers du projet importent ces noms directement.
// Ce sont des getters réactifs vers l'instance authStore.
// =====================================================================

/**
 * Utilisateur courant (null si déconnecté).
 * Usage : authUser?.id, authUser?.name, authUser?.role
 */
export function authUser() {
  return authStore.user;
}

/**
 * true si l'utilisateur est connecté et a un token valide.
 */
export function isAuthenticated() {
  return authStore.isAuthenticated;
}

/**
 * true si l'utilisateur a le rôle 'admin'.
 */
export function isAdmin() {
  return authStore.isAdmin;
}

/**
 * true si le compte nécessite un changement de mot de passe (premier login admin).
 */
export function needsPasswordChange() {
  return authStore.user?.needs_password_change ?? false;
}

/**
 * Mise à jour de l'état auth après login réussi.
 * Appelé dans login/+page.svelte après retour API.
 */
export function setAuthenticated(userData, _isAdminFlag) {
  authStore.login(userData, userData.token ?? crypto.randomUUID());
}

// =====================================================================
// INITIALISATION — vérifier la session côté serveur au démarrage
// Doit être appelé dans onMount du layout principal.
// =====================================================================
export async function initAuth() {
  authLoading = true;
  try {
    const resp = await fetch('/api/auth/me', { credentials: 'include' });
    if (!resp.ok) {
      authStore.logout();
      return;
    }
    const data = await resp.json();
    if (data.authenticated && data.user) {
      authStore.user = data.user;
    } else {
      authStore.logout();
    }
  } catch (e) {
    console.error('[AuthStore] initAuth error :', e);
    authStore.logout();
  } finally {
    authLoading = false;
  }
}

// =====================================================================
// HELPERS (accesseurs fonctionnels — compatibilité anciens usages)
// =====================================================================
export function getIsAuthenticated() {
  return authStore.isAuthenticated;
}

export function getAuthHeaders() {
  return authStore.authHeaders;
}

export function getCurrentUser() {
  return authStore.user;
}