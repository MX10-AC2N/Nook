// src/lib/ui/ThemeStore.ts
import { writable, readonly, get } from 'svelte/store';

// ---------------------------------------------------------------------
// Types & constantes
// ---------------------------------------------------------------------
export type Theme = 'jardin' | 'space' | 'maison';

export interface ThemeInfo {
  id: Theme;
  name: string;
  description: string;
  icon: string;
}

/** Liste des thèmes disponibles (utilisée par le ThemeSwitcher) */
export const availableThemes: ThemeInfo[] = [
  {
    id: 'jardin',
    name: 'Jardin Secret',
    description: 'Un thème apaisant aux tons verts et naturels',
    icon: '🌿',
  },
  {
    id: 'space',
    name: 'Space Hub',
    description: 'Un thème sombre et moderne aux couleurs cosmiques',
    icon: '🌌',
  },
  {
    id: 'maison',
    name: 'Maison Chaleureuse',
    description: "Un thème chaleureux aux nuances d'ambre et de feu",
    icon: '🏠',
  },
];

// ---------------------------------------------------------------------
// Store du thème actuel (mutable – utilisé en interne)
// ---------------------------------------------------------------------
const _currentTheme = writable<Theme>('jardin');

/** Export en lecture‑seule pour les composants Svelte */
export const currentTheme = readonly(_currentTheme);

/** Alias pratique (si tu as besoin de la version mutable en interne) */
export const _currentThemeMutable = _currentTheme;

// ---------------------------------------------------------------------
// Clé de stockage local & thème par défaut
// ---------------------------------------------------------------------
const THEME_STORAGE_KEY = 'nook-theme';
const DEFAULT_THEME: Theme = 'jardin';

// ---------------------------------------------------------------------
// 1️⃣ Fonction d’application du thème (modifie le DOM & le store)
// ---------------------------------------------------------------------
function applyTheme(theme: Theme): void {
  if (typeof window === 'undefined') return;

  // 1️⃣ Supprimer les classes de thème précédentes
  document.body.classList.remove('theme-jardin', 'theme-space', 'theme-maison');

  // 2️⃣ Ajouter la classe du nouveau thème
  document.body.classList.add(`theme-${theme}`);

  // 3️⃣ Persister le choix dans le localStorage
  localStorage.setItem(THEME_STORAGE_KEY, theme);

  // 4️⃣ Mettre à jour le store Svelte
  _currentTheme.set(theme);
}

// ---------------------------------------------------------------------
// 2️⃣ Initialisation du thème au chargement du module (client‑only)
// ---------------------------------------------------------------------
function initTheme(): void {
  if (typeof window === 'undefined') return;

  // Récupérer le thème sauvegardé ou, à défaut, le thème système
  const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as Theme | null;
  const initialTheme = savedTheme ?? getSystemTheme();

  applyTheme(initialTheme);
}

// ---------------------------------------------------------------------
// 3️⃣ Exported helpers (public API)
// ---------------------------------------------------------------------

/** Change le thème et persiste le choix. */
export function setTheme(theme: Theme): void {
  applyTheme(theme);
}

/** Bascule cycliquement entre les thèmes disponibles. */
export function toggleTheme(): void {
  const current = get(_currentTheme);
  const currentIdx = availableThemes.findIndex((t) => t.id === current);
  const nextIdx = (currentIdx + 1) % availableThemes.length;
  const nextTheme = availableThemes[nextIdx].id;
  applyTheme(nextTheme);
}

/** Retourne le thème « système » (dark → space, light → jardin). */
export function getSystemTheme(): Theme {
  if (typeof window === 'undefined') return DEFAULT_THEME;

  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  return prefersDark ? 'space' : 'jardin';
}

/** Initialise un listener qui réagit aux changements du thème système.  
 *  Retourne une fonction de nettoyage à appeler si besoin. */
export function initSystemThemeListener(): () => void {
  if (typeof window === 'undefined') return () => {};

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

  const handler = (e: MediaQueryListEvent) => {
    // Si l'utilisateur a déjà choisi un thème, on ne le surcharge pas.
    const saved = localStorage.getItem(THEME_STORAGE_KEY);
    if (saved) return;

    applyTheme(e.matches ? 'space' : 'jardin');
  };

  mediaQuery.addEventListener('change', handler);

  // Fonction de nettoyage (utile en cas de hot‑module‑replacement)
  return () => {
    mediaQuery.removeEventListener('change', handler);
  };
}

// ---------------------------------------------------------------------
// 4️⃣ Initialisation immédiate (client‑only)
// ---------------------------------------------------------------------
if (typeof window !== 'undefined') {
  initTheme();

  // On garde le cleanup au cas où le module serait re‑chargé (HMR, SSR…)
  const cleanupSystemThemeListener = initSystemThemeListener();

  // Si tu utilises un bundler qui supporte le HMR, tu peux exposer le cleanup :
  // (exemple avec Vite) export const __hmr_cleanup = cleanupSystemThemeListener;
}