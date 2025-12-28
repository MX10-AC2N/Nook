import { writable, derived } from 'svelte/store';
import { checkAuth, logout as apiLogout } from './auth.js';

// État d'authentification avec stores Svelte classiques
function createAuthStore() {
  const initialState = {
    isAuthenticated: false,
    isAdmin: false,
    user: null,
    loading: true
  };

  const { subscribe, set, update } = writable(initialState);

  return {
    subscribe,
    set,
    update,
    setLoading: () => update(state => ({ ...state, loading: true })),
    setAuthenticated: (user, isAdmin = false) => set({
      isAuthenticated: true,
      isAdmin,
      user,
      loading: false
    }),
    setGuest: () => set({
      isAuthenticated: false,
      isAdmin: false,
      user: null,
      loading: false
    }),
    setError: () => set({
      isAuthenticated: false,
      isAdmin: false,
      user: null,
      loading: false
    }),
    updateUser: (userData) => update(state => ({
      ...state,
      user: state.user ? { ...state.user, ...userData } : null
    }))
  };
}

export const authStore = createAuthStore();

// Dérivés pour l'accès facile
export const isAuthenticated = derived(authStore, $store => $store.isAuthenticated);
export const isAdmin = derived(authStore, $store => $store.isAdmin);
export const authUser = derived(authStore, $store => $store.user);
export const authLoading = derived(authStore, $store => $store.loading);

export async function initAuth() {
  try {
    authStore.setLoading();
    const result = await checkAuth();
    
    if (result.status === 'approved' || result.status === 'admin') {
      const userResponse = await fetch('/api/user-info', { credentials: 'include' });
      let userData = null;
      if (userResponse.ok) {
        userData = await userResponse.json();
      }
      authStore.setAuthenticated(userData, result.status === 'admin');
    } else {
      authStore.setGuest();
    }
  } catch (error) {
    console.error('Auth initialization failed:', error);
    authStore.setError();
  }
}

export async function logout() {
  try {
    await apiLogout();
  } catch (err) {
    console.error('Logout error:', err);
  }
  authStore.setGuest();
}

export function updateUser(userData) {
  authStore.updateUser(userData);
}

// Initialiser l'authentification au chargement
if (typeof window !== 'undefined') {
  initAuth();
}
