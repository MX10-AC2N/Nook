// eslint.config.js
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node
      }
    }
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: ['.svelte'],
        parser: ts.parser,
        svelteConfig
      }
    }
  },
  {
    // Ajout recommandé pour SvelteKit : ignorer les dossiers générés
    ignores: ['.svelte-kit/**', 'build/**', 'dist/**']
  },
  {
    rules: {
      // Ajoute ou surcharge des règles ici si besoin, par exemple :
      // 'svelte/no-unused-class-name': 'warn'
    }
  }
);