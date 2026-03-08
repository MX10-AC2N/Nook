# 🎨 Rôle : UI/UX Optimizer — Nook

> Garant de la cohérence visuelle, de l'accessibilité et des performances frontend.
> Activer ce rôle pour : refactorisation CSS, audit design system, nouvelles pages, a11y.

---

## 🎯 Périmètre

```
frontend/src/
├── app.css                        → Global reset + import thèmes
├── lib/ui/themes/
│   ├── themes.css                 → Variables globales (typo, spacing, shadows, animations)
│   ├── jardin-secret.css          → Thème vert/naturel (défaut)
│   ├── space-hub.css              → Thème sombre/violet
│   └── maison-chaleureuse.css     → Thème ambre/chaleureux
└── routes/**/+page.svelte         → Pages à auditer
```

---

## 📐 Design System Nook — Référence complète

### Variables définies dans `themes.css` (globales, jamais override)

```css
/* Typographie */
--font-primary | --font-mono
--text-xs(0.75) | --text-sm(0.875) | --text-base(1) | --text-lg(1.125)
--text-xl(1.25) | --text-2xl(1.5) | --text-3xl(1.875) | --text-4xl(2.25)
--leading-tight(1.25) | --leading-normal(1.5) | --leading-relaxed(1.625)
--font-normal(400) | --font-medium(500) | --font-semibold(600) | --font-bold(700)

/* Espacement */
--space-1(0.25rem) ... --space-20(5rem)

/* Rayons */
--radius-sm(0.25) | --radius-md(0.5) | --radius-lg(0.75) | --radius-xl(1)
--radius-2xl(1.5) | --radius-full(9999px)

/* Ombres */
--shadow-sm | --shadow-md | --shadow-lg | --shadow-xl | --shadow-2xl | --shadow-inner

/* Effets */
--glow-sm(0 0 0 2px var(--accent))
--glow-md(0 0 0 4px var(--accent))
--glow-lg(0 0 20px var(--accent))

/* Transitions */
--transition-fast(150ms) | --transition-normal(250ms) | --transition-slow(350ms)

/* Animations (keyframes définis) */
--fade-in | --slide-up | --slide-down | --scale-in | --pulse
```

### Variables par thème (à toujours utiliser dans les pages)

```css
/* Couleurs de fond */
--bg-primary     → fond principal de la page
--bg-secondary   → cartes, sidebars, zones secondaires
--bg-tertiary    → hover states, zones très légères
--input-bg       → fond des inputs (souvent blanc même en thème sombre)

/* Typographie */
--text-primary   → texte principal (titres, labels)
--text-secondary → texte secondaire (sous-titres, descriptions)
--text-muted     → texte très atténué (placeholders, dates)

/* Accent / boutons */
--accent         → couleur principale du thème (boutons, liens actifs)
--accent-light   → version plus claire (hover léger, badges)
--accent-dark    → version plus foncée (hover boutons, focus)
--button-hover   → background bouton au hover

/* Bordures */
--border         → bordures standard
--border-light   → bordures très légères (dividers)

/* Chat */
--chat-mine      → fond bulle mes messages
--chat-theirs    → fond bulle messages des autres

/* Statuts */
--status-online  | --status-away | --status-offline

/* Effets et profondeur */
--depth          → box-shadow standard des cartes
--depth-lg       → box-shadow cartes hover
--animation      → cubic-bezier propre au thème

/* RGB pour rgba() */
--bg-rgb         → ex: rgba(var(--bg-rgb), 0.7)
--accent-rgb     → ex: rgba(var(--accent-rgb), 0.2)

/* Sémantique */
--success | --success-light
--warning | --warning-light
--error   | --error-light
--info    | --info-light
--notification-bg | --notification-text
```

---

## 🚨 Règle absolue N°1 — Zéro couleur hexadécimale hardcodée

**JAMAIS** de couleur hex `#xxxxxx` dans un fichier `.svelte`. Toujours une variable CSS.

```css
/* ❌ Interdit — brise les thèmes */
color: #1e293b;
background: #4ade80;
border: 1px solid #e2e8f0;

/* ✅ Correct */
color: var(--text-primary);
background: var(--accent);
border: 1px solid var(--border);
```

**Exception unique** : les fichiers `.css` des thèmes eux-mêmes.

### Table de conversion hex → variable

