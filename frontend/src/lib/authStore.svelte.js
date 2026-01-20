// src/lib/authStore.svelte.js (Svelte 5 – store d’authentification)

import { checkAuth, logout as apiLogout } from './auth.js';

// ---------------------------------------------------------------------
// Store réactif principal
// ---------------------------------------------------------------------
/**
 * @typedef {Object} AuthState
 * @property {boolean} isAuthenticated
 * @property {boolean} isAdmin
 * @property {Object|null} user
 * @property {boolean} loading
 * @property {boolean} needsPasswordChange
 */

/** @type {AuthState} */
export const authStore = $state({
  isAuthenticated: false,
  isAdmin: false,
  user: null,
  loading: true,
  needsPasswordChange: false,
});

// ---------------------------------------------------------------------
// Getters publics (exposés sous forme de derived stores)
// ---------------------------------------------------------------------
export const isAuthenticated   = $derived(authStore.isAuthenticated);
export const isAdmin           = $derived(authStore.isAdmin);
export const authUser          = $derived(authStore.user);
export const authLoading       = $derived(authStore.loading);
export const needsPasswordChange = $derived(authStore.needsPasswordChange);

// ---------------------------------------------------------------------
// Actions – fonctions qui mutent le store
// ---------------------------------------------------------------------
/** Met le store en état de chargement. */
export function setLoading() {
  authStore.loading = true;
}

/**
 * Marque l'utilisateur comme authentifié.
 * @param {Object} user   – objet utilisateur retourné par l’API
 * @param {boolean} [isAdmin=false]
 */
export function setAuthenticated(user, isAdmin = false) {
  Object.assign(authStore, {
    isAuthenticated: true,
    isAdmin,
    user,
    loading: false,
    needsPasswordChange: !!user?.needs_password_change,
  });
}

/** Marque l'état comme invité (non‑authentifié). */
export function setGuest() {
  Object.assign(authStore, {
    isAuthenticated: false,
    isAdmin: false,
    user: null,
    loading: false,
    needsPasswordChange: false,
  });
}

/** En cas d’erreur d’authentification. */
export function setError() {
  setGuest(); // pour l’instant on revient à l’état « guest »
}

/** Met à jour les champs de l'utilisateur sans toucher aux flags. */
export function updateUser(userData) {
  if (authStore.user) {
    authStore.user = { ...authStore.user, ...userData };
  }
}

// ---------------------------------------------------------------------
// Initialisation du store côté client
// ---------------------------------------------------------------------
let initialized = false;

/** Initialise l’état d’authentification au démarrage de l’app. */
export async function initAuth() {
  if (initialized) return;
  initialized = true;

  try {
    setLoading();
    const result = await checkAuth();

    if (result.status === 'authenticated' && result.user) {
      setAuthenticated(result.user, result.isAdmin);
    } else {
      setGuest();
    }
  } catch (err) {
    console.error('Auth initialization failed:', err);
    setError();
  }
}

/** Déconnecte l’utilisateur et remet le store en état « guest ». */
export async function logout() {
  try {
    await apiLogout();
  } catch (err) {
    console.error('Logout error:', err);
  }
  setGuest();
}

// Lancement automatique côté client (SSR désactivé)
if (typeof window !== 'undefined') {
  initAuth();
}
