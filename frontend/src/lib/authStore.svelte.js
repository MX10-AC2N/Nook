// src/lib/authStore.svelte.js
// Store d'authentification — Svelte 5 runes (pattern propre)
//
// Architecture auth :
//   - Le token de session vit dans un cookie HttpOnly géré par le backend.
//   - Le frontend n'a PAS accès au token (c'est le but du HttpOnly).
//   - authStore.sessionId est un identifiant LOCAL (timestamp),
//     uniquement pour savoir si l'utilisateur est "connu localement".
//   - L'authentification réelle se fait via le cookie envoyé automatiquement
//     par le navigateur à chaque requête /api/* (credentials: 'include').
//
// ✅ Règles d'usage dans les composants :
//   import { authStore } from '$lib/authStore.svelte.js';
//
//   authStore.user            → utilisateur courant (réactif)
//   authStore.isAuthenticated → boolean (réactif)
//   authStore.isAdmin         → boolean (réactif)
//   authStore.loading         → boolean (réactif)
//
//   authStore.login(user)     ← signature simplifiée, plus de token
//   authStore.logout()
//   authStore.updateUser({ name: '...' })
//   await authStore.init()

import { browser } from '$app/environment';

/**
 * @typedef {Object} User
 * @property {string} id
 * @property {string} username
 * @property {string} [name]
 * @property {string} [role]
 * @property {boolean} [needs_password_change]
 * @property {string} [avatar_style]
 * @property {string} [avatar_seed]
 */

// =====================================================================
// CLASSE AuthStore — source unique de vérité
// =====================================================================
class AuthStore {
  // --- État réactif ---
  /** @type {User | null} */
  user      = $state(null);
  // sessionId : identifiant local de session, généré via Date.now()
  // Compatible HTTP (LAN) et HTTPS (WAN) — pas d'API crypto.randomUUID()
  /** @type {string | null} */
  sessionId = $state(null);
  /** @type {boolean} */
  loading   = $state(true);

  // --- Dérivés réactifs ---
  /** @type {boolean} */
  isAuthenticated     = $derived(this.user !== null && this.sessionId !== null);
  /** @type {boolean} */
  isAdmin             = $derived(this.user?.role === 'admin');
  /** @type {boolean} */
  needsPasswordChange = $derived(this.user?.needs_password_change ?? false);

  constructor() {
    if (!browser) return;
    try {
      /** @type {string | null} */
      const savedUser      = localStorage.getItem('nook_user');
      /** @type {string | null} */
      const savedSessionId = localStorage.getItem('nook_session_id');
      if (savedUser) {
        /** @type {User} */
        this.user = JSON.parse(savedUser);
      }
      if (savedSessionId) {
        /** @type {string} */
        this.sessionId = savedSessionId;
      }
    } catch (e) {
      console.error('[AuthStore] Erreur lecture localStorage :', e);
    }
  }

  // ------------------------------------------------------------------
  // login — appelé après authentification réussie.
  // Le cookie HttpOnly est déjà posé par le backend.
  // On stocke uniquement les infos user + un sessionId local (timestamp).
  // ------------------------------------------------------------------
  /**
   * @param {User} userData
   */
  login(userData) {
    /** @type {User} */
    this.user      = userData;
    /** @type {string} */
    this.sessionId = String(Date.now());
    if (browser) {
      localStorage.setItem('nook_user',       JSON.stringify(userData));
      localStorage.setItem('nook_session_id', this.sessionId);
      // Migration : supprimer l'ancien nook_token s'il traîne
      localStorage.removeItem('nook_token');
    }
  }

  // ------------------------------------------------------------------
  // logout — nettoyage complet (local + cookie révoqué côté serveur)
  // ------------------------------------------------------------------
  logout() {
    this.user      = null;
    this.sessionId = null;
    if (browser) {
      localStorage.removeItem('nook_user');
      localStorage.removeItem('nook_session_id');
      localStorage.removeItem('nook_token'); // migration
    }
  }

  // ------------------------------------------------------------------
  // updateUser — mise à jour partielle du profil sans re-login
  // ------------------------------------------------------------------
  /**
   * @param {Partial<User>} partial
   */
  updateUser(partial) {
    /** @type {Partial<User>} */
    const p = partial;
    if (!this.user) return;
    /** @type {User} */
    this.user = { ...this.user, ...p };
    if (browser) {
      localStorage.setItem('nook_user', JSON.stringify(this.user));
    }
  }

  // ------------------------------------------------------------------
  // init — vérification de session côté serveur (appelé dans layout).
  // Le cookie HttpOnly est envoyé automatiquement par le navigateur.
  // ------------------------------------------------------------------
  async init() {
    this.loading = true;
    try {
      const resp = await fetch('/api/auth/me', { credentials: 'include' });
      if (!resp.ok) {
        this.logout();
        return;
      }
      /** @type {{authenticated: boolean, user?: User}} */
      const data = await resp.json();
      if (data.authenticated && data.user) {
        /** @type {User} */
        this.user = data.user;
        // Régénérer sessionId si absent (ex: refresh navigateur)
        if (!this.sessionId) {
          /** @type {string} */
          this.sessionId = String(Date.now());
        }
        if (browser) {
          localStorage.setItem('nook_user',       JSON.stringify(this.user));
          localStorage.setItem('nook_session_id', this.sessionId);
        }
      } else {
        this.logout();
      }
    } catch (e) {
      console.error('[AuthStore] init error :', e);
      // Erreur réseau : on garde la session locale si elle existe
    } finally {
      this.loading = false;
    }
  }
}

// =====================================================================
// SINGLETON
// =====================================================================
export const authStore = new AuthStore();