import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import routify from '@roxi/routify/vite-plugin'; // Syntaxe corrigée pour Routify v3

export default defineConfig({
  plugins: [
    routify({
      // Configuration Routify pour Svelte 5
      routesDir: 'src/routes',
      layoutsDir: 'src/layouts',
      useHash: false,
      singleFetch: true,
      extensions: ['.svelte', '.js', '.ts'],
      dynamicImports: true,
      // Désactiver le routage côté serveur pour le moment
      ssr: false,
      // Options spécifiques à Routify v3
      routifyDir: '.routify',
      ignore: ['**/+*'],
      dynamic: true
    }),
    svelte({
      compilerOptions: {
        // Activer le mode Svelte 5 (runes)
        runes: true,
        // Désactiver l'accessibilité warnings pour le développement
        // (à réactiver en production si nécessaire)
        accessors: true,
        immutable: true
      },
      onwarn: (warning, handler) => {
        // Ignorer certains warnings spécifiques à Routify
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
          // Séparer les grosses dépendances
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