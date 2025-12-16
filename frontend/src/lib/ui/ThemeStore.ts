import { writable } from 'svelte/store';

export type Theme = 'jardin' | 'space' | 'maison';
export const currentTheme = writable<Theme>('jardin');

if (typeof window !== 'undefined') {
  const saved = localStorage.getItem('nook-theme') as Theme | null;
  if (saved) currentTheme.set(saved);
}

currentTheme.subscribe(theme => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('nook-theme', theme);
    document.body.className = `theme-${theme}`;
  }
});