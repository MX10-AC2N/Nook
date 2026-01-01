import { writable, derived } from 'svelte/store';
import { checkAuth, logout as apiLogout } from './auth.js';

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

export const isAuthenticated = derived(authStore, $store => $store.isAuthenticated);
export const isAdmin = derived(authStore, $store => $store.isAdmin);
export const authUser = derived(authStore, $store => $store.user);
export const authLoading = derived(authStore, $store => $store.loading);

export async function initAuth() {
    try {
        authStore.setLoading();
        const result = await checkAuth();

        if (result.status === 'authenticated' && result.user) {
            authStore.setAuthenticated(result.user, result.isAdmin);
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

// Initialiser au chargement de la page
if (typeof window !== 'undefined') {
    initAuth();
}