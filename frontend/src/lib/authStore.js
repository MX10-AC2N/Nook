import { writable, derived } from 'svelte/store';
import { checkAuth, logout as apiLogout } from './auth.js';

let authState = $state({
	isAuthenticated: false,
	isAdmin: false,
	user: null,
	loading: true
});

const store = writable(authState);

export const isAuthenticated = derived(store, $s => $s.isAuthenticated);
export const isAdmin = derived(store, $s => $s.isAdmin);
export const authUser = derived(store, $s => $s.user);
export const authLoading = derived(store, $s => $s.loading);

export async function initAuth() {
	try {
		const tempStore = writable({
			isAuthenticated: false,
			isAdmin: false,
			user: null,
			loading: true
		});
		
		const result = await checkAuth();
		
		if (result.status === 'approved' || result.status === 'admin') {
			const userResponse = await fetch('/api/user-info', { credentials: 'include' });
			let userData = null;
			if (userResponse.ok) userData = await userResponse.json();
			
			const newState = { isAuthenticated: true, isAdmin: result.status === 'admin', user: userData, loading: false };
			tempStore.set(newState);
			authState = newState;
			store.set(newState);
		} else {
			const newState = { isAuthenticated: false, isAdmin: false, user: null, loading: false };
			tempStore.set(newState);
			authState = newState;
			store.set(newState);
		}
	} catch (error) {
		console.error('Auth initialization failed:', error);
		const errorState = { isAuthenticated: false, isAdmin: false, user: null, loading: false };
		store.set(errorState);
		authState = errorState;
	}
}

export async function logout() {
	try {
		await apiLogout();
	} catch (err) {
		console.error('Logout error:', err);
	}
	const newState = { isAuthenticated: false, isAdmin: false, user: null, loading: false };
	authState = newState;
	store.set(newState);
}

export function updateUser(userData) {
	const newState = { ...authState, user: { ...authState.user, ...userData } };
	authState = newState;
	store.set(newState);
}

initAuth();

export const authStore = {
	subscribe: store.subscribe,
	set: store.set,
	update: store.update,
	isAuthenticated,
	isAdmin,
	authUser,
	authLoading,
	logout,
	updateUser,
	initAuth
};