| Hex fréquent | Variable |
|---|---|
| `#1e293b`, `#0f172a` | `var(--text-primary)` |
| `#64748b`, `#94a3b8` | `var(--text-secondary)` |
| `#94a3b8`, `#b45309` | `var(--text-muted)` |
| `#f8fafc`, `#f1f5f9` | `var(--bg-secondary)` |
| `#e2e8f0`, `#cbd5e1` | `var(--border)` |
| `#4ade80`, `#8b5cf6`, `#ea580c` | `var(--accent)` (thème-dépendant) |
| `#22c55e`, `#7c3aed`, `#c2410c` | `var(--accent-dark)` |
| `#86efac`, `#a78bfa`, `#fb923c` | `var(--accent-light)` |
| `white`, `#ffffff` | `var(--input-bg)` ou `var(--chat-theirs)` |
| `#dc2626`, `#ef4444` | `var(--error)` |
| `#22c55e` | `var(--success)` |
| `#fbbf24`, `#eab308` | `var(--warning)` |

---

## 🚨 Règle absolue N°2 — Jamais de `rgba(0,0,0, x)` hardcodé

Utiliser les variables RGB fournies :

```css
/* ❌ */
box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
background: rgba(74, 222, 128, 0.2);

/* ✅ */
box-shadow: var(--depth);
background: rgba(var(--accent-rgb), 0.2);
/* ou : */
background: color-mix(in srgb, var(--accent) 20%, transparent);
```

---

## 🧩 Composants globaux disponibles (classes CSS dans `themes.css`)

```
Boutons    : .btn .btn-primary .btn-secondary .btn-ghost .btn-icon
Inputs     : .input (avec .input:focus déjà styé)
Cartes     : .card .card-hover .card-glass
Chat       : .chat-bubble .chat-mine .chat-theirs .chat-bubble-meta
Statuts    : .status-dot .status-online .status-away .status-offline
Liste      : .list-item .list-item-avatar .list-item-content
Nav        : .header .nav-bar .nav-item
Modales    : .modal-overlay .modal .modal-header .modal-footer
Badges     : .badge .badge-secondary
Loaders    : .spinner .skeleton
Utilitaires: .sr-only .flex-center .truncate .w-full .hidden
```

**Règle** : avant d'écrire un style custom dans un `<style>` scoped, vérifier si la classe globale couvre le besoin.

---

## 📊 Audit — État actuel des pages (session 35)

| Page | Hex hardcodés | Priorité fix |
|---|---|---|
| `chess/[game_id]/+page.svelte` | 105 | 🔴 Haute |
| `chat/+page.svelte` | 105 | 🔴 Haute |
| `chess/+page.svelte` | 53 | 🔴 Haute |
| `polls/+page.svelte` | 51 | 🟡 Moyenne (partiellement fixé S34) |
| `calendar/+page.svelte` | 42 | 🟡 Moyenne |
| `admin/+page.svelte` | 39 | 🟡 Moyenne (partiellement fixé S34) |
| `call/[id]/+page.svelte` | 35 | 🟡 Moyenne |
| `+layout.svelte` | 35 | 🔴 Haute (global, impact maximal) |
| `settings/+page.svelte` | 31 | 🟡 Moyenne (partiellement fixé S34) |
| `admin/analytics/+page.svelte` | 28 | 🟢 Basse |
| `help/+page.svelte` | 26 | 🟢 Basse (fixé S34) |
| `join/+page.svelte` | 21 | 🟢 Basse |
| `change-password/+page.svelte` | 21 | 🟢 Basse |

**Priorité suivante** : `+layout.svelte` (35 hex → impact global sur tous les thèmes).

---

## 🎯 Checklist d'intervention — Refactorisation CSS

Avant de livrer une page refactorisée, vérifier :

```
□ 0 couleur #hex dans le <style> scoped
□ 0 rgba(N, N, N, x) hardcodé (utiliser var(--accent-rgb) ou color-mix)
□ Toutes les tailles de police → var(--text-*)
□ Tous les rayons → var(--radius-*)
□ Tous les spacings → var(--space-*) ou rem cohérent
□ box-shadow → var(--depth) / var(--shadow-*) / var(--glow-*)
□ Transitions → var(--transition-fast) / --transition-normal
□ Animations → var(--animation) pour cubic-bezier
□ Utilisation des classes globales .btn, .input, .card quand pertinent
□ Mode sombre vérifié (thème space-hub) → pas de texte illisible
□ Thème Maison vérifié → fond orangé → contrastes OK
□ Responsive vérifié @media (max-width: 640px)
□ Pas de position: fixed sans z-index explicite
□ Focus visible sur tous les éléments interactifs (outline ou glow-sm)
```

---

## ♿ Checklist Accessibilité (a11y)

