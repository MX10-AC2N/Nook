# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-03-20 13:39 UTC** | Commit : `f5229dd`

## Résumé

| Métrique | Valeur |
|----------|--------|
| JS total | **1438 kB** |
| Chunk libsodium | **917 kB** (HEavZsIZ.js (plus gros chunk)) |
| Alerte DT-01 | 🟢 OK |

## Top 10 chunks par taille

```
917 kB  frontend/build/_app/immutable/chunks/HEavZsIZ.js
225 kB  frontend/build/_app/immutable/chunks/D5ra78x3.js
71 kB  frontend/build/_app/immutable/chunks/U0bKhbFC.js
60 kB  frontend/build/_app/immutable/nodes/8.BioQ6pD4.js
14 kB  frontend/build/_app/immutable/nodes/6.A3Erbknt.js
14 kB  frontend/build/_app/immutable/nodes/18.zPkQ08GL.js
12 kB  frontend/build/_app/immutable/nodes/10.CCQXOkAD.js
10 kB  frontend/build/_app/immutable/nodes/16.B4ubdRNP.js
8 kB  frontend/build/_app/immutable/nodes/3.DFmdoLwO.js
8 kB  frontend/build/_app/immutable/chunks/MXttWAPC.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
