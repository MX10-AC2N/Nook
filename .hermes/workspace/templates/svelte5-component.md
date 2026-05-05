# Template : Composant Svelte 5

```svelte
<!-- src/routes/new-page/+page.svelte -->

<script>
  // Props (si composant réutilisable)
  let { title = "Default Title" } = $props();
  
  // State (Runes Svelte 5)
  let count = $state(0);
  let items = $state([]);
  let loading = $state(false);
  let error = $state(null);
  
  // Derived values
  let doubled = $derived(count * 2);
  let hasItems = $derived(items.length > 0);
  
  // Effects
  $effect(() => {
    console.log('Count changed:', count);
  });
  
  // Fonctions helper (pas d'expressions complexes dans le template)
  function increment() {
    count++;
  }
  
  async function fetchItems() {
    loading = true;
    error = null;
    try {
      const response = await fetch('/api/items');
      items = await response.json();
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }
  
  // Lifecycle (mount/unmount)
  $effect(() => {
    fetchItems();
    
    return () => {
      // Cleanup on unmount
      console.log('Component unmounted');
    };
  });
</script>

<!-- Template (pas d'expressions complexes ici) -->
<div class="container">
  <h1>{title}</h1>
  
  {#if loading}
    <p>Chargement...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <button onclick={increment}>
      Count: {count} (doubled: {doubled})
    </button>
    
    {#if hasItems}
      <ul>
        {#each items as item (item.id)}
          <li>{item.name}</li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .container {
    padding: 1rem;
  }
  
  .error {
    color: var(--color-error, red);
  }
</style>
```

## Store Svelte 5 (optionnel)

```javascript
// src/lib/stores/myStore.svelte.js
import { writable } from 'svelte/store';

function createMyStore() {
  let count = $state(0);
  let items = $state([]);
  
  return {
    get count() { return count; },
    get items() { return items; },
    increment() { count++; },
    async loadItems() {
      const res = await fetch('/api/items');
      items = await res.json();
    }
  };
}

export const myStore = createMyStore();
```

## Utilisation avec E2EE (si nécessaire)

```svelte
<script>
  import { cryptoStore } from '$lib/stores/cryptoStore.svelte.js';
  
  let message = $state('');
  
  async function sendEncrypted() {
    const encrypted = await cryptoStore.encrypt(message);
    await fetch('/api/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ payload: encrypted })
    });
    message = '';
  }
</script>
```
