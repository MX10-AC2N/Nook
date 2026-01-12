// vite.config.js
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import replace from '@rollup/plugin-replace';   // <‑‑ AJOUT

export default defineConfig({
  plugins: [
    sveltekit(),
    // -------------------------------------------------
    // 1️⃣ Remplacement du placeholder dans app.html
    // -------------------------------------------------
    replace({
      // empêche que le plugin remplace des parties du code qui
      // ressemblent à notre token par accident
      preventAssignment: true,
      // le token que l’on mettra dans src/app.html
      values: {
        // si PUBLIC_SITE_URL n’est pas définie, on met une chaîne vide
        '%PUBLIC_SITE_URL%': JSON.stringify(process.env.PUBLIC_SITE_URL || '')
      }
    })
  ],

  // -------------------------------------------------
  // 2️⃣ Tes réglages existants (chunks, alias, proxy, …)
  // -------------------------------------------------
  assetsInclude: ['**/*.svg', '**/*.png', '**/*.jpg', '**/*.jpeg', '**/*.ico'],

  build: {
    target: 'es2020',
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('libsodium')) return 'libsodium';
          if (id.includes('node_modules')) return 'vendor';
        }
      }
    }
  },

  optimizeDeps: {
    include: [
      'libsodium-wrappers',
      'libsodium-wrappers/dist/modules/libsodium-wrappers.js'
    ],
    exclude: []
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
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true,
        changeOrigin: true
      }
    }
  },

  resolve: {
    alias: {
      'libsodium-wrappers': 'libsodium-wrappers'
    },
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.svelte']
  },

  define: {
    'import.meta.vitest': 'undefined',
    global: 'globalThis'
  },

  esbuild: {
    legalComments: 'inline',
    target: 'es2020'
  }
});