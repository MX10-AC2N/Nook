# 🎨 Svelte 5 Resources — Nook

> Référence **opérationnelle** pour le développement Svelte 5 dans Nook.
> Focalisé sur les pièges connus, les patterns validés, et les libs utiles.
> Mis à jour : session 44

---

## ⚠️ Pièges Svelte 5 — Erreurs fréquentes dans Nook

### 1. `$derived(() => fn)` — PIÈGE CRITIQUE
```typescript
// ❌ Retourne la FONCTION, pas le résultat
const kingInCheckSquare = $derived(() => {
  return findKingSquare(engine.side_to_move);
});
// kingInCheckSquare est une Function, pas une string|null !

// ✅ IIFE pour les blocs multi-lignes
const kingInCheckSquare = $derived(
  (() => {
    return findKingSquare(engine.side_to_move);
  })()
);

// ✅ Ou expression directe si possible
const isOver = $derived(OVER_SET.has(game?.status ?? ''));
```

### 2. Export `$state` — Jamais de `let` réassignable
```typescript
// ❌ state_invalid_export au build
export let count = $state(0);
count = 1; // INTERDIT

// ✅ Objet encapsulant — mutation via propriété
export const myStore = $state<MyState>({ count: 0 });
myStore.count = 1; // OK
```

### 3. Modificateurs d'événements — Syntaxe Svelte 4 invalide
```svelte
<!-- ❌ Svelte 4 — invalide en Svelte 5, provoque une erreur de build -->
<div onclick|stopPropagation={() => fn()}>

<!-- ✅ Svelte 5 — handler inline -->
<div onclick={(e) => { e.stopPropagation(); fn(); }}>

<!-- ✅ Pour les modals (stopper la propagation sans handler) -->
<div onclick={(e) => e.stopPropagation()}>
```

### 4. `$derived` vs `$effect` — Quand utiliser quoi
```typescript
// $derived → valeur calculée réactive (pas d'effets de bord)
const totalUnread = $derived(
  Object.values(chatStore.unreadCounts).reduce((s, n) => s + (n ?? 0), 0)
);

// $effect → effets de bord réactifs (scroll, focus, DOM)
$effect(() => {
  if (showModal && inputRef) inputRef.focus();
});

// ❌ Jamais $derived/$effect hors d'un composant .svelte
// (ils ne fonctionnent que dans le contexte d'un composant)
```

### 5. `participants` dans conversationStore — Objet, pas Array
```typescript
// conversationStore.svelte.ts exporte :
export const participants = {
  get value() { return conversationStore.participants; },
  subscribe(fn) { ... }
};

// ❌ participants.map(p => p.id)  — TypeError: not a function
// ✅ participants.value.map(p => p.id)
```

---

## ✅ Patterns validés dans Nook

### Store global (pattern standard Nook)
```typescript
// lib/myStore.svelte.ts
interface MyState { items: string[]; loading: boolean; }

export const myStore = $state<MyState>({ items: [], loading: false });

export async function loadItems() {
  myStore.loading = true;
  try {
    const res = await fetch('/api/items', { credentials: 'include' });
    myStore.items = await res.json();
  } finally {
    myStore.loading = false;
  }
}
```

### Loading local à la page (évite race conditions)
```typescript
// ❌ Utiliser chessStore.loading (partagé entre pages → race condition)
// ✅ Loading local — toujours préféré pour les pages
let pageLoading = $state(true);
onMount(async () => {
  pageLoading = true;
  await chessStore.loadGame(gameId);
  pageLoading = false;
});
```

### scroll auto chat (pattern validé S43)
```typescript
$effect(() => {
  const count = chatStore.messages.length;
  if (!chatContainer || count === 0) return;
  // Ne scroll que si l'user est déjà près du bas
  const el = chatContainer;
  if (el.scrollHeight - el.scrollTop - el.clientHeight < 150) {
    Promise.resolve().then(() => {
      if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
    });
  }
});
```

---

## 📦 Bibliothèques UI — Recommandations pour Nook

### shadcn-svelte ← RECOMMANDÉ pour les prochains composants
**Site :** https://www.shadcn-svelte.com/  
**Svelte 5 natif ✅ + Tailwind v4 + Accessible**  
**Usage dans Nook :** Modals, dropdowns, select, toasts  
**Avantage :** CLI `npx shadcn-svelte add dialog` → composant dans le projet, pas de dépendance externe

```bash
# Installation
npm install -D @shadcn/svelte
npx shadcn-svelte init

# Ajouter un composant
npx shadcn-svelte add dialog
npx shadcn-svelte add toast
```

### bits-ui ← Pour les primitives headless
**Site :** https://bits-ui.com/  
**Usage dans Nook :** Si on veut construire des composants custom accessibles  
**Avantage :** TypeScript first, Svelte 5 natif, ARIA automatique

### vkurko/calendar ← Pour remplacer notre calendrier custom si besoin
**Repo :** https://github.com/vkurko/calendar  
**Usage :** Calendrier multi-vues (mois/semaine/jour), drag & drop natif  
**Note :** Notre calendrier custom S43 couvre les besoins actuels — à considérer si on ajoute les vues semaine/jour

### Superforms ← Pour les formulaires complexes
**Site :** https://superforms.rocks/  
**Usage dans Nook :** Validation côté client + serveur des formulaires invite/chess/polls  
**Note :** Intégration naturelle avec Zod ou Valibot pour typage des schémas

---

## 🎬 Animations

### AutoAnimate ← Simple, 0 config
```typescript
import autoAnimate from '@formkit/auto-animate'

// Dans une action Svelte
<ul use:autoAnimate>
  {#each items as item}
    <li>{item}</li>
  {/each}
</ul>
```
**Usage dans Nook :** Listes de messages, apparition de modals, transitions de pages

---

## 📚 Ressources officielles

- **Svelte 5 docs :** https://svelte.dev/docs/svelte
- **Migration Svelte 4→5 :** https://svelte.dev/docs/svelte/v5-migration-guide
- **SvelteKit docs :** https://svelte.dev/docs/kit
- **Playground :** https://svelte.dev/playground
