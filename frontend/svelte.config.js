// svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // -----------------------------------------------------------------
  // 1️⃣ Preprocess – on garde vitePreprocess (TS, SCSS, PostCSS, …)
  // -----------------------------------------------------------------
  preprocess: vitePreprocess(),

  // -----------------------------------------------------------------
  // 2️⃣ Options du compilateur Svelte
  // -----------------------------------------------------------------
  compilerOptions: {
    // `runes` et `accessors` sont utiles si tu utilises des getters/setters
    // ou que tu veux que les propriétés soient accessibles depuis le JS.
    runes: true,
    accessors: true,
    // `immutable` à `false` signifie que Svelte ne suppose pas que tes props
    // sont immuables – c’est correct pour la plupart des apps.
    immutable: false
  },

  // -----------------------------------------------------------------
  // 3️⃣ Kit – configuration principale
  // -----------------------------------------------------------------
  kit: {
    // ---------- Adapter static ----------
    adapter: adapter({
      // dossiers où le build sera écrit
      pages: 'build',
      assets: 'build',
      // fallback utilisé pour le routing client‑side (SPA)
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),

    // ---------- Dossiers de sortie ----------
    // `appDir` garde la même structure que ton projet actuel.
    appDir: 'app',

    // ---------- Alias ----------
    // Les chemins relatifs sont résolus depuis la racine du repo.
    alias: {
      $lib: './src/lib',
      $components: './src/components',
      $routes: './src/routes',
      $assets: './static'
    },

    // ---------- Service worker ----------
    // Tu ne veux pas de SW auto‑register, donc on le désactive.
    serviceWorker: {
      register: false
    }

    // -----------------------------------------------------------------
    // NOTE : **`files.appTemplate` a été retiré** – SvelteKit utilise
    // automatiquement `src/app.html` s’il existe.
    // -----------------------------------------------------------------
  }
};

export default config;