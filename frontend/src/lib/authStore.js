// src/lib/authStore.js
import { writable, derived } from 'svelte/store';
import { checkAuth, logout as apiLogout } from './auth.js';

/**
 * @typedef {Object} AuthUser
 * @property {string} id
 * @property {string} username
 * @property {string} email
 * @property {string} role
 * @property {boolean} needs_password_change
 */

/**
 * @typedef {Object} AuthState
 * @property {boolean} isAuthenticated
 * @property {boolean} isAdmin
 * @property {AuthUser|null} user
 * @property {boolean} loading
 * @property {boolean} needsPasswordChange
 */

/**
 * Crée le store d’authentification avec des helpers.
 *
 * @returns {{ subscribe: Function, set: Function, update: Function,
 *           setLoading: Function, setAuthenticated: Function,
 *           setGuest: Function, setError: Function,
 *           updateUser: Function }}
 */
function createAuthStore() {
  /** @type {AuthState} */
  const initialState = {
    isAuthenticated: false,
    isAdmin: false,
    user: null,
    loading: true,
    needsPasswordChange: false,
  };

  const { subscribe, set, update } = writable(initialState);

  /**
   * Helper interne – merge un état partiel avec l’état actuel.
   *
   * @param {Partial<AuthState>} partial
   */
  function setState(partial) {
    update((prev) => ({
      ...prev,
      ...partial,
    }));
  }

  return {
    subscribe,
    set,
    update,

    /** Met le store en état de chargement. */
    setLoading: () => setState({ loading: true }),

    /**
     * Marque l’utilisateur comme authentifié.
     *
     * @param {AuthUser} user
     * @param {boolean} [isAdmin=false]
     */
    setAuthenticated: (user, isAdmin = false) =>
      setState({
        isAuthenticated: true,
        isAdmin,
        user,
        loading: false,
        needsPasswordChange: !!user.needs_password_change,
      }),

    /** Marque l’état comme invité (non‑authentifié). */
    setGuest: () =>
      setState({
        isAuthenticated: false,
        isAdmin: false,
        user: null,
        loading: false,
        needsPasswordChange: false,
      }),

    /** En cas d’erreur (ex. appel API qui échoue). */
    setError: () =>
      setState({
        isAuthenticated: false,
        isAdmin: false,
        user: null,
        loading: false,
        needsPasswordChange: false,
      }),

    /**
     * Met à jour les informations de l’utilisateur (ex. changement de pseudo).
     *
     * @param {Partial<AuthUser>} userData
     */
    updateUser: (userData) =>
      update((state) => ({
        ...state,
        user: state.user ? { ...state.user, ...userData } : null,
      })),
  };
}

// ---------------------------------------------------------------------
// Export du store principal
// ---------------------------------------------------------------------
export const authStore = createAuthStore();

// ---------------------------------------------------------------------
// Stores dérivés (facilitent l’accès dans les composants)
// ---------------------------------------------------------------------
export const isAuthenticated = derived(authStore, ($s) => $s.isAuthenticated);
export const isAdmin = derived(authStore, ($s) => $s.isAdmin);
export const authUser = derived(authStore, ($s) => $s.user);
export const authLoading = derived(authStore, ($s) => $s.loading);
export const needsPasswordChange = derived(
  authStore,
  ($s) => $s.needsPasswordChange
);

// ---------------------------------------------------------------------
// Initialisation de l’authentification (appelée une seule fois côté client)
// ---------------------------------------------------------------------
let initialized = false;

/**
 * Initialise le store en interrogeant le backend.
 * Met à jour le store selon le résultat (`authenticated` ou `guest`).
 */
export async function initAuth() {
  if (initialized) return; // évite les doubles appels (ex. HMR)
  initialized = true;

  try {
    authStore.setLoading();

    const result = await checkAuth();

    if (result.status === 'authenticated' && result.user) {
      authStore.setAuthenticated(result.user, result.isAdmin);
    } else {
      authStore.setGuest();
    }
  } catch (err) {
    console.error('Auth initialization failed:', err);
    authStore.setError();
  }
}

/**
 * Déconnexion – appelle l’API puis remet le store en état de guest.
 */
export async function logout() {
  try {
    await apiLogout();
  } catch (err) {
    console.error('Logout error:', err);
  }
  authStore.setGuest();
}

/**
 * Met à jour les données de l’utilisateur dans le store.
 *
 * @param {Partial<AuthUser>} userData
 */
export function updateUser(userData) {
  authStore.updateUser(userData);
}

// ---------------------------------------------------------------------
// Lancement de l’initialisation côté client uniquement
// ---------------------------------------------------------------------
if (typeof window !== 'undefined') {
  initAuth();
}