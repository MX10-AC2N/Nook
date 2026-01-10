import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  
  // Assets statiques
  assetsInclude: ['**/*.svg', '**/*.png', '**/*.jpg', '**/*.jpeg', '**/*.ico'],
  
  build: {
    target: 'es2020',
    // SvelteKit gère déjà le chunking, on garde juste l'essentiel
    rollupOptions: {
      output: {
        manualChunks(id) {
          // Séparer libsodium dans son propre chunk pour le lazy loading
          if (id.includes('libsodium')) {
            return 'libsodium';
          }
          // Vendor chunk pour les grosses librairies
          if (id.includes('node_modules')) {
            return 'vendor';
          }
        }
      }
    }
  },
  
  // Optimisation des dépendances
  optimizeDeps: {
    // On inclut libsodium pour pré-bundling (meilleure perf en dev)
    include: [
      'libsodium-wrappers',
      'libsodium-wrappers/dist/modules/libsodium-wrappers.js'
    ],
    // Exclure les packages qui ont des problèmes avec le pré-bundling
    exclude: []
  },
  
  // Configuration du serveur de dev
  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: {
      strict: false
    },
    proxy: {
      // API REST
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      // WebSocket
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true,
        changeOrigin: true
      }
    }
  },
  
  // Résolution des modules
  resolve: {
    alias: {
      // Alias pour libsodium qui fonctionne partout
      'libsodium-wrappers': 'libsodium-wrappers'
    },
    // Extensions supportées
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.svelte']
  },
  
  // Variables globales
  define: {
    'import.meta.vitest': 'undefined',
    // Pour la compatibilité avec certaines libs
    global: 'globalThis'
  },
  
  // Configuration ESBuild
  esbuild: {
    legalComments: 'inline',
    target: 'es2020'
  }
});