# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-03-29 05:42 UTC** | Commit : `dc71054`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1445 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
225 kB  frontend/build/_app/immutable/chunks/D5ra78x3.js
71 kB  frontend/build/_app/immutable/chunks/DojMlZzs.js
60 kB  frontend/build/_app/immutable/nodes/8.xchIXdzy.js
16 kB  frontend/build/_app/immutable/nodes/6.D6ktQKk5.js
14 kB  frontend/build/_app/immutable/nodes/10.D3CfvxB-.js
14 kB  frontend/build/_app/immutable/nodes/18.D7wwy1YQ.js
10 kB  frontend/build/_app/immutable/chunks/T5aUK-4h.js
10 kB  frontend/build/_app/immutable/nodes/16.CmtCwxAV.js
8 kB  frontend/build/_app/immutable/nodes/12.Bikulima.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
