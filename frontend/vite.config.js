// frontend/vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  assetsInclude: ['**/*.svg', '**/*.png', '**/*.jpg', '**/*.jpeg', '**/*.ico'],
  build: {
    commonjsOptions: {
      include: [/libsodium/, /node_modules/],
      transformMixedEsModules: true
    },
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('libsodium-wrappers-sumo')) {
            return 'libsodium';
          }
        }
      }
    }
  },
  optimizeDeps: {
    // Suppression de l'exclude qui empêchait le pre-bundle
    // exclude: ['libsodium-wrappers-sumo'],

    // Ajout de l'include pour forcer Vite à pre-bundle ce package (fix le bug de résolution interne)
    include: ['libsodium-wrappers-sumo', 'svelte', 'svelte/internal', '@sveltejs/kit']
  },
  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: {
      strict: false,
      allow: ['..']
    },
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true
      }
    }
  },
  define: {
    'import.meta.vitest': 'undefined'
  }
});