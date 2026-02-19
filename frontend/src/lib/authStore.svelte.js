// src/lib/authStore.svelte.js
// Store d'authentification — Svelte 5 runes (pattern propre)
//
// ✅ Règles d'usage dans les composants :
//   import { authStore } from '$lib/authStore.svelte.js';
//
//   authStore.user          → utilisateur courant (réactif)
//   authStore.isAuthenticated → boolean (réactif)
//   authStore.isAdmin       → boolean (réactif)
//   authStore.loading       → boolean (réactif)
//
//   authStore.login(user, token)
//   authStore.logout()
//   authStore.updateUser({ name: '...' })
//   await authStore.init()

import { browser } from '$app/environment';

// =====================================================================
// CLASSE AuthStore — source unique de vérité
// =====================================================================
class AuthStore {
  // --- État réactif ---
  user    = $state(null);
  token   = $state(null);
  loading = $state(true);

  // --- Dérivés réactifs ---
  isAuthenticated  = $derived(this.user !== null && this.token !== null);
  isAdmin          = $derived(this.user?.role === 'admin');
  needsPasswordChange = $derived(this.user?.needs_password_change ?? false);
  authHeaders      = $derived(
    this.token ? { Authorization: `Bearer ${this.token}` } : {}
  );

  constructor() {
    if (!browser) return;
    try {
      const savedUser  = localStorage.getItem('nook_user');
      const savedToken = localStorage.getItem('nook_token');
      if (savedUser)  this.user  = JSON.parse(savedUser);
      if (savedToken) this.token = savedToken;
    } catch (e) {
      console.error('[AuthStore] Erreur lecture localStorage :', e);
    }
  }

  // ------------------------------------------------------------------
  // login — appelé après authentification réussie
  // ------------------------------------------------------------------
  login(userData, token) {
    this.user  = userData;
    this.token = token;
    if (browser) {
      localStorage.setItem('nook_user',  JSON.stringify(userData));
      localStorage.setItem('nook_token', token);
    }
  }

  // ------------------------------------------------------------------
  // logout — nettoyage complet
  // ------------------------------------------------------------------
  logout() {
    this.user  = null;
    this.token = null;
    if (browser) {
      localStorage.removeItem('nook_user');
      localStorage.removeItem('nook_token');
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
  // init — vérification de session côté serveur (appelé dans layout)
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
        // On conserve le token existant s'il n'en arrive pas un nouveau
        this.user  = data.user;
        this.token = data.token ?? this.token ?? crypto.randomUUID();
        if (browser) {
          localStorage.setItem('nook_user',  JSON.stringify(this.user));
          localStorage.setItem('nook_token', this.token);
        }
      } else {
        this.logout();
      }
    } catch (e) {
      console.error('[AuthStore] init error :', e);
      this.logout();
    } finally {
      this.loading = false;
    }
  }
}

// =====================================================================
// SINGLETON — une seule instance pour toute l'application
// =====================================================================
export const authStore = new AuthStore();
