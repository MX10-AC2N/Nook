# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-03-20 12:18 UTC** | Commit : `5bbe66c`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1405 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
225 kB  frontend/build/_app/immutable/chunks/D5ra78x3.js
71 kB  frontend/build/_app/immutable/chunks/WVY2Wwp4.js
27 kB  frontend/build/_app/immutable/nodes/8.CP_W-HJL.js
14 kB  frontend/build/_app/immutable/nodes/6.D72D77-g.js
14 kB  frontend/build/_app/immutable/nodes/18.BzDrJRia.js
12 kB  frontend/build/_app/immutable/nodes/10.BLXNI5KU.js
10 kB  frontend/build/_app/immutable/nodes/16.DlkE7AtL.js
8 kB  frontend/build/_app/immutable/nodes/3.sa4Hdquy.js
8 kB  frontend/build/_app/immutable/chunks/BTDE_aYy.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
