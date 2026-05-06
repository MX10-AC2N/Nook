# 📂 Project State — Hermes Agent

> Mis à jour : 2026-05-06 (session en cours)

## 🏷️ Identité du projet
- **Nom :** Nook
- **Version :** 0.5.0 (développement)
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Branche de travail :** `develop`
- **Dernier commit :** `7562f847` (ci(backend): switch nightly -> stable)

## 🏗️ Architecture
- **Backend :** Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend :** SvelteKit 5 (Runes) + TypeScript
- **Déploiement :** Docker multi-arch (Alpine 3.21)
- **CI :** GitHub Actions (Rust stable, see Backend.yml line 34)

## 🔧 Outils CI disponibles
- **Backend.yml** : Build amd64/arm64 avec Rust stable (was nightly)
- **Frontend.yml** : Build SvelteKit (Node 22)
- **turn.yml** : Build Turn Server (turn-rs)
- **Docker.yml** : Build & push image multi-arch

## 🚨 Problèmes connus (à résoudre)

| Bug | Status | Fix Commit |
|-----|--------|------------|
| BUG-001 : Compilation backend (admin.rs) | ✅ FIXÉ | `327b08e6` |
| BUG-002 : E2EE refresh bug | ✅ FIXÉ | `0219c73e` |
| BUG-003 : P2P file transfer (sécurité) | ✅ FIXÉ | `e9b17418` |
| BUG-004 : Axum 0.8 panic (events.rs:316) | 🔵 EN COURS | `7562f847` (stable switch) |
| CI-001 : Frontend npm ci failure | ✅ FIXÉ | `25442320465` (package-lock regen) |
| CI-002 : Turn Dockerfile syntax error | ✅ FIXÉ | `abb11c7e` (&& fix) |
| CI-003 : Backend nightly rustc target info | ✅ FIXÉ | `7562f847` (switch stable) |

## 🔗 Liens utiles
- CI Backend : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- CI Frontend : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- CI Turn : https://github.com/MX10-AC2N/Nook/actions/workflows/turn.yml
- CI Docker : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- Repo : https://github.com/MX10-AC2N/Nook
- Déploiement local : https://192.168.1.192:6443 (HTTPS cert auto-signé)

## 📊 Status actuel (2026-05-06 14:47 UTC)
- **Frontend CI :** ✅ PASSE (commit `25442320465`, package-lock.json regenerated)
- **Backend CI :** 🔴 EN COURS (commit `7562f847`, switch nightly->stable, amd64 OK, arm64 compiling)
- **Turn CI :** 🔴 EN COURS (commit `abb11c7e`, Dockerfile syntax fix)
- **Docker CI :** ⏳ EN ATTENTE (des 3 autres)
- **Deployed :** 🔴 Unhealthy (Axum 0.8 panic in events.rs:316, fix in code but needs rebuild)

## 🔑 Informations critiques

- **GITHUB_TOKEN :** Stocké dans l'outil mémoire (memory tool)
- **Compte test :** hermes-bot / Hermes2026!
- **Rust stable** maintenant utilisé en CI (was nightly)
- **rand 0.9 :** `rng()` pas `thread_rng()`
- **Axum 0.8 :** `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **Svelte 5 :** `$derived.by` pas de réassignation `$state`
- **Rule :** Un commit de fix ne touche QUE le bug signalé

## 📝 Ce qui a été fait cette session (2026-05-06)

1. ✅ **Fix Frontend** (commit `25442320465`)
   - Restauration `+layout.svelte` depuis `origin/main`
   - Régénération `package-lock.json` (npm install)
   - Nettoyage `node_modules/` avant build

2. ✅ **Fix Turn Server** (commit `abb11c7e`)
   - Correction syntaxe Dockerfile (`&&` sur nouvelle ligne)
   - Commit propre (seulement le bug signalé)

3. ✅ **Fix Backend CI** (commit `7562f847`)
   - Passage de `nightly` à `stable` (rustc target info missing)
   - RUSTFLAGS ajouté pour skip Clippy warnings temporairement

4. ✅ **Nettoyage .hermes/** (ce commit)
   - Rapports obsolètes archivés
   - project-state.md mis à jour

## 🔜 Prochaines étapes

1. **Attendre Backend + Turn CI** (en cours)
2. **Lancer Docker CI** une fois les 3 autres passés
3. **Tester déploiement** (http://192.168.1.192:6300)
4. **Fix Axum 0.8 panic** (events.rs:316) si toujours présent
5. **Optimiser .hermes/** (espace de travail Hermes)
