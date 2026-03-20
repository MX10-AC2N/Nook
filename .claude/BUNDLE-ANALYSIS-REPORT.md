# 📦 Bundle Analysis — Nook Frontend

> Généré le : **2026-03-20 12:25 UTC** | Commit : `fef6d44`

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
71 kB  frontend/build/_app/immutable/chunks/Dzdtuv4f.js
27 kB  frontend/build/_app/immutable/nodes/8.GE_oZY_S.js
14 kB  frontend/build/_app/immutable/nodes/6.BMj5A11h.js
14 kB  frontend/build/_app/immutable/nodes/18.B-0XN-np.js
12 kB  frontend/build/_app/immutable/nodes/10.CVsGXQ6n.js
10 kB  frontend/build/_app/immutable/nodes/16.ko9AonqN.js
8 kB  frontend/build/_app/immutable/nodes/3.DsRGzY98.js
8 kB  frontend/build/_app/immutable/chunks/DL_I5HTg.js
```

## DT-01 — Suivi libsodium

> **Statut S39** : libsodium est chargé en dynamic import depuis S37 (fire-and-forget, non bloquant).
> Le chunk (~917 kB) est téléchargé uniquement au premier appel crypto — pas au layout initial.
> Objectif futur : tree-shaking ou WASM partiel pour descendre sous 200 kB.

ℹ️ Taille dans les limites attendues (dynamic import actif).
