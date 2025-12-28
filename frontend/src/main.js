import { mount } from 'svelte';
import App from './App.svelte';

// Initialiser l'application SvelteKit/Svelte 5
const app = mount(App, {
  target: document.getElementById('app')
});

// Gestion des erreurs globales
window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
});

window.addEventListener('error', (event) => {
  console.error('Global error:', event.error);
});

console.log('🚀 Nook frontend initialisé avec Svelte 5 et SvelteKit');

export default app;
