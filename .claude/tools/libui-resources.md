# Svelte UI Libraries - Knowledge Base (Enterprise & General)

**Sources** :
- Article dev.to : « 10 UI Libraries for Svelte to Try in 2024 » (Olga Tash)
- Article Medium : « Best 10+ Svelte UI Components & Libraries for Building Enterprise Apps » (Olga Tashlikovich, juin 2025)

**Objectif** : Base de connaissance pour projets Svelte / SvelteKit (général + enterprise-grade).

## 1. Bibliothèques générales / UI classiques (dev.to + Medium)

### Flowbite Svelte
- Site : https://flowbite-svelte.com/
- 60+ composants Tailwind natifs, dark mode, data table complet.

### Skeleton
- Site : https://www.skeleton.dev/
- Design system complet + Figma kit, support React & Svelte.

### Bits UI
- Site : https://bits-ui.com/
- Primitives headless (basé sur Melt UI), Svelte 5, très accessible.

### shadcn-svelte
- Site : https://www.shadcn-svelte.com/
- Port officiel de shadcn/ui, CLI, full control, Svelte 5 + Tailwind v4 + LayerChart intégré.

### SVAR Svelte UI Components
- Site : https://svar.dev/svelte/
- Très orienté enterprise : DataGrid avec virtual scrolling, Gantt, Filter Builder.

## 2. Bibliothèques Enterprise / Data-heavy (nouveautés Medium 2025)

### Tzezar Svelte DataGrid
- Site : https://datagrid.tzezar.pl/
- Tableau headless ultra-performant (virtualisation avec svelte-virtuallists), tri, filtre, grouping, TypeScript first.

### vkurko/calendar (EventCalendar)
- GitHub : https://github.com/vkurko/calendar
- Calendrier complet multi-vues (day/week/month/resource/timeline), léger, thèmes clair/sombre.

### svelte-gantt
- Demo : https://anovokmet.github.io/svelte-gantt/
- Gantt chart haute performance, drag & drop, zoom, dépendances, tree view.

### LayerChart
- Site : https://www.layerchart.com/
- Suite de composants de visualisation (Bar, Area, Scatter, Pie, Treemap, etc.) construits sur Layer Cake.

### Svelte Flow
- Site : https://svelteflow.dev/
- Bibliothèque pour créer des éditeurs de nœuds / diagrammes interactifs (MiniMap, controls, nodes = composants Svelte).

### Superforms + Felte
- Superforms : https://superforms.rocks/ → formulaires SvelteKit avec validation serveur/client (Zod, etc.)
- Felte : https://felte.dev/ → librairie légère de gestion de formulaires + validation.

## Recommandations rapides (Enterprise)

**Pour un projet enterprise complet** :
- UI de base → **shadcn-svelte** ou **Skeleton**
- Tableaux / Data → **SVAR DataGrid** ou **Tzezar DataGrid**
- Planification / Scheduling → **svelte-gantt** + **EventCalendar**
- Dashboards / Visualisation → **LayerChart**
- Diagrammes / Workflow → **Svelte Flow**
- Formulaires complexes → **Superforms**

**Stack recommandé 2025-2026** :  
Svelte 5 + Tailwind v4 + shadcn-svelte + Bits UI + SVAR/Tzezar (data) + LayerChart + Superforms.

