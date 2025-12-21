// src/lib/authStore.js
import { writable, derived } from 'svelte/store';
import { validateSession, logout as apiLogout } from './auth.js';

const store = writable({
  isAuthenticated: false,
  isAdmin: false,
  user: null,
  loading: true
});

// Stores dérivés pour une utilisation plus simple
export const isAuthenticated = derived(store, $s => $s.isAuthenticated);
export const isAdmin = derived(store, $s => $s.isAdmin);
export const authUser = derived(store, $s => $s.user);
export const authLoading = derived(store, $s => $s.loading);

// Hydrater au démarrage
validateSession().then(data => {
  if (data) {
    store.set({
      isAuthenticated: true,
      isAdmin: data.role === 'admin',
      user: data,
      loading: false
    });
  } else {
    store.set({
      isAuthenticated: false,
      isAdmin: false,
      user: null,
      loading: false
    });
  }
});

export async function logout() {
  await apiLogout();
  store.set({
    isAuthenticated: false,
    isAdmin: false,
    user: null,
    loading: false
  });
}