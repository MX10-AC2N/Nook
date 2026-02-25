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

          // Svelte runtime — séparé pour maximiser le cache navigateur
          if (id.includes('node_modules/svelte') || id.includes('@sveltejs')) return 'svelte';

          // Reste de node_modules → vendor
          if (id.includes('node_modules')) return 'vendor';
        },
      },
    },
  },

  optimizeDeps: {
    include: [
      'libsodium-wrappers',
      'libsodium-wrappers/dist/modules/libsodium-wrappers.js',
    ],
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
    alias: {
      'libsodium-wrappers': 'libsodium-wrappers',
    },
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