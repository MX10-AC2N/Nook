import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: false,        // ← Désactive le runes mode par défaut
    accessors: false,    // Nettoie les warnings accessors
    immutable: false     // Nettoie les warnings immutable
  },
  kit: {
    adapter: adapter({
      fallback: 'index.html'  // Génère un index.html comme fallback SPA
    }),
    paths: {
      base: ''  // '' car servi à la racine (par Axum)
    }
  }
};

export default config;