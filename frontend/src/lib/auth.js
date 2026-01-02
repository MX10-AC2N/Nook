import { goto } from '$app/navigation';
import { browser } from '$app/environment';

export async function checkAuth() {
    try {
        const response = await fetch('/api/validate-session', { credentials: 'include' });

        if (response.ok) {
            const data = await response.json();
            if (data.authenticated && data.user) {
                return {
                    status: 'authenticated',
                    user: data.user,
                    isAdmin: data.user.role === 'admin',
                    needsPasswordChange: data.user.needs_password_change || false
                };
            }
        }

        // Gestion du token d'invitation (legacy, garde si tu en as besoin)
        if (browser) {
            const urlParams = new URLSearchParams(window.location.search);
            const inviteToken = urlParams.get('token');
            if (inviteToken) {
                sessionStorage.setItem('pending_invite_token', inviteToken);
                return { status: 'guest', user: null, isAdmin: false, needsPasswordChange: false };
            }
        }

        return { status: 'guest', user: null, isAdmin: false, needsPasswordChange: false };
    } catch (error) {
        console.error('Auth check failed:', error);
        return { status: 'guest', user: null, isAdmin: false, needsPasswordChange: false };
    }
}

export async function login(username, password) {
    const response = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
        credentials: 'include'
    });

    if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || 'Identifiants incorrects ou erreur serveur');
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