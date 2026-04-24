// vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    sveltekit(),
  ],

  assetsInclude: [
    '**/*.svg',
    '**/*.png',
    '**/*.jpg',
    '**/*.jpeg',
    '**/*.ico',
  ],

  build: {
    target: 'es2020',
    // Seuil d'alerte relevé – les 4 chunks resteront sous ~600 kB
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          // libsodium — isolé (WASM lourd, uniquement sur /chat chiffré)
          if (id.includes('libsodium')) return 'libsodium';

          // chess.js / chessground — uniquement chargé sur /chess
          if (id.includes('chess.js') || id.includes('chessground')) return 'chess';

          // chart.js — uniquement chargé quand un graphique est affiché
          if (id.includes('chart.js')) return 'chart';

          // Svelte runtime — séparé pour maximiser le cache navigateur
          if (id.includes('node_modules/svelte') || id.includes('@sveltejs')) return 'svelte';

          // Reste de node_modules → vendor
          if (id.includes('node_modules')) return 'vendor';
        },
      },
    },
  },

  optimizeDeps: {
    // libsodium retiré de include — le dynamic import gère sa propre résolution.
    // L'inclure ici forçait un pré-bundle statique incompatible avec import().
    exclude: ['libsodium-wrappers'],
  },

  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: { strict: false },
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true,
        changeOrigin: true,
      },
    },
  },

  resolve: {
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.svelte'],
  },

  define: {
    'import.meta.vitest': 'undefined',
    global: 'globalThis',
  },

  esbuild: {
    legalComments: 'inline',
    target: 'es2020',
  },
});