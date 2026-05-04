# 🎨 Mémoire SVELTE - Apprentissages & Patterns

> **DERNIÈRE MISE À JOUR** : 2026-05-04
> Patterns Svelte 5 Runes + SvelteKit pour Nook

## 📦 Stack Frontend

### Versions
- **Svelte** : 5 (Runes mode OBLIGATOIRE)
- **SvelteKit** : Latest
- **TypeScript** : Oui
- **Build** : SvelteKit avec adapter (voir svelte.config.js)

## 🎯 Svelte 5 Runes - Patterns Critiques

### Variables Réactives
```svelte
<script>
  // ✅ State de base
  let count = $state(0);
  
  // ✅ State avec objet
  let user = $state({ name: 'John', age: 25 });
  
  // ✅ Derived (calculée)
  let doubled = $derived(count * 2);
  
  // ✅ Derived avec fonction (pour logique complexe)
  let formatted = $derived.by(() => {
    return `${user.name} (${user.age})`;
  });
  
  // ✅ Effect (side effects)
  $effect(() => {
    console.log(`Count changed: ${count}`);
  });
</script>
```

### Props (Propriétés du composant)
```svelte
<script>
  // ✅ Svelte 5 - destructuring avec $props()
  let { name, age = 25 } = $props();
</script>
```

### Template Syntax
```svelte
<!-- ✅ Condition (Svelte 5) -->
{#if count > 0}
  <p>Count is positive</p>
{:else if count < 0}
  <p>Count is negative</p>
{:else}
  <p>Count is zero</p>
{/if}

<!-- ✅ Boucle -->
{#each items as item, index (item.id)}
  <div>{index}: {item.name}</div>
{/each}

<!-- ❌ Ne PAS mettre d'expressions complexes dans le template -->
<!-- ✅ Utiliser une fonction helper -->
```

## 🚫 Règles Strictes Svelte 5

### Pas d'expressions complexes dans le template
```svelte
<!-- ❌ INCORRECT -->
<div class:active={item.status === 'active' && user.isAdmin && theme === 'dark'}>
  ...
</div>

<!-- ✅ CORRECT - utiliser une fonction helper -->
<script>
  function isActive(item, user, theme) {
    return item.status === 'active' && user.isAdmin && theme === 'dark';
  }
</script>
<div class:active={isActive(item, user, theme)}>
  ...
</div>
```

### Pas d'accolades simples pour les conditions
```svelte
<!-- ❌ INCORRECT (Svelte 4) -->
{if count > 0}
  ...
{/if}

<!-- ✅ CORRECT (Svelte 5) -->
{#if count > 0}
  ...
{/if}
```

## 🎨 Patterns SvelteKit

### Load Functions
```typescript
// ✅ +page.ts ou +page.server.ts
export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/users');
  const users = await res.json();
  return { users };
};
```

### Form Actions
```typescript
// ✅ +page.server.ts
export const actions: Actions = {
  default: async ({ request }) => {
    const formData = await request.formData();
    const name = formData.get('name');
    // ...
    return { success: true };
  }
};
```

### API Routes
```typescript
// ✅ +server.ts
export const GET: RequestHandler = async ({ url }) => {
  const id = url.searchParams.get('id');
  return json({ id });
};
```

## 🖼️ Assets & Icons

### SVG Icons (OBLIGATOIRE - pas d'emojis)
```svelte
<!-- ✅ Utiliser des composants SVG ou import -->
<script>
  import SettingsIcon from '$lib/assets/icons/settings.svg?component';
</script>
<SettingsIcon />
```

### Images
```svelte
<!-- ✅ Import statique -->
<script>
  import logo from '$lib/assets/logo.png';
</script>
<img src={logo} alt="Logo" />

<!-- ✅ Import dynamique via Vite -->
<img src="/images/photo.jpg" alt="Photo" />
```

## 🎭 Thèmes (Dark/Light)

### Gestion des thèmes
```svelte
<script>
  let theme = $state('light');
  
  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', theme);
  }
</script>

<div class="theme-{theme}">
  ...
</div>
```

## 📱 PWA & Responsive

### Manifest & Service Worker
- Fichier `static/manifest.json` présent
- PWA broken (P0 à fixer) - voir known-issues.md

### Responsive Design
```css
/* ✅ Utiliser rem et em */
.container {
  max-width: 1200rem;
  padding: 1rem;
}

/* ✅ Media queries */
@media (max-width: 768px) {
  .sidebar { display: none; }
}
```

## 🧪 Tests

### Structure de tests
- ❌ Pas de tests frontend actuellement (P1)
- 106 E2E tests skippés (P1)
- Voir `nook-frontend-audit.md` pour détails

## 📝 Notes de Session

- Migration vers Svelte 5 Runes réussie
- Plusieurs corrections de syntaxe effectuées
- MCP Svelte disponible pour documentation/autofixer
- Build frontend échoue parfois (voir tools-state.md)

---
*Mettre à jour après chaque session de dev frontend*
