// frontend/vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  assetsInclude: ['**/*.svg', '**/*.png', '**/*.jpg', '**/*.jpeg', '**/*.ico'],
  resolve: {
    alias: {
      // Fix définitif pour le bug de résolution interne ESM de libsodium-wrappers
      // Pointe vers le bundle browser combiné (stable, pas de imports cassés)
      'libsodium-wrappers': 'libsodium-wrappers/dist/browsers/combined/libsodium.js'
    }
  },
  optimizeDeps: {
    include: ['libsodium-wrappers']
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