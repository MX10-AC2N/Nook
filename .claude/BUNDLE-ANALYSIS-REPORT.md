# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-03-29 05:44 UTC** | Commit : `98ed8b1`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1446 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
225 kB  frontend/build/_app/immutable/chunks/D5ra78x3.js
71 kB  frontend/build/_app/immutable/chunks/DGYtKmzH.js
60 kB  frontend/build/_app/immutable/nodes/8.CParL4tD.js
17 kB  frontend/build/_app/immutable/nodes/6.CZOsaphh.js
14 kB  frontend/build/_app/immutable/nodes/10.CSUVtrLm.js
14 kB  frontend/build/_app/immutable/nodes/18.CiImIhLG.js
10 kB  frontend/build/_app/immutable/chunks/C069ctwH.js
10 kB  frontend/build/_app/immutable/nodes/16.CcBjJFyY.js
8 kB  frontend/build/_app/immutable/nodes/12.DyaGidZ2.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
