import { writable } from 'svelte/store';

export type Theme = 'jardin' | 'space' | 'maison';

export interface ThemeInfo {
  id: Theme;
  name: string;
  description: string;
  icon: string;
}

export const availableThemes: ThemeInfo[] = [
  {
    id: 'jardin',
    name: 'Jardin Secret',
    description: 'Un thème apaisant aux tons verts et naturels',
    icon: '🌿'
  },
  {
    id: 'space',
    name: 'Space Hub',
    description: 'Un thème sombre et moderne aux couleurs cosmiques',
    icon: '🌌'
  },
  {
    id: 'maison',
    name: 'Maison Chaleureuse',
    description: 'Un thème chaleureux aux nuances d\'ambre et de feu',
    icon: '🏠'
  }
];

// Store du thème actuel
export const currentTheme = writable<Theme>('jardin');

// Clé de stockage
const THEME_STORAGE_KEY = 'nook-theme';

// Thème par défaut
const DEFAULT_THEME: Theme = 'jardin';

// Initialisation
function initTheme(): void {
  if (typeof window === 'undefined') return;
  
  // Récupérer le thème sauvegardé ou utiliser le thème système
  const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as Theme | null;
  const initialTheme = savedTheme || DEFAULT_THEME;
  
  currentTheme.set(initialTheme);
  applyTheme(initialTheme);
}

// Appliquer un thème
function applyTheme(theme: Theme): void {
  if (typeof window === 'undefined') return;
  
  // Supprimer tous les thèmes existants
  document.body.classList.remove('theme-jardin', 'theme-space', 'theme-maison');
  
  // Appliquer le nouveau thème
  document.body.classList.add(`theme-${theme}`);
  
  // Sauvegarder dans localStorage
  localStorage.setItem(THEME_STORAGE_KEY, theme);
  
  // Sauvegarder dans le store
  currentTheme.set(theme);
}

// Fonction pour changer de thème
export function setTheme(theme: Theme): void {
  applyTheme(theme);
}

// Fonction pour basculer entre les thèmes
export function toggleTheme(): void {
  let current: Theme | undefined;
  
  const unsubscribe = currentTheme.subscribe(value => {
    current = value;
  });
  
  unsubscribe();
  
  if (!current) return;
  
  const currentIndex = availableThemes.findIndex(t => t.id === current);
  const nextIndex = (currentIndex + 1) % availableThemes.length;
  const nextTheme = availableThemes[nextIndex].id;
  
  applyTheme(nextTheme);
}

// Fonction pour obtenir le thème système
export function getSystemTheme(): Theme {
  if (typeof window === 'undefined') return DEFAULT_THEME;
  
  // Vérifier prefers-color-scheme
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  return prefersDark ? 'space' : 'jardin';
}

// Suivre les changements du thème système
export function initSystemThemeListener(): void {
  if (typeof window === 'undefined') return;
  
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  const handleChange = (e: MediaQueryListEvent): void => {
    const savedTheme = localStorage.getItem(THEME_STORAGE_KEY);
    
    // Ne changer automatiquement que si aucun thème n'a été sauvegardé
    if (!savedTheme) {
      applyTheme(e.matches ? 'space' : 'jardin');
    }
  };
  
  mediaQuery.addEventListener('change', handleChange);
  
  // Retourner une fonction pour nettoyer
  return () => {
    mediaQuery.removeEventListener('change', handleChange);
  };
}

// Initialiser au chargement du module
initTheme();
initSystemThemeListener();
