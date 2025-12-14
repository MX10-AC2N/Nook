// frontend/src/lib/ui/ThemeStore.ts
import { writable } from 'svelte/store';

export type Theme = 'jardin' | 'space' | 'maison';
export const currentTheme = writable<Theme>('jardin');

// Sauvegarde dans localStorage
if (typeof window !== 'undefined') {
  const saved = localStorage.getItem('nook-theme') as Theme | null;
  if (saved) currentTheme.set(saved);
}

// Écoute les changements
currentTheme.subscribe((theme) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('nook-theme', theme);
    document.body.className = `theme-${theme}`;
  }
});