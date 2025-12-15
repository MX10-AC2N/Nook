import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      // Mode SPA pur : génère seulement les assets client + index.html comme fallback
      fallback: 'index.html'
    }),
    paths: {
      base: process.env.NODE_ENV === 'production' ? '' : ''  // '' est OK si servi à la racine
    },
    // Important : désactive le prerender global pour éviter les pages statiques vides
    prerender: {
      enabled: false
    }
  }
};

export default config;