import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.{js,ts,svelte}']
    }
  },
  resolve: {
    alias: {
      '$lib': path.resolve(__dirname, 'src/lib'),
      '$app': path.resolve(__dirname, 'node_modules/@sveltejs/kit/src/app')
    }
  }
});
