# Svelte 5 - Ressources pour Interface Graphique

Ce fichier recense les ressources utiles pour l'optimisation et le développement d'interfaces graphiques en Svelte 5.

## Composants UI (Headless)

**[shadcn-svelte](https://github.com/shadcn-ui/svelte)** - Composants UI headless basés sur shadcn-ui

| Aspect | Détails |
|--------|---------|
| **Style** | Non stylisés, à personnaliser avec Tailwind CSS |
| **Accessibilité** | ARIA compliant |
| **Composants** | Dialog, Dropdown, Select, Tabs, etc. |
| **Intégration** | Svelte 5 natif |

**[bits-ui](https://github.com/huntabyte/bits-ui)** - Primitives de composants headless pour Svelte 5

| Aspect | Détails |
|--------|---------|
| **Type-safe** | Entièrement typé avec TypeScript |
| **Léger** | Bundle minimal |
| **Purpose** | Construction de composants personnalisés |
| **Popularité** | Recommandé par la communauté Svelte |

**[Svelte Headless UI](https://github.com/sveltejs/headlessui)** - Composants accessibes et non stylisés

| Aspect | Détails |
|--------|---------|
| **Maintenance** | Équipe officielle Svelte |
| **Focus** | Accessibilité et UX |
| **Composants** | Menu, Listbox, Combobox, Dialog, etc. |

**[Carbon Components Svelte](https://github.com/carbon-design-system/carbon-components-svelte)** - Design system IBM Carbon

| Aspect | Détails |
|--------|---------|
| **Design system** | Complet et cohérent |
| **Thèmes** | Multiple (White, Gray 10, etc.) |
| **Composants** | 40+ composants |
| **Documentation** | Extensive |

## Bibliothèques d'Animation

**[svelte-motion](https://github.com/Psychopatoecci/svelte-motion)** - Animation library inspirée de Framer Motion

| Aspect | Détails |
|--------|---------|
| **API** | Actions Svelte (use:motion) |
| **Variants** | Presets d'animation réutilisables |
| **Gestures** | Support des gestes tactiles |
| **Lifecycle** | onUpdate, onAnimationStart, onAnimationComplete |

**Exemple d'utilisation :**
```svelte
<script>
  import { Motion } from 'svelte-motion';

  let isVisible = $state(false);
</script>

<Motion let:motion animate={{ x: isVisible ? 100 : 0, opacity: 1 }}>
  <div use:motion>Contenu animé</div>
</Motion>
