import { goto } from '$app/navigation';
import { browser } from '$app/environment';

export async function checkAuth() {
    try {
        const response = await fetch('/api/auth/me', { credentials: 'include' });
    
        if (response.ok) {
            const data = await response.json();
            console.log('Session validation data:', data);
       
            if (data.authenticated && data.user) {
                const isAdmin = data.user.role === 'admin';
                console.log('Is admin?', isAdmin); // ← AJOUTEZ CE LOG
                
                return {
                    status: 'authenticated',
                    user: data.user,
                    isAdmin: isAdmin,
                    needsPasswordChange: data.user.needs_password_change || false
                };
            }
        }

        // Gestion du token d'invitation
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
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
        credentials: 'include'
    });

    // 1. Lire la réponse UNE SEULE FOIS
    const data = await response.json();

    // 2. Vérifier le statut HTTP et la logique métier
    if (!response.ok || !data.success) {
        // Utiliser le message du backend ou un message par défaut
        throw new Error(data.message || 'Identifiants incorrects ou erreur serveur');
    }

    // 3. Retourner les données utilisateur
    return data.user;
}

export async function logout() {
    try {
        await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
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