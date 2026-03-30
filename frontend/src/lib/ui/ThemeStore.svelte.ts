// src/lib/ui/ThemeStore.svelte.ts (Svelte 5 avec runes)
import { browser } from '$app/environment';

// ---------------------------------------------------------------------
// Types & constantes
// ---------------------------------------------------------------------
export type Theme = 'jardin-secret' | 'space-hub' | 'maison-chaleureuse' | 'nuit-douce';

export interface ThemeInfo {
  id: Theme;
  name: string;
  description: string;
  icon: string;
}

/** Liste des thèmes disponibles (utilisée par le ThemeSwitcher) */
export const availableThemes: ThemeInfo[] = [
  {
    id: 'jardin-secret',
    name: 'Jardin Secret',
    description: 'Un thème apaisant aux tons verts et naturels',
    icon: '🌿',
  },
  {
    id: 'space-hub',
    name: 'Space Hub',
    description: 'Un thème sombre et moderne aux couleurs cosmiques',
    icon: '🌌',
  },
  {
    id: 'maison-chaleureuse',
    name: 'Maison Chaleureuse',
    description: "Un thème chaleureux aux nuances d'ambre et de feu",
    icon: '🏠',
  },
  {
    id: 'nuit-douce',
    name: 'Nuit Douce',
    description: 'Mode sombre doux — repose les yeux le soir',
    icon: '🌙',
  },
];

// ---------------------------------------------------------------------
// État réactif du thème (Svelte 5 runes)
// ---------------------------------------------------------------------
/** Thème actuel - état réactif Svelte 5 */
let currentTheme = $state<Theme>('jardin-secret');

// ---------------------------------------------------------------------
// Clé de stockage local & thème par défaut
// ---------------------------------------------------------------------
const THEME_STORAGE_KEY = 'nook-theme';
const DEFAULT_THEME: Theme = 'jardin-secret';

// ---------------------------------------------------------------------
// 1️⃣ Fonction d'application du thème (modifie le DOM & l'état)
// ---------------------------------------------------------------------
function applyTheme(theme: Theme): void {
  if (!browser) return;

  // 1️⃣ Supprimer les classes de thème précédentes
  document.body.classList.remove('theme-jardin-secret', 'theme-space-hub', 'theme-maison-chaleureuse', 'theme-nuit-douce');

  // 2️⃣ Ajouter la classe du nouveau thème
  document.body.classList.add(`theme-${theme}`);

  // 3️⃣ Persister le choix dans le localStorage
  localStorage.setItem(THEME_STORAGE_KEY, theme);

  // 4️⃣ Mettre à jour l'état réactif (Svelte 5)
  currentTheme = theme;
}

// ---------------------------------------------------------------------
// 2️⃣ Initialisation du thème au chargement (client-only)
// ---------------------------------------------------------------------
function initTheme(): void {
  if (!browser) return;

  // Récupérer le thème sauvegardé ou, à défaut, le thème système
  const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as Theme | null;
  const initialTheme = savedTheme ?? getSystemTheme();

  applyTheme(initialTheme);
}

// ---------------------------------------------------------------------
// 3️⃣ API publique (exportée)
// ---------------------------------------------------------------------

/** Change le thème et persiste le choix. */
export function setTheme(theme: Theme): void {
  applyTheme(theme);
}

/** Bascule cycliquement entre les thèmes disponibles. */
export function toggleTheme(): void {
  const currentIdx = availableThemes.findIndex((t) => t.id === currentTheme);
  const nextIdx = (currentIdx + 1) % availableThemes.length;
  const nextTheme = availableThemes[nextIdx].id;
  applyTheme(nextTheme);
}

/** Retourne le thème « système » (dark → space, light → jardin). */
export function getSystemTheme(): Theme {
  if (!browser) return DEFAULT_THEME;

  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  return prefersDark ? 'nuit-douce' : 'jardin-secret';
}

/** 
 * Initialise un listener qui réagit aux changements du thème système.  
 * Retourne une fonction de nettoyage à appeler si besoin. 
 */
export function initSystemThemeListener(): () => void {
  if (!browser) return () => {};

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

  const handler = (e: MediaQueryListEvent) => {
    // Si l'utilisateur a déjà choisi un thème, on ne le surcharge pas.
    const saved = localStorage.getItem(THEME_STORAGE_KEY);
    if (saved) return;

    applyTheme(e.matches ? 'space-hub' : 'jardin-secret');
  };

  mediaQuery.addEventListener('change', handler);

  // Fonction de nettoyage
  return () => {
    mediaQuery.removeEventListener('change', handler);
  };
}

// ---------------------------------------------------------------------
// 4️⃣ Export de l'état réactif (CORRIGÉ)
// ---------------------------------------------------------------------
/** 
 * Thème actuel - en lecture seule pour les composants.
 * Usage dans un composant : `import { getCurrentTheme } from '$lib/ui/ThemeStore'`
 * Puis dans le template : `{getCurrentTheme()}` ou `{#if getCurrentTheme() === 'jardin-secret'}`
 */
export function getCurrentTheme(): Theme {
  return currentTheme;
}

// ---------------------------------------------------------------------
// 5️⃣ Initialisation immédiate (client-only)
// ---------------------------------------------------------------------
if (browser) {
  initTheme();

  // On garde le cleanup au cas où le module serait re-chargé (HMR, SSR…)
  const cleanupSystemThemeListener = initSystemThemeListener();

  // Si tu utilises un bundler qui supporte le HMR, tu peux exposer le cleanup :
  // (exemple avec Vite) export const __hmr_cleanup = cleanupSystemThemeListener;
}
