# 🎨 Lib UI Resources — Nook

> Bibliothèques UI Svelte évaluées pour Nook.
> Focalisé sur ce qui est compatible Svelte 5 + Tailwind v4 + notre système de thèmes CSS variables.
> Mis à jour : session 44

---

## ✅ Recommandé pour Nook

### shadcn-svelte — Composants headless prêts à l'emploi
**Site :** https://www.shadcn-svelte.com/  
**Svelte 5 ✅ | Tailwind v4 ✅ | TypeScript ✅**

Contrairement aux autres libs, shadcn **copie le code source** dans le projet — pas de dépendance externe. On adapte le CSS aux variables de thème Nook (`--accent`, `--bg-primary`, etc.).

**Composants utiles pour Nook :**

| Composant | Usage dans Nook |
|---|---|
| `Dialog` | Remplacer nos modals manuels |
| `Toast` | Notifications succès/erreur |
| `DropdownMenu` | Menu contexte messages |
| `Select` | Sélection difficulté IA chess |
| `Tabs` | Onglets emoji/GIF dans le picker |
| `Popover` | Picker émoji ancré sur le message |

```bash
npx shadcn-svelte@latest init
npx shadcn-svelte@latest add dialog toast dropdown-menu
```

---

### bits-ui — Primitives headless ultra-légères
**Site :** https://bits-ui.com/  
**Svelte 5 ✅ | 0 dépendance externe ✅**

Utilisé par shadcn-svelte en interne. Utile si on veut construire un composant très custom (ex: picker de date pour le calendrier, slider pour le volume dans les appels).

---

### AutoAnimate — Animations de listes sans configuration
**Site :** https://auto-animate.formkit.com/

```typescript
import autoAnimate from '@formkit/auto-animate'

// Animer l'apparition de messages dans le chat
let messagesEl: HTMLElement;
$effect(() => { if (messagesEl) autoAnimate(messagesEl); });
```
**Usage dans Nook :** Apparition des messages, ouverture/fermeture des conversations, listes de sondages.

---

## 🟡 À considérer plus tard

### vkurko/calendar — Calendrier multi-vues
**Repo :** https://github.com/vkurko/calendar  
**Quand :** Si on ajoute une vue semaine ou jour au calendrier familial  
**Pas maintenant :** Notre calendrier custom S43 couvre les besoins actuels (mois + clic + édition)

### LayerChart — Graphiques avancés
**Site :** https://www.layerchart.com/  
**Quand :** Si on enrichit la page analytics admin (Chart.js est déjà en place et suffisant)

---

## ❌ Non retenus pour Nook

| Lib | Raison |
|---|---|
| Carbon Components | IBM design system — style incompatible avec nos thèmes |
| Skeleton | Design system complet — trop opinionated, conflit avec notre système CSS variables |
| Flowbite Svelte | Basé sur Tailwind classes hardcodées — incompatible avec nos thèmes CSS variables |
| svelte-motion | Framer Motion port — animations trop complexes pour nos besoins |
| Svelte Headless UI | Équipe officielle Svelte — abandonné, pas maintenu en Svelte 5 |
| SVAR DataGrid | Enterprise data grid — hors scope pour une app familiale |

---

## 🎨 Notre système de thèmes — Règle absolue

Toute lib UI intégrée dans Nook **doit** utiliser nos variables CSS, pas des couleurs hardcodées :

```css
/* ✅ Variables à utiliser */
var(--bg-primary)
var(--bg-secondary)
var(--bg-tertiary)
var(--accent)
var(--accent-dark)
var(--text-primary)
var(--text-secondary)
var(--border)
var(--depth)

/* ❌ Jamais de couleurs hardcodées dans les composants Nook */
color: #1e293b;  /* même si c'est "la bonne couleur" pour jardin-secret */
```

Les 4 thèmes actuels (`jardin-secret`, `space-hub`, `maison-chaleureuse`, `nuit-douce`) appliquent leurs valeurs sur `.theme-*` via `body.classList`.
