import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import routify from '@roxi/routify/vite-plugin';

export default defineConfig({
  plugins: [
    routify({
      // Configuration Routify adaptée
      routesDir: 'src/routes',
      layoutsDir: 'src/routes',
      pagesDir: 'src/routes',
      extensions: ['.svelte', '.js', '.ts'],
      dynamicImports: true,
      ssr: false,
      routifyDir: '.routify',
      ignore: [
        '**/+*.*',                      // Ignorer fichiers +layout.svelte, +page.svelte
        '**/*.bak',                     // Fichiers backup
        '**/*.d.ts',                    // Fichiers de type
        '**/*.md',                     // Fichiers Markdown
        '**/app.html'                   // Fichier app.html simplifié
      ],
      dynamic: true,
      autoFileRoutes: false,
      base: '/',
      trailingSlash: 'never'
    }),
    svelte({
      compilerOptions: {
        runes: true,
        accessors: true,
        immutable: true
      },
      onwarn: (warning, handler) => {
        // Ignorer warnings spécifiques
        const ignoredWarnings = [
          'a11y-missing-attribute',
          'css-unused-selector',
          'Svelte 5',
          'unused-css-selector'
        ];
        
        if (ignoredWarnings.some(code => warning.code?.includes(code) || warning.message.includes(code))) {
          return;
        }
        
        // Ignorer warnings sur les events custom
        if (warning.message.includes('customElement')) {
          return;
        }
        
        handler(warning);
      }
    })
  ],
  build: {
    target: 'esnext',
    outDir: 'build',
    assetsDir: '_app',
    rollupOptions: {
      input: {
        main: './index.html'  // Point d'entrée principal
      },
      output: {
        manualChunks: (id) => {
          if (id.includes('node_modules')) {
            if (id.includes('libsodium')) return 'libsodium';
            if (id.includes('chart.js')) return 'chart';
            if (id.includes('simple-peer')) return 'peer';
            if (id.includes('@roxi/routify')) return 'routify';
            if (id.includes('svelte')) return 'svelte';
            return 'vendor';
          }
        }
      }
    }
  },
  server: {
    port: 5173,
    strictPort: true,
    host: true,
    fs: {
      strict: false,  // Désactiver la restriction stricte pour éviter les problèmes d'accès
      allow: [
        '..',
        './src',
        './node_modules'
      ]
    },
    watch: {
      ignored: [
        '**/node_modules/**',
        '**/.git/**',
        '**/.routify/**',
        '**/*.bak'
      ]
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
      '$routes': './src/routes',
      '$assets': '/static'
    }
  },
  optimizeDeps: {
    include: [
      'svelte',
      'svelte/internal',
      'svelte/store',
      'svelte/animate',
      'svelte/easing',
      'svelte/motion',
      'svelte/transition',
      'svelte/parse',
      'svelte/compiler',
      'uuid',
      'libsodium-wrappers',
      'simple-peer',
      '@roxi/routify',
      'chart.js'
    ],
    exclude: [
      'vite',
      'vitest'
    ]
  },
  assetsInclude: [
    '**/*.png',
    '**/*.jpg',
    '**/*.jpeg',
    '**/*.gif',
    '**/*.svg',
    '**/*.webp',
    '**/*.ico',
    '**/*.woff',
    '**/*.woff2',
    '**/*.ttf',
    '**/*.eot',
    '**/*.otf',
    '**/*.map'
  ]
});