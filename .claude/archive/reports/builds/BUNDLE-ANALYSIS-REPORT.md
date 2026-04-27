# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-04-13 13:41 UTC** | Commit : `fd097905`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1480 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
195 kB  frontend/build/_app/immutable/chunks/UxwqlMfu.js
74 kB  frontend/build/_app/immutable/chunks/CrwuEMVs.js
63 kB  frontend/build/_app/immutable/nodes/8.ClQerUfx.js
30 kB  frontend/build/_app/immutable/chunks/BiJu5QHs.js
17 kB  frontend/build/_app/immutable/nodes/5.Bw6_wRzQ.js
17 kB  frontend/build/_app/immutable/nodes/10.BZ7sBMAH.js
14 kB  frontend/build/_app/immutable/nodes/17.BGoovB9B.js
13 kB  frontend/build/_app/immutable/chunks/B21V1eOL.js
12 kB  frontend/build/_app/immutable/chunks/30-BkH1W.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
