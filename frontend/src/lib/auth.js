import { goto } from '$app/navigation';
import { browser } from '$app/environment';

export async function checkAuth() {
    try {
        // Utilise l'endpoint moderne qui retourne authenticated + user (avec role)
        const response = await fetch('/api/validate-session', { credentials: 'include' });

        if (response.ok) {
            const data = await response.json();
            if (data.authenticated && data.user) {
                return {
                    status: 'authenticated',
                    user: data.user,              // contient id, username, name, role, etc.
                    isAdmin: data.user.role === 'admin'
                };
            }
        }

        // Gestion du token d'invitation (si tu gardes cette feature legacy)
        if (browser) {
            const urlParams = new URLSearchParams(window.location.search);
            const inviteToken = urlParams.get('token');
            if (inviteToken) {
                sessionStorage.setItem('pending_invite_token', inviteToken);
                return { status: 'guest', user: null, isAdmin: false };
            }
        }

        return { status: 'guest', user: null, isAdmin: false };
    } catch (error) {
        console.error('Auth check failed:', error);
        return { status: 'guest', user: null, isAdmin: false };
    }
}

export async function login(username, password) {
    const response = await fetch('/api/login', {  // ou /api/login-json si tu utilises l'endpoint JSON
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
        credentials: 'include'
    });

    if (!response.ok) {
        throw new Error('Login failed');
    }
    return response.json();
}

export async function logout() {
    try {
        await fetch('/api/logout', { method: 'POST', credentials: 'include' });
    } catch (err) {
        console.error('Logout error:', err);
    }
    if (browser) goto('/');
}

export function getPendingInviteToken() {
    if (!browser) return null;
    const token = sessionStorage.getItem('pending_invite_token');
    if (token) sessionStorage.removeItem('pending_invite_token');
    return token;
}

export function setPendingInviteToken(token) {
    if (browser) sessionStorage.setItem('pending_invite_token', token);
}