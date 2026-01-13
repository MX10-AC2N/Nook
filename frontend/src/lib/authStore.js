// src/lib/authStore.js (version Svelte 5)
import { checkAuth, logout as apiLogout } from './auth.js';

// ---------------------------------------------------------------------
// État réactif avec runes
// ---------------------------------------------------------------------

/** @type {AuthState} */
const authState = $state({
  isAuthenticated: false,
  isAdmin: false,
  user: null,
  loading: true,
  needsPasswordChange: false
});

// ---------------------------------------------------------------------
// Getters publics (exportés)
// ---------------------------------------------------------------------
export const isAuthenticated = $derived(authState.isAuthenticated);
export const isAdmin = $derived(authState.isAdmin);
export const authUser = $derived(authState.user);
export const authLoading = $derived(authState.loading);
export const needsPasswordChange = $derived(authState.needsPasswordChange);

// ---------------------------------------------------------------------
// Actions (fonctions pour modifier l'état)
// ---------------------------------------------------------------------

/** Met le store en état de chargement. */
export function setLoading() {
  authState.loading = true;
}

/** Marque l'utilisateur comme authentifié. */
export function setAuthenticated(user, isAdmin = false) {
  Object.assign(authState, {
    isAuthenticated: true,
    isAdmin,
    user,
    loading: false,
    needsPasswordChange: !!user?.needs_password_change
  });
}

/** Marque l'état comme invité (non-authentifié). */
export function setGuest() {
  Object.assign(authState, {
    isAuthenticated: false,
    isAdmin: false,
    user: null,
    loading: false,
    needsPasswordChange: false
  });
}

/** En cas d'erreur. */
export function setError() {
  setGuest(); // Même état que guest pour l'instant
}

/** Met à jour les informations de l'utilisateur. */
export function updateUser(userData) {
  if (authState.user) {
    authState.user = { ...authState.user, ...userData };
  }
}

// ---------------------------------------------------------------------
// Initialisation (identique à votre code)
// ---------------------------------------------------------------------
let initialized = false;

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

export async function logout() {
  try {
    await apiLogout();
  } catch (err) {
    console.error('Logout error:', err);
  }
  setGuest();
}

// Initialisation côté client
if (typeof window !== 'undefined') {
  initAuth();
}