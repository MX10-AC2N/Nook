# 🎨 Contexte Frontend - Nook

> Mis à jour : 2026-05-05

## Stack Technique

- **Framework** : SvelteKit 5 (Runes mode)
- **State** : Svelte 5 Runes ($state, $derived, $effect)
- **Styling** : CSS variables, dark/light themes
- **Build** : Vite + adapter-static
- **E2EE** : Crypto API (XChaCha20-Poly1305)

## Architecture

```
frontend/
├── src/
│   ├── routes/             # SvelteKit routes (+layout.svelte, +page.svelte)
│   │   ├── chat/           # Chat pages
│   │   ├── calls/          # WebRTC calls
│   │   ├── chess/          # Chess game
│   │   └── ...
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   ├── stores/         # Svelte 5 rune stores
│   │   │   ├── chatStore.svelte.js
│   │   │   ├── cryptoStore.svelte.js
│   │   │   └── ...
│   │   └── utils/          # Helpers
│   └── app.html
├── static/                 # Assets statiques
└── package.json
```

## Règles Svelte 5

### ✅ Correct
```svelte
<script>
  let { name } = $props();
  let count = $state(0);
  let doubled = $derived(count * 2);
  
  function increment() {
    count++;
  }
</script>

<button onclick={increment}>{count}</button>
```

### ❌ Incorrect (éviter)
```svelte
<!-- Pas d'expressions complexes dans le template -->
<button>{count * 2}</button>  <!-- À éviter, préférer $derived -->
```

## Points Critiques

### ✅ Corrections Récentes
- **+layout.svelte** : Supprimé `{capture}` invalide ligne 2
- **package-lock.json** : Régénéré avec dépendances optionnelles (Rollup)
- **npm ci** : Supprimé `--omit optional` du workflow Frontend.yml

### ⚠️ À Surveiller
- **Svelte 5 migration** : Plus d'expressions complexes dans les templates
- **Stores** : Migration vers les runes ($state au lieu de writable)
- **a11y** : Attributs alt sur les images, contrastes

## Commandes Utiles

```bash
# Dev server
npm run dev

# Build production
npm run build

# Check Svelte
npx svelte-check

# MCP Svelte (si disponible)
# Utiliser list-sections → get-documentation → svelte-autofixer
```

## Patterns Récurrents

### Store Svelte 5
```javascript
// lib/stores/myStore.svelte.js
import { writable, derived } from 'svelte/store';

export function createMyStore() {
  let count = $state(0);
  
  return {
    get count() { return count; },
    increment() { count++; }
  };
}
```

### Route avec E2EE
```svelte
<script>
  import { cryptoStore } from '$lib/stores/cryptoStore.svelte.js';
  
  async function sendMessage(text) {
    const encrypted = await cryptoStore.encrypt(text);
    // Envoyer encrypted au backend
  }
</script>
```

## Connexions MCP

- **Svelte MCP** (https://mcp.svelte.dev/mcp) : Documentation Svelte 5
- **svelte-llm** : Alternative Svelte MCP
