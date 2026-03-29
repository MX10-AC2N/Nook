# Svelte UI Libraries - Ressources & Knowledge Base

**Source** : Article « 10 UI Libraries for Svelte to Try in 2024 » par Olga Tash  
**URL** : https://dev.to/olga_tash/10-ui-libraries-for-svelte-to-try-in-2024-1692  
**Date de mise à jour de l’article** : mars 2026 (titre mis à jour en « 2026 »)

## Introduction

Svelte est un framework UI basé sur un compilateur qui transforme des composants déclarés en HTML, CSS et JavaScript en JavaScript vanille efficace avec un runtime minimal dans le navigateur.

Svelte gagne en popularité grâce à ses performances rapides, sa légèreté et sa facilité d’apprentissage. Il est utilisé par IKEA, Spotify, Apple Music & TV, Yahoo Finance et StackOverflow.

Cet article recense **10 bibliothèques UI Svelte les plus complètes et utiles** pour accélérer le développement d’applications.

## Liste des 10 bibliothèques UI Svelte

### 1. Flowbite Svelte
- **Site officiel** : [flowbite-svelte.com](https://flowbite-svelte.com/)
- **GitHub** : [themesberg/flowbite-svelte](https://github.com/themesberg/flowbite-svelte) (2.7k ⭐)
- **Description** : Bibliothèque UI open-source composée de composants Svelte natifs + Tailwind CSS. Plus de 63 composants UI et des centaines d’éléments interactifs basés sur Flowbite.
- **Points forts** :
  - Data table avec tri, filtrage et pagination
  - Documentation très bien organisée (exemples + previews responsives)
  - Support light/dark + RTL/LTR
- **Usage recommandé** : Projets Tailwind qui veulent des composants prêts à l’emploi et élégants.

### 2. Skeleton
- **Site officiel** : [skeleton.dev](https://www.skeleton.dev/)
- **GitHub** : [skeletonlabs/skeleton](https://github.com/skeletonlabs/skeleton) (5.9k ⭐)
- **Description** : Système de design + bibliothèque de composants construite sur Tailwind CSS pour des interfaces scalables et cohérentes (Svelte + React).
- **Points forts** :
  - Thèmes, éléments stylés et composants pré-construits
  - Intégrations avec Bits UI, Melt UI, Radix
  - Construit sur Zag.js
- **Usage recommandé** : Applications qui ont besoin d’un design system cohérent.

### 3. SVAR Svelte Core
- **Site officiel** : [svar.dev/svelte/core](https://svar.dev/svelte/core/)
- **GitHub** : [svar-widgets/core](https://github.com/svar-widgets/core) (247 ⭐)
- **Description** : Sélection de contrôles de formulaires, popups, date pickers, menus, etc. pour créer des interfaces d’applications élégantes.
- **Points forts** :
  - Composants enterprise : data grid, Gantt chart, file manager
  - Thèmes clair/sombre + personnalisation CSS pure (pas de Tailwind)
- **Usage recommandé** : Applications enterprise-grade qui ne veulent pas dépendre de Tailwind.

### 4. Carbon Components Svelte
- **Site officiel** : [svelte.carbondesignsystem.com](https://svelte.carbondesignsystem.com/)
- **GitHub** : [carbon-design-system/carbon-components-svelte](https://github.com/carbon-design-system/carbon-components-svelte) (2.9k ⭐)
- **Description** : Bibliothèque basée sur le Carbon Design System d’IBM.
- **Points forts** : Très grand nombre de composants avec fonctionnalités avancées + style minimaliste (nuances de gris).
- **Usage recommandé** : Applications professionnelles / business / enterprise.

### 5. Svelte UX
- **Site officiel** : [svelte-ux.techniq.dev](https://svelte-ux.techniq.dev/)
- **GitHub** : [techniq/svelte-ux](https://github.com/techniq/svelte-ux) (1.1k ⭐)
- **Description** : Bibliothèque complète construite sur Tailwind CSS avec une personnalisation poussée (thèmes, variants, styling flexible).
- **Points forts** : Intégration native avec LayerChart (visualisation de données).
- **Usage recommandé** : Applications data-heavy qui veulent beaucoup de customisation + charts.

### 6. Kampsy-ui
- **GitHub** : [kampsy/ui](https://github.com/kampsy/ui) (260 ⭐)
- **Description** : Plus de 30 composants UI Svelte interactifs intégrables avec Tailwind CSS. Inspiré du design system Geist de Vercel.
- **Points forts** : Design moderne et cohérent.
- **Usage recommandé** : Projets qui veulent un look “Vercel-like” sans trop d’efforts.

### 7. Melt UI
- **Site officiel** : [melt-ui.com](https://melt-ui.com/)
- **GitHub** : [melt-ui/melt-ui](https://github.com/melt-ui/melt-ui) (2.9k ⭐)
- **Description** : Bibliothèque **headless** la plus puissante et complète pour Svelte (composants accessibles et hautement personnalisables).
- **Points forts** :
  - Builders de composants très flexibles
  - Contrôle total du styling (CSS, Tailwind, CSS-in-JS…)
  - Documentation excellente avec démos
- **Usage recommandé** : Tout projet qui veut un maximum de contrôle et d’accessibilité.

### 8. Bits UI
- **Site officiel** : [bits-ui.com](https://www.bits-ui.com/docs/introduction)
- **GitHub** : [huntabyte/bits-ui](https://github.com/huntabyte/bits-ui) (3.1k ⭐)
- **Description** : Primitives de composants headless construites sur Melt UI.
- **Points forts** : Complètement unstyled, accessibles, styling flexible (Tailwind, UnoCSS, etc.).
- **Usage recommandé** : Construction de design systems sur des bases solides.

### 9. shadcn-svelte
- **Site officiel** : [shadcn-svelte.com](https://www.shadcn-svelte.com/)
- **GitHub** : [huntabyte/shadcn-svelte](https://github.com/huntabyte/shadcn-svelte) (8.4k ⭐)
- **Description** : Port Svelte officiel de shadcn/ui (React). Composants réutilisables à copier-coller ou via CLI (aucune dépendance lourde).
- **Points forts** :
  - Basé sur Bits UI + styles Tailwind par défaut
  - Theme builder + kit Figma
  - Tu possèdes entièrement le code
- **Usage recommandé** : Projets qui veulent des composants de qualité tout en gardant un contrôle total.

### 10. DaisyUI
- **Site officiel** : [daisyui.com](https://daisyui.com/)
- **GitHub** : [saadeghi/daisyui](https://github.com/saadeghi/daisyui) (40k ⭐)
- **Description** : Plugin Tailwind CSS qui fournit 65+ composants UI sous forme de classes CSS de haut niveau.
- **Points forts** : Intégration parfaite avec SvelteKit et tout projet Tailwind.
- **Usage recommandé** : Styling ultra-rapide avec très peu de CSS personnalisé.

## Recommandations rapides (pour Claude)

- **Tu veux du style prêt à l’emploi + Tailwind** → Flowbite Svelte, Skeleton, DaisyUI, Kampsy-ui, Svelte UX
- **Tu veux du headless / contrôle total** → Melt UI → Bits UI → shadcn-svelte
- **Tu fais une app enterprise / business** → SVAR Svelte Core ou Carbon Components
- **Tu veux un design system cohérent** → Skeleton ou shadcn-svelte

**Note** : La majorité des bibliothèques s’intègrent très bien avec Tailwind CSS (sauf SVAR et Carbon qui ont leur propre styling).

---
