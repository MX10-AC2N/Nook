import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: 'index.html' // Nécessaire pour le mode SPA
    }),
    paths: {
      base: ''
    },
    // ✅ Active le prerender pour générer index.html
    prerender: {
      enabled: true,
      entries: ['/'] // Génère seulement la page racine
    }
  }
};

export default config;