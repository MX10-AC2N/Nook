import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: 'index.html'  // Génère un index.html comme fallback SPA
    }),
    paths: {
      base: '/app/static'  // '' car servi à la racine (par Axum)
    }
  }
};

export default config;