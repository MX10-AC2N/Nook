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

// =====================================================================
// CLASSE AuthStore — source unique de vérité
// =====================================================================
class AuthStore {
  // --- État réactif ---
  user      = $state(null);
  // sessionId : identifiant local de session, généré via Date.now()
  // Compatible HTTP (LAN) et HTTPS (WAN) — pas d'API crypto.randomUUID()
  sessionId = $state(null);
  loading   = $state(true);

  // --- Dérivés réactifs ---
  isAuthenticated     = $derived(this.user !== null && this.sessionId !== null);
  isAdmin             = $derived(this.user?.role === 'admin');
  needsPasswordChange = $derived(this.user?.needs_password_change ?? false);

  constructor() {
    if (!browser) return;
    try {
      const savedUser      = localStorage.getItem('nook_user');
      const savedSessionId = localStorage.getItem('nook_session_id');
      if (savedUser)      this.user      = JSON.parse(savedUser);
      if (savedSessionId) this.sessionId = savedSessionId;
    } catch (e) {
      console.error('[AuthStore] Erreur lecture localStorage :', e);
    }
  }

  // ------------------------------------------------------------------
  // login — appelé après authentification réussie.
  // Le cookie HttpOnly est déjà posé par le backend.
  // On stocke uniquement les infos user + un sessionId local (timestamp).
  // ------------------------------------------------------------------
  login(userData) {
    this.user      = userData;
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
  updateUser(partial) {
    if (!this.user) return;
    this.user = { ...this.user, ...partial };
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
      const data = await resp.json();
      if (data.authenticated && data.user) {
        this.user = data.user;
        // Régénérer sessionId si absent (ex: refresh navigateur)
        if (!this.sessionId) {
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