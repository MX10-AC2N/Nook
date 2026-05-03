# 🎨 Mémoire Svelte - Apprentissages Frontend

> Dernière mise à jour: 2026-05-03
> Consulté lors de tout dev frontend

## 📦 Versions & Outils

- **SvelteKit** : 5.x (Runes mode)
- **Svelte** : 5.x
- **TypeScript** : Strict mode
- **MCP Svelte** : `https://mcp.svelte.dev/mcp` (obligatoire depuis S38)

## 🧠 Svelte 5 Runes - Patterns

### Variables réactives
```svelte
<script>
  // ✅ Svelte 5 Runes
  let count = $state(0);
  let doubled = $derived(count * 2);
  let tripled = $derived.by(() => count * 3); // Pour expressions complexes
  
  function increment() {
    count++;
  }
</script>
```

### Props
```svelte
<script>
  // ✅ Svelte 5
  let { name, age = 25 } = $props();
</script>
```

### Effects
```svelte
<script>
  $effect(() => {
    console.log('count changed:', count);
  });
  
  // Cleanup
  $effect(() => {
    const timer = setInterval(...);
    return () => clearInterval(timer);
  });
</script>
```

## ⚠️ RÈGLES CRITIQUES

### 1. Pas d'expressions complexes dans templates
```svelte
<!-- ❌ ÉVITER -->
{#if myArray.filter(x => x.active).length > 0}
  ...

<!-- ✅ PRÉFÉRER -->
{@const activeCount = myArray.filter(x => x.active).length}
{#if activeCount > 0}
  ...
```

### 2. Syntaxe templates
```svelte
<!-- ✅ Svelte 5 -->
{#if condition}
{:else if otherCondition}
{:else}
{/if}

<!-- ❌ PLUS VALIDE -->
{if condition}
```

### 3. Stores (Svelte 5 way)
```svelte
<script>
  import { get } from 'svelte/store';
  import { userStore } from '$lib/stores';
  
  // Pour lire une valeur
  let user = get(userStore);
  
  // Pour s'abonner
  $: user = $userStore; // Déprécié en Svelte 5 ?
  // Ou utiliser $derived avec une fonction
</script>
```

## 🔧 Workflow MCP Svelte (OBLIGATOIRE)

Avant tout code Svelte :
```bash
# 1. Lister les sections disponibles
mcp_svelte_mcp_list_sections

# 2. Récupérer la doc pertinente
mcp_svelte_mcp_get_documentation --section "runes"
mcp_svelte_mcp_get_documentation --section "state"

# 3. Coder

# 4. Valider avec autofixer
mcp_svelte_mcp_svelte_autofixer --code "<votre code>" --desired_svelte_version 5
```

## 🎨 UI/UX - Règles Nook

- ✅ Utiliser **SVG icons** (pas d'emojis)
- ✅ Thème sombre/clair supporté
- ✅ Responsive (mobile first)
- ✅ Accessibilité (a11y) respectée

## 📝 Learnings Sessions

### Session 50-53
- ✅ Migration Svelte 5 Runes complétée
- ✅ Syntaxe `$derived.by(() => ...)` pour expressions complexes
- ✅ MCP Svelte intégré au workflow

### Erreurs fréquentes
1. **Expressions dans templates** → créer fonctions helper
2. **`{if}` au lieu de `{#if}`** → vérifier syntaxe
3. **Oublier MCP Svelte** → toujours consulter doc avant code

## 🔗 Ressources

- [Svelte 5 Docs](https://svelte.dev/docs/svelte)
- [SvelteKit Docs](https://kit.svelte.dev/docs)
- [MCP Svelte Server](https://mcp.svelte.dev/)

---
*Ajouter nouveaux apprentissages au fur et à mesure*
