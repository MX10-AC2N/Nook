import { $state, $derived } from 'svelte';

// Création d'une classe pour encapsuler l'état authentifié
class AuthStore {
  user = $state(null);
  token = $state(null);
  
  constructor() {
    // Charger l'utilisateur depuis localStorage au démarrage
    if (typeof window !== 'undefined') {
      const savedUser = localStorage.getItem('user');
      const savedToken = localStorage.getItem('token');
      
      if (savedUser) {
        try {
          this.user = JSON.parse(savedUser);
        } catch (e) {
          console.error('Error parsing saved user:', e);
          this.user = null;
        }
      }
      
      if (savedToken) {
        this.token = savedToken;
      }
    }
  }
  
  // Méthode pour se connecter
  login(userData, token) {
    this.user = userData;
    this.token = token;
    
    if (typeof window !== 'undefined') {
      localStorage.setItem('user', JSON.stringify(userData));
      localStorage.setItem('token', token);
    }
  }
  
  // Méthode pour se déconnecter
  logout() {
    this.user = null;
    this.token = null;
    
    if (typeof window !== 'undefined') {
      localStorage.removeItem('user');
      localStorage.removeItem('token');
    }
  }
  
  // Getter pour vérifier si l'utilisateur est authentifié
  get isAuthenticated() {
    return $derived(this.user !== null && this.token !== null);
  }
  
  // Getter pour récupérer les headers d'authentification
  get authHeaders() {
    return $derived(this.token ? { Authorization: `Bearer ${this.token}` } : {});
  }
}

// Exporter une seule instance du store
export const authStore = new AuthStore();

// Exporter des fonctions utilitaires au lieu d'exporter directement des valeurs dérivées
export function getIsAuthenticated() {
  return authStore.isAuthenticated;
}

export function getAuthHeaders() {
  return authStore.authHeaders;
}

export function getCurrentUser() {
  return authStore.user;
}
