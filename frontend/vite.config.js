import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import routifyPlugin from '@roxi/routify/vite-plugin'; // Import par défaut

export default defineConfig({
  plugins: [
    routifyPlugin({ // Utiliser le plugin directement
      // Configuration Routify
      routesDir: 'src/routes',
      layoutsDir: 'src/layouts',
      useHash: false,
      singleFetch: true,
      extensions: ['.svelte', '.js', '.ts'],
      dynamicImports: true,
      ssr: false,
      // Options spécifiques Routify v3
      routifyDir: '.routify',
      ignore: ['**/+page.svelte', '**/+layout.svelte', '**/+error.svelte'],
      dynamic: true,
      // Désactiver le routage automatique des fichiers +*
      autoFileRoutes: false
    }),
    svelte({
      compilerOptions: {
        runes: true,
        accessors: true,
        immutable: true
      },
      onwarn: (warning, handler) => {
        if (
          warning.code === 'a11y-missing-attribute' || 
          warning.code === 'css-unused-selector' ||
          warning.message.includes('Svelte 5')
        ) {
          return;
        }
        handler(warning);
      }
    })
  ],
  server: {
    port: 5173,
    strictPort: true,
    host: true,
    fs: {
      strict: true,
      allow: ['..']
    }
  },
  build: {
    target: 'esnext',
    outDir: 'build',
    assetsDir: '_app',
    rollupOptions: {
      output: {
        manualChunks: {
          'libsodium': ['libsodium-wrappers'],
          'chart': ['chart.js'],
          'peer': ['simple-peer']
        }
      }
    }
  },
  preview: {
    port: 4173,
    strictPort: true
  },
  resolve: {
    alias: {
      '$lib': '/src/lib',
      '$routes': '/src/routes',
      '$components': '/src/components'
    }
  }
});