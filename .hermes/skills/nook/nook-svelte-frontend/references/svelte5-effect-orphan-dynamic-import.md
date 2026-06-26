# Svelte 5 `effect_orphan` lors d'import dynamique dans root layout

## Problème

Lorsqu'un module `.svelte.ts` (ex: `chatStore.svelte.ts`) contient :
1. Un `$effect` au niveau module qui surveille un `$state` d'un **autre** module (ex: `cryptoStore.ready`)
2. Ce module est importé dynamiquement dans `+layout.svelte` (root) via `import('/_app/.../app.js').then(...)`

Au moment où `kit.start(app, element)` s'exécute, l'import dynamique charge le module → le `$effect` s'enregistre → mais il n'y a pas de composant parent actif → **`effect_orphan` runtime error** → page blanche.

## Stack trace typique

```
Error: https://svelte.dev/e/effect_orphan
  at Ce (_app/immutable/chunks/B--DwDqQ.js)
  at Vr (_app/immutable/chunks/B--DwDqQ.js)
  at Kr (_app/immutable/chunks/B--DwDqQ.js)
  at .../DMcFecfH.js (le module qui importe chatStore)
```

## Solution pattern

**Ne JAMAIS** mettre un `$effect` réactif cross-module au niveau module dans un fichier qui peut être importé dynamiquement avant montage composant.

### Pattern correct

```typescript
// chatStore.svelte.ts
// ❌ AVANT (cassé)
$effect(() => {
  if (cryptoStore.ready) _decryptAllIfReady();
});
_setupCryptoReadyListener(); // appelle l'effet au chargement module

// ✅ APRÈS (fix)
function _setupCryptoReadyListener(): void { ... }

// Export pour initialisation côté composant
export function initCryptoListener(): void {
  _setupCryptoReadyListener();
}

// Dans chat/+page.svelte
onMount(() => {
  await loadMessages(activeConvId);
  initCryptoListener(); // ← Appelé DANS le composant monté
  setActiveConv(activeConvId);
});
```

## Règle générale

| Contexte | `$effect` autorisé ? |
|----------|---------------------|
| Dans `.svelte` (composant) | ✅ Oui |
| Dans `.svelte.ts` store, appelé depuis `onMount` composant | ✅ Oui (via fonction exportée) |
| Dans `.svelte.ts` store, au niveau module (chargement eager) | ❌ Non si surveille `$state` cross-module |
| Dans `.svelte.ts` store, au niveau module (import dynamique root layout) | ❌ **Jamais** — cause `effect_orphan` |

## Diagnostique rapide

| Symptôme | Vérification |
|----------|-------------|
| Page blanche, HTML charge mais `<body>` vide | `effect_orphan` dans console browser |
| Erreur Svelte `https://svelte.dev/e/effect_orphan` | Chercher `$effect` au niveau module dans stores importés par layout |
| Build OK mais runtime fail | L'erreur est **runtime only** — le build ne la détecte pas |

## Application Nook

Ce pattern s'applique à **tous les stores Svelte 5** qui :
- Sont importés par d'autres stores (`chatStore` → `cryptoStore`)
- Sont chargés via l'import dynamique SvelteKit dans `+layout.svelte`
- Utilisent `$effect` pour surveiller l'état d'un autre store

Fichiers concernés potentiellement :
- `chatStore.svelte.ts` (cryptoStore.ready) — **CORRIGÉ session actuelle**
- `webrtc-calls.svelte.ts` (surveille cryptoStore, authStore)
- `mediaStore.svelte.js` (si surveille d'autres stores)
- `conversationStore.svelte.ts` (si surveille authStore)

## Test de régression

```bash
# Après modification d'un store, vérifier :
curl http://192.168.1.192:6300 | grep -c "effect_orphan"  # doit être 0

# Ou au browser console :
# - Pas d'erreur Svelte effect_orphan
# - Page charge normalement (pas blanc)
```