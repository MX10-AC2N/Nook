import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: true,
    host: true
  },
  ssr: {
    noExternal: [
      'tsparticles',
      '@tsparticles/slim',
      '@tsparticles/engine',
      '@tsparticles/svelte',
      '@tsparticles/shape-star' // Au cas où, pour le shape star que tu utilises
    ]
    // Cette option force Vite à bundler ces libs client-only au lieu de les externaliser pendant le SSR
  }
});