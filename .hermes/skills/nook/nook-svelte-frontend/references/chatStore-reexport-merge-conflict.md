# Résolution merge conflict chatStore.ts / chatStore.svelte.ts

## Contexte

Le projet Nook utilise un pattern de **ré-export** :
- `chatStore.svelte.ts` = implémentation réelle (stores, fonctions, logique)
- `chatStore.ts` = module `.ts` pur qui ré-exporte depuis `.svelte.ts` pour éviter les problèmes de résolution d'imports

## Problème rencontré

Un commit local (fix `effect_orphan`) a supprimé l'appel module-level `_setupCryptoReadyListener()` et exporté `initCryptoListener()` à la place.

Simultanément, le remote (`origin/develop`) avait ajouté 3 nouvelles exports à `chatStore.svelte.ts` :
- `MAX_BYTES_SERVER` (const)
- `cancelTransfer()` (function)
- `triggerDecryptAllIfReady()` (function)

Et `chatStore.ts` les ré-exportait déjà.

Lors du `git pull --rebase`, conflict sur `chatStore.ts` car le local n'avait plus ces exports dans l'objet de ré-export.

## Résolution correcte

1. **Accepter les changements remote** pour les exports ajoutés (ils sont nécessaires par `chat/+page.svelte`)
2. **Garder le changement local** pour `initCryptoListener`
3. Fusionner les deux dans l'objet de ré-export

```typescript
// chatStore.ts (après résolution)
export {
  chatStore,
  messagesStore,
  loadMoreMessages,
  loadMessages,
  sendMessage,
  editMessage,
  deleteMessage,
  sendEmoji,
  toggleEmojiPicker,
  formatTimestamp,
  setActiveConv,
  disconnectWs,
  requestNotificationPermission,
  MAX_BYTES_SERVER,           // ← remote ajouté
  cancelTransfer,             // ← remote ajouté
  triggerDecryptAllIfReady,   // ← remote ajouté
  initCryptoListener,         // ← local ajouté (fix effect_orphan)
} from './chatStore.svelte';
```

## Règle générale

| Fichier | Rôle | Qui modifie quoi |
|---------|------|------------------|
| `chatStore.svelte.ts` | Source de vérité (implémentation) | Dev qui fait la feature/fix |
| `chatStore.ts` | Ré-export pur (provider pattern) | **Toujours sync avec .svelte.ts** |

**Ne jamais** éditer `chatStore.ts` directement pour ajouter/supprimer des exports — toujours modifier `chatStore.svelte.ts` puis copier les exports vers `chatStore.ts`.

## Checklist avant push

```bash
# 1. Vérifier que chatStore.ts ré-exporte TOUT ce que chatStore.svelte.ts exporte
grep -n "^export " frontend/src/lib/chatStore.svelte.ts | grep -v "^export interface" | grep -v "^export type"

# 2. Comparer avec chatStore.ts
grep -A 30 "export {" frontend/src/lib/chatStore.ts

# 3. Si différence → sync manuel AVANT commit
```

## Application Nook

Ce pattern s'applique à tous les stores qui ont un module `.ts` de ré-export :
- `authStore.js` ↔ `authStore.svelte.js` (si pattern adopté)
- `cryptoStore.svelte.ts` → pas de `.ts` wrapper (import direct)
- `webrtc-calls.svelte.ts` → pas de `.ts` wrapper

Seul `chatStore` utilise ce pattern actuellement (héritage historique pour compat imports).