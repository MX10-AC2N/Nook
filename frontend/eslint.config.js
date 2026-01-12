// eslint.config.js
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
  // -------------------------------------------------
  // 1️⃣ Règles JavaScript et TypeScript recommandées
  // -------------------------------------------------
  js.configs.recommended,
  ...ts.configs.recommended,

  // -------------------------------------------------
  // 2️⃣ Règles Svelte (flat config) – recommandé pour SvelteKit
  // -------------------------------------------------
  ...svelte.configs['flat/recommended'],

  // -------------------------------------------------
  // 3️⃣ Options globales (browser + node)
  // -------------------------------------------------
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },

  // -------------------------------------------------
  // 4️⃣ Traitement des fichiers *.svelte (et *.svelte.ts / *.svelte.js)
  // -------------------------------------------------
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parser: ts.parser,
      parserOptions: {
        // Utilise le service de projet TypeScript (pour le typage dans les .svelte)
        projectService: true,
        // Permet à ESLint de reconnaître les extensions .svelte
        extraFileExtensions: ['.svelte'],
        // Fournit la configuration Svelte à l’analyseur TypeScript
        svelteConfig,
      },
    },
  },

  // -------------------------------------------------
  // 5️⃣ Ignorer les dossiers générés (SvelteKit, build, dist)
  // -------------------------------------------------
  {
    ignores: ['.svelte-kit/**', 'build/**', 'dist/**'],
  },

  // -------------------------------------------------
  // 6️⃣ Règles additionnelles / overrides (personnalisez ici)
  // -------------------------------------------------
  {
    rules: {
      // Exemple : avertir sur les classes CSS inutilisées dans les composants Svelte
      // 'svelte/no-unused-class-name': 'warn',

      // Vous pouvez ajouter ou surcharger d’autres règles ESLint/TypeScript ici.
    },
  }
);