```
□ Tous les boutons ont un aria-label ou du texte visible
□ Tous les inputs ont un <label> associé (for= ou aria-labelledby=)
□ Images : alt non vide (ou alt="" si décorative)
□ Contraste texte/fond ≥ 4.5:1 (AA) — spécialement sur Space Hub
□ Focus trap dans les modales (tabindex, Escape pour fermer)
□ role="alert" sur les messages d'erreur/succès
□ Pas de animation si prefers-reduced-motion (ajouter @media check)
□ Liens de navigation : aria-current="page" sur l'élément actif
□ Éléments interactifs ≥ 44×44px sur mobile
```

---

## 🔧 Patterns CSS recommandés — Nook

### Couleur avec opacité (remplace rgba hardcodé)
```css
/* ✅ Moderne, thème-aware */
background: color-mix(in srgb, var(--accent) 15%, transparent);
border-color: color-mix(in srgb, var(--accent) 40%, transparent);
```

### Dark mode overlay sur un thème
```css
:global(.dark-mode) .ma-section {
  filter: brightness(0.82) saturate(0.9);
}
```

### Animation thème-aware
```css
.mon-element {
  animation: fade-in 0.3s var(--animation);
  /* var(--animation) = cubic-bezier propre au thème (ressort vs linéaire) */
}
```

### Focus ring cohérent
```css
.btn:focus-visible,
.input:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  box-shadow: var(--glow-sm);
}
```

### Responsive — breakpoints Nook
```css
/* Mobile first */
@media (min-width: 640px)  { /* sm — tablette portrait */ }
@media (min-width: 768px)  { /* md — tablette paysage */ }
@media (min-width: 1024px) { /* lg — desktop          */ }

/* Ou : max-width si mobile-last (legacy) */
@media (max-width: 640px) { /* petit mobile */ }
```

---

## 📏 Règles de layout

### `+layout.svelte` — Structure fixe (ne pas modifier)
```
<header class="app-header">   position:sticky top:0 z:100
<main class="app-main">       padding:1.5rem max-width:1200px margin:auto
<footer class="app-footer">   border-top fixe
Menu slide-in                 z:201 (au-dessus overlay z:200)
```

### Pages — Pattern standard
```svelte
<div class="page-container">
  <header class="page-header">
    <h1>🎯 Titre</h1>
    <p class="subtitle">Description courte</p>
  </header>
  <!-- contenu -->
</div>

<style>
  .page-container {
    max-width: 800px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
    color: var(--text-primary);
  }
  .page-header { text-align: center; margin-bottom: var(--space-8); }
  .page-header h1 { font-size: var(--text-2xl); color: var(--text-primary); margin: 0 0 var(--space-2); }
  .subtitle { color: var(--text-secondary); font-size: var(--text-sm); margin: 0; }
</style>
```

---

## 🤝 Flux inter-agents

```
Déclencheur : tout commit touchant un fichier .svelte
← 📐 ARCHITECT : nouvelles pages/features à styler
← 🦀 RUST      : nouvelles données à afficher
→ 🧪 E2E       : data-testid= à préserver sur les éléments refactorisés
→ 📐 ARCHITECT : signaler si une feature nécessite un nouveau composant partagé
```

**Contrat** : ne jamais supprimer un attribut `id=`, `data-testid=` ou `aria-*` lors d'une refactorisation CSS.

---

## 📚 Apprentissages

### [UI-01] layout.svelte n'utilise pas le design system — Session 35
Le layout global utilise encore 16 couleurs hex hardcodées (`#4ade80`, `#1e293b`…).
→ **Fix prioritaire** car le header/menu/footer s'affichent sur TOUTES les pages.
→ Space Hub (fond sombre) : le header reste blanc — non themé.

### [UI-02] `color-mix()` disponible — Session 35
`color-mix(in srgb, var(--accent) 20%, transparent)` est supporté par tous les navigateurs
cibles (Chrome 111+, Firefox 113+, Safari 16.2+). À préférer à `rgba()` hardcodé.

### [UI-03] Sessions 33-34 : 5 pages partiellement migrées
`settings`, `admin`, `polls`, `chat`, `help` ont reçu des variables CSS en S34.
Résidu : hex dans les états hover, focus, médias queries, fallbacks.

### [UI-04] `--shadow-2xl` utilisé dans les .svelte mais non défini dans les thèmes
`var(--shadow-2xl)` est utilisé dans `themes.css` mais **pas** dans les fichiers thème.
Il est défini dans `themes.css` `:root` → OK. Mais `--space-1` à `--space-6` aussi
utilisés depuis les svelte → définis dans `themes.css` `:root` → OK.

### [UI-05] Variables redondantes à unifier — Session 35
`--glow-sm` référencé depuis `.input:focus` dans `themes.css` mais valeur dépend de
`var(--accent)` → ✅ dynamique, correct. Pas de bug.
