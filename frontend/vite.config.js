// vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import compression from 'vite-plugin-compression';

export default defineConfig({
  plugins: [
    sveltekit(),
    // Compression Brotli + Gzip pour les assets statiques
    compression({ algorithm: 'gzip', ext: '.gz', threshold: 10240 }),
    compression({ algorithm: 'brotliCompress', ext: '.br', threshold: 10240 }),
  ],

  // Filter Svelte a11y warnings that break the build
  onwarn(warning, warn) {
    if (warning.code === 'a11y_click_events_have_key_events') return;
    if (warning.code === 'a11y_no_static_element_interactions') return;
    if (warning.code === 'a11y_label_has_associated_control') return;
    if (warning.code === 'a11y_interactive_supports_focus') return;
    if (warning.code === 'css_unused_selector') return;
    if (warning.code === 'a11y_media_has_caption') return;
    warn(warning);
  },

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