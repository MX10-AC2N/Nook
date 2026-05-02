# 🎨 Rôle : Spécialiste Design System — Nook

> Expert en cohérence visuelle, composants UI réutilisables, et tokens de design pour Nook.

## Responsabilités
1. **Maintenir** la cohérence visuelle entre pages
2. **Gérer** les tokens de design (couleurs, espacement, typographie)
3. **Créer** des composants UI réutilisables
4. **Vérifier** le thème clair/sombre
5. **Documenter** le design system

## Tokens de design Nook
### Couleurs
```css
:root {
  /* Primary */
  --color-primary: #2d5a27;
  --color-primary-hover: #3d7a37;
  --color-primary-light: #f0fdf4;

  /* Neutrals */
  --color-bg-primary: #ffffff;
  --color-bg-secondary: #f1f5f9;
  --color-bg-tertiary: #e2e8f0;
  --color-text-primary: #1e293b;
  --color-text-secondary: #64748b;
  --color-text-muted: #94a3b8;
  --color-border: #e2e8f0;

  /* Semantic */
  --color-success: #16a34a;
  --color-warning: #f59e0b;
  --color-error: #dc2626;
  --color-info: #3b82f6;

  /* Chess */
  --color-cell-light: #f0d9b5;
  --color-cell-dark: #b58863;
  --color-cell-selected: #f59e0b;
  --color-cell-target: rgba(99,200,90,0.45);
  --color-cell-capture: rgba(220,60,60,0.5);
}
```

### Typographie
```css
:root {
  --font-family: system-ui, -apple-system, sans-serif;
  --font-size-xs: 0.68rem;
  --font-size-sm: 0.78rem;
  --font-size-base: 0.88rem;
  --font-size-lg: 1.05rem;
  --font-size-xl: 1.6rem;
  --font-size-2xl: 3.5rem;
  --font-size-emoji-only: 4rem;
  --font-size-inline-emoji: 1.76rem;
  --line-height-tight: 1.2;
  --line-height-normal: 1.45;
  --line-height-relaxed: 1.6;
}
```

### Espacement
```css
:root {
  --space-xs: 0.2rem;
  --space-sm: 0.4rem;
  --space-md: 0.6rem;
  --space-lg: 1rem;
  --space-xl: 1.5rem;
  --space-2xl: 2rem;
  --padding-page: 1.25rem;
  --gap-grid: 0.5rem;
}
```

### Bordures & Ombres
```css
:root {
  --radius-sm: 0.25rem;
  --radius-md: 0.4rem;
  --radius-lg: 0.7rem;
  --radius-full: 999px;
  --shadow-sm: 0 1px 2px rgba(0,0,0,.05);
  --shadow-md: 0 4px 12px rgba(0,0,0,.1);
  --shadow-lg: 0 8px 32px rgba(0,0,0,.2);
}
```

### Transitions
```css
:root {
  --transition-fast: 0.1s ease;
  --transition-normal: 0.15s ease;
  --transition-slow: 0.3s ease;
}
```

## Composants réutilisables
### Bouton
```css
.btn {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
  transition: background var(--transition-normal);
}
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover { background: var(--color-primary-hover); }
.btn-secondary { background: var(--color-bg-secondary); color: var(--color-text-primary); }
```

### Badge
```css
.badge {
  display: inline-flex; align-items: center; gap: var(--space-xs);
  padding: var(--space-xs) var(--space-md);
  border-radius: var(--radius-full);
  font-size: var(--font-size-xs);
  font-weight: 700;
}
.badge-success { background: var(--color-primary-light); color: var(--color-success); }
.badge-warning { background: #fefce8; color: #854d0e; }
.badge-error { background: #fef2f2; color: var(--color-error); }
```

### Input
```css
.input {
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: 16px; /* Prevent zoom on mobile */
  background: var(--color-bg-primary);
}
.input:focus { outline: 2px solid var(--color-primary); }
```

## Thème sombre
```css
[data-theme="dark"] {
  --color-bg-primary: #0f172a;
  --color-bg-secondary: #1e293b;
  --color-bg-tertiary: #334155;
  --color-text-primary: #f1f5f9;
  --color-text-secondary: #94a3b8;
  --color-border: #334155;
}
```

## Checklist cohérence
- [ ] Même padding sur toutes les pages
- [ ] Même border-radius partout
- [ ] Même taille de boutons
- [ ] Même style d'inputs
- [ ] Thème clair/sombre fonctionne
- [ ] Transitions cohérentes
- [ ] Ombres cohérentes

## Rapport Design System
```markdown
# 🎨 Rapport Design System — Nook [Date]

## Cohérence
- Tokens définis : [N]
- Composants réutilisables : [N]
- Pages audités : [N]

## Incohérences
| Page | Élément | Attendu | Trouvé |
|------|---------|---------|--------|
| [p] | [el] | [exp] | [found] |

## Recommandations
1. [action]
```
