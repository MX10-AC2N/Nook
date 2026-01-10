import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  assetsInclude: ['**/*.svg', '**/*.png', '**/*.jpg', '**/*.jpeg', '**/*.ico'],
  build: {
    commonjsOptions: {
      include: [/libsodium/, /node_modules/],
      transformMixedEsModules: true,
      ignoreDynamicRequires: true
    },
    rollupOptions: {
      external: [],
      output: {
        manualChunks(id) {
          if (id.includes('node_modules/libsodium')) {
            return 'libsodium';
          }
          if (id.includes('node_modules')) {
            return 'vendor';
          }
        }
      }
    },
    target: 'es2020'
  },
  optimizeDeps: {
    exclude: ['libsodium-wrappers'],
    include: ['svelte', 'svelte/internal', '@sveltejs/kit']
  },
  server: {
    port: 5173,
    strictPort: false,
    host: true,
    fs: {
      strict: false,
      allow: ['..', '.']
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
  resolve: {
    extensions: ['.mjs', '.js', '.mts', '.ts', '.jsx', '.tsx', '.json'],
    alias: {
      // Chemin correct pour libsodium dans GitHub Actions
      'libsodium-wrappers': 'libsodium-wrappers/dist/modules/index.js'
    }
  },
  define: {
    'import.meta.vitest': 'undefined',
    global: 'globalThis'
  },
  esbuild: {
    legalComments: 'inline'
  }
});