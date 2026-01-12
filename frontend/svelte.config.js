import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: true,
    accessors: true,
    immutable: false
  },
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),
    appDir: 'app',
    files: {
      appTemplate: 'src/app.html'
    },
    alias: {
      $lib: './src/lib',
      $components: './src/components',
      $routes: './src/routes',
      $assets: './static'
    },
    serviceWorker: {
      register: false
    }
  }
};

export default config;