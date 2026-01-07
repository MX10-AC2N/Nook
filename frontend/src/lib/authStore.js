import { writable, derived } from 'svelte/store';
import { checkAuth, logout as apiLogout } from './auth.js';

function createAuthStore() {
    const initialState = {
        isAuthenticated: false,
        isAdmin: false,
        user: null,
        loading: true,
        needsPasswordChange: false  // Nouveau : pour le premier login admin
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
            loading: false,
            needsPasswordChange: user?.needs_password_change || false
        }),
        setGuest: () => set({
            isAuthenticated: false,
            isAdmin: false,
            user: null,
            loading: false,
            needsPasswordChange: false
        }),
        setError: () => set({
            isAuthenticated: false,
            isAdmin: false,
            user: null,
            loading: false,
            needsPasswordChange: false
        }),
        updateUser: (userData) => update(state => ({
            ...state,
            user: state.user ? { ...state.user, ...userData } : null
        }))
    };
}

export const authStore = createAuthStore();

// Dérivés pour accès facile
export const isAuthenticated = derived(authStore, $store => $store.isAuthenticated);
export const isAdmin = derived(authStore, $store => $store.isAdmin);
export const authUser = derived(authStore, $store => $store.user);
export const authLoading = derived(authStore, $store => $store.loading);
export const needsPasswordChange = derived(authStore, $store => $store.needsPasswordChange);

export async function initAuth() {
    try {
        authStore.setLoading();
        const result = await checkAuth();
        console.log('initAuth() result:', result);

        if (result.status === 'authenticated' && result.user) {
            console.log('Setting authenticated, isAdmin:', result.isAdmin);
            authStore.setAuthenticated(result.user, result.isAdmin);
        } else {
            authStore.setGuest();
            console.log('Setting guest');
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

// Initialiser l'authentification au chargement de la page
if (typeof window !== 'undefined') {
    initAuth();
}