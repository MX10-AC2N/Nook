// vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    // -------------------------------------------------
    // SvelteKit – le seul plugin nécessaire
    // -------------------------------------------------
    sveltekit(),
  ],

  // -------------------------------------------------
  // Tes réglages existants (chunks, alias, proxy, …)
  // -------------------------------------------------
  assetsInclude: [
    '**/*.svg',
    '**/*.png',
    '**/*.jpg',
    '**/*.jpeg',
    '**/*.ico',
  ],

  build: {
    target: 'es2020',
    rollupOptions: {
      output: {
        manualChunks(id) {
          // Séparer libsodium dans son propre chunk pour le lazy‑loading
          if (id.includes('libsodium')) return 'libsodium';
          // Vendor chunk pour les grosses librairies
          if (id.includes('node_modules')) return 'vendor';
        },
      },
    },
  },

  // -------------------------------------------------
  // Optimisation des dépendances
  // -------------------------------------------------
  optimizeDeps: {
    // On pré‑bundle libsodium pour de meilleures perf en dev
    include: [
      'libsodium-wrappers',
      'libsodium-wrappers/dist/modules/libsodium-wrappers.js',
    ],
    exclude: [],
  },

  // -------------------------------------------------
  // Configuration du serveur de développement
  // -------------------------------------------------
  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: { strict: false },
    proxy: {
      // API REST
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      // WebSocket
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true,
        changeOrigin: true,
      },
    },
  },

  // -------------------------------------------------
  // Résolution des modules
  // -------------------------------------------------
  resolve: {
    alias: {
      // Alias pour libsodium qui fonctionne partout
      'libsodium-wrappers': 'libsodium-wrappers',
    },
    // Extensions supportées
    extensions: [
      '.mjs',
      '.js',
      '.ts',
      '.jsx',
      '.tsx',
      '.json',
      '.svelte',
    ],
  },

  // -------------------------------------------------
  // Variables globales (aucune URL publique ici)
  // -------------------------------------------------
  define: {
    'import.meta.vitest': 'undefined',
    global: 'globalThis',
  },

  // -------------------------------------------------
  // Configuration ESBuild
  // -------------------------------------------------
  esbuild: {
    legalComments: 'inline',
    target: 'es2020',
  },
});