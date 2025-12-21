import { createRoutify } from '@roxi/routify';
import App from './App.svelte';
import { authStore } from '$lib/authStore';
import { initCrypto } from '$lib/crypto';

// Initialiser Routify avec les routes
const routify = createRoutify({
  routes: import.meta.glob('./routes/**/+page.svelte', { eager: true }),
  layouts: {
    default: () => import('./routes/+layout.svelte')
  },
  options: {
    svelte5: true,
    ssr: false,
    transitions: true,
    linkPreload: true
  }
});

// Initialiser l'application
async function initApp() {
  try {
    // Initialiser le chiffrement
    await initCrypto();
    
    // Créer l'application Svelte
    const app = new App({
      target: document.getElementById('app'),
      props: {
        routify
      },
      runes: true,
      hydrate: import.meta.env.PROD
    });
    
    // Gestion des erreurs globales
    window.addEventListener('unhandledrejection', (event) => {
      console.error('Unhandled promise rejection:', event.reason);
    });
    
    window.addEventListener('error', (event) => {
      console.error('Global error:', event.error);
    });
    
    console.log('🚀 Nook frontend initialisé avec Svelte 5 et Routify');
    return app;
  } catch (error) {
    console.error('❌ Erreur lors de l\'initialisation de l\'application:', error);
    
    // Afficher une page d'erreur utilisateur
    document.getElementById('app').innerHTML = `
      <div style="display: flex; justify-content: center; align-items: center; min-height: 100vh; flex-direction: column; padding: 2rem; text-align: center;">
        <h1 style="font-size: 2.5rem; color: #ef5350; margin-bottom: 1rem;">🚀 Nook</h1>
        <p style="font-size: 1.2rem; margin-bottom: 2rem; color: #666;">Impossible de charger l'application</p>
        <p style="background: #ffebee; color: #c62828; padding: 1rem; border-radius: 8px; margin-bottom: 1rem; font-family: monospace;">${error.message || 'Erreur inconnue'}</p>
        <button onclick="location.reload()" style="background: #4caf50; color: white; border: none; padding: 0.75rem 1.5rem; border-radius: 8px; font-size: 1rem; cursor: pointer; transition: opacity 0.2s;">
          🔄 Recharger l'application
        </button>
        <p style="margin-top: 2rem; color: #999; font-size: 0.9rem;">Si le problème persiste, contactez l'administrateur</p>
      </div>
    `;
  }
}

// Démarrer l'application quand le DOM est prêt
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}