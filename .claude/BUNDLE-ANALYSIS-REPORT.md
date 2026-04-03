# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-04-03 06:07 UTC** | Commit : `bfdf7cd5`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1452 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
225 kB  frontend/build/_app/immutable/chunks/D5ra78x3.js
74 kB  frontend/build/_app/immutable/chunks/ByEBcqM_.js
60 kB  frontend/build/_app/immutable/nodes/8.C7qK-gE5.js
16 kB  frontend/build/_app/immutable/nodes/6.CvyYfORZ.js
16 kB  frontend/build/_app/immutable/nodes/10.CwsKAw4-.js
14 kB  frontend/build/_app/immutable/nodes/17.B7sSEGEI.js
13 kB  frontend/build/_app/immutable/nodes/5.xBtDDtoG.js
12 kB  frontend/build/_app/immutable/chunks/ClWcLenB.js
10 kB  frontend/build/_app/immutable/nodes/15.CYpOiPG5.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
