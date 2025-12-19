import { writable, derived } from 'svelte/store';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';

// Store pour l'état d'authentification
export const user = writable(null);
export const isAuthenticated = writable(false);
export const isLoading = writable(false);
export const isAdmin = writable(false);

// Vérifier l'authentification au chargement
if (browser) {
    checkAuth();
}

/**
 * Vérifie l'état d'authentification avec le backend
 */
export async function checkAuth() {
    if (!browser) return;
    
    isLoading.set(true);
    
    try {
        // Vérifier session utilisateur normale
        const userRes = await fetch('/api/validate-session', {
            credentials: 'include'
        });
        
        if (userRes.ok) {
            const userData = await userRes.json();
            user.set(userData);
            isAuthenticated.set(true);
            isAdmin.set(false);
            console.log('Utilisateur connecté:', userData);
            isLoading.set(false);
            return true;
        }
        
        // Vérifier session admin
        const adminRes = await fetch('/api/admin/validate', {
            credentials: 'include'
        });
        
        if (adminRes.ok) {
            const adminData = await adminRes.json();
            user.set({ 
                member_id: 'admin',
                member_name: 'Administrateur',
                isAdmin: true 
            });
            isAuthenticated.set(true);
            isAdmin.set(true);
            console.log('Admin connecté');
            isLoading.set(false);
            return true;
        }
        
        // Aucune session valide
        user.set(null);
        isAuthenticated.set(false);
        isAdmin.set(false);
        return false;
        
    } catch (error) {
        console.error('Erreur vérification auth:', error);
        user.set(null);
        isAuthenticated.set(false);
        isLoading.set(false);
        return false;
    } finally {
        isLoading.set(false);
    }
}

/**
 * Connexion d'un membre
 */
export async function loginMember(identifier, password) {
    isLoading.set(true);
    
    try {
        const response = await fetch('/api/member/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ identifier, password }),
            credentials: 'include'
        });

        if (response.ok) {
            await checkAuth();
            goto('/chat');
            return { success: true };
        } else {
            return { 
                success: false, 
                error: 'Identifiants incorrects ou compte non approuvé' 
            };
        }
    } catch (error) {
        console.error('Erreur connexion:', error);
        return { 
            success: false, 
            error: 'Impossible de contacter le serveur' 
        };
    } finally {
        isLoading.set(false);
    }
}

/**
 * Connexion admin
 */
export async function loginAdmin(username, password) {
    isLoading.set(true);
    
    try {
        const response = await fetch('/api/admin/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
            credentials: 'include'
        });

        if (response.ok) {
            await checkAuth();
            goto('/admin');
            return { success: true };
        } else {
            return { 
                success: false, 
                error: 'Identifiants admin incorrects' 
            };
        }
    } catch (error) {
        console.error('Erreur connexion admin:', error);
        return { 
            success: false, 
            error: 'Impossible de contacter le serveur' 
        };
    } finally {
        isLoading.set(false);
    }
}

/**
 * Déconnexion
 */
export async function logout() {
    try {
        // Déconnexion admin si c'est un admin
        if (isAdmin) {
            await fetch('/api/admin/logout', {
                method: 'POST',
                credentials: 'include'
            });
        }
        
        // Nettoyer les stores
        user.set(null);
        isAuthenticated.set(false);
        isAdmin.set(false);
        
        // Rediriger vers la page de login
        goto('/login');
        
    } catch (error) {
        console.error('Erreur déconnexion:', error);
    }
}

/**
 * Vérifie si un membre a un mot de passe défini
 */
export async function checkPasswordStatus() {
    try {
        const response = await fetch('/api/member/check-password', {
            credentials: 'include'
        });
        
        if (response.ok) {
            return await response.json();
        }
        return null;
    } catch (error) {
        console.error('Erreur vérification mot de passe:', error);
        return null;
    }
}

/**
 * Crée un mot de passe pour le membre actuel
 */
export async function createPassword(password) {
    try {
        const response = await fetch('/api/member/create-password', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ password }),
            credentials: 'include'
        });
        
        return response.ok;
    } catch (error) {
        console.error('Erreur création mot de passe:', error);
        return false;
    }
}

/**
 * Middleware pour protéger les routes
 */
export function requireAuth() {
    if (!browser) return true;
    
    let authState = false;
    const unsubscribe = isAuthenticated.subscribe(value => {
        authState = value;
    });
    unsubscribe();
    
    return authState;
}

// Store dérivé pour les infos utilisateur formatées
export const userInfo = derived(
    [user, isAuthenticated, isAdmin],
    ([$user, $isAuthenticated, $isAdmin]) => ({
        ...$user,
        isAuthenticated: $isAuthenticated,
        isAdmin: $isAdmin,
        displayName: $user?.member_name || 'Invité'
    })
);