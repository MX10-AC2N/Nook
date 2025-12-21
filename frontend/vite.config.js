import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import routify from '@roxi/routify/vite-plugin';

export default defineConfig({
  plugins: [
    routify({
      // Configuration adaptée à ta structure
      routesDir: 'src/routes',           // Dossier des routes
      layoutsDir: 'src/routes',           // Les layouts sont dans le même dossier que les routes
      pagesDir: 'src/routes',             // Dossier des pages
      useHash: false,
      singleFetch: true,
      extensions: ['.svelte', '.js', '.ts'],
      dynamicImports: true,
      ssr: false,
      // Options Routify v3 spécifiques
      routifyDir: '.routify',
      ignore: [
        '**/+*.*',                      // Ignorer les fichiers +layout.svelte, +page.svelte, etc.
        '**/*.bak',                     // Ignorer les fichiers backup
        '**/*.d.ts',                    // Ignorer les fichiers de type
        '**/*.ts',                      // Ignorer les fichiers TypeScript purs
        '**/*.js'                       // Ignorer les fichiers JavaScript purs
      ],
      dynamic: true,
      autoFileRoutes: false,
      // Spécifier explicitement le point d'entrée
      entry: 'src/app.html'
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
  // Spécifier explicitement le fichier d'entrée
  build: {
    target: 'esnext',
    outDir: 'build',
    assetsDir: '_app',
    rollupOptions: {
      input: {
        main: './src/app.html'  // Point d'entrée principal
      },
      output: {
        manualChunks: {
          'libsodium': ['libsodium-wrappers'],
          'chart': ['chart.js'],
          'peer': ['simple-peer'],
          'routify': ['@roxi/routify']
        }
      }
    }
  },
  server: {
    port: 5173,
    strictPort: true,
    host: true,
    fs: {
      strict: true,
      allow: ['..', './src']
    }
  },
  preview: {
    port: 4173,
    strictPort: true
  },
  resolve: {
    alias: {
      '$lib': './src/lib',
      '$components': './src/components',
      '$routes': './src/routes'
    }
  },
  // Configuration spécifique pour Svelte 5
  optimizeDeps: {
    include: [
      'svelte',
      'svelte/internal',
      'svelte/elements',
      'svelte/store',
      'svelte/animate',
      'svelte/easing',
      'svelte/motion',
      'svelte/transition',
      'svelte/parse',
      'svelte/compiler'
    ]
  }
});