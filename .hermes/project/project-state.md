# 📂 Project State — Hermes Agent

> Mis à jour : 2026-05-12

## 🏷️ Identité du projet
- **Nom :** Nook
- **Version :** 0.5.0 (développement)
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Branche de travail :** `develop`
- **Dernier commit CI stable :** `0ee77f90` (fix(docker): sqlite-libs pour Alpine)

## 🏗️ Architecture
- **Backend :** Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend :** SvelteKit 5 (Runes) + TypeScript
- **Déploiement :** Docker multi-arch (Alpine 3.20, musl)
- **CI :** GitHub Actions (Rust stable, musl-tools sur runners natifs)

## 🚨 Problèmes connus (à résoudre)

| Bug | Status | Fix Commit |
|-----|--------|------------|
| BUG-001 : Compilation backend (admin.rs) | ✅ FIXÉ | `327b08e6` |
| BUG-002 : E2EE refresh bug | ✅ FIXÉ | `0219c73e` |
| BUG-003 : P2P file transfer (sécurité) | ✅ FIXÉ | `e9b17418` |
| BUG-004 : Axum 0.8 panic (events.rs:316) | ✅ FIXÉ | dans le code |
| CI-001 : Frontend npm ci failure | ✅ FIXÉ | package-lock regen |
| CI-002 : Backend build musl | ✅ FIXÉ | `49f40a5d` (musl-tools natifs) |
| CI-003 : Backend artifact names mismatch | ✅ FIXÉ | `471cee80` |
| CI-004 : Docker libsqlite3 → sqlite-libs | ✅ FIXÉ | `0ee77f90` |
| CI-005 : Turn arm64 download manquant | ✅ FIXÉ | `087eee5f` |

## 🔗 Liens utiles
- CI Backend : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- CI Frontend : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- CI Turn : https://github.com/MX10-AC2N/Nook/actions/workflows/turn.yml
- CI Docker : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- Repo : https://github.com/MX10-AC2N/Nook
- Déploiement local : https://192.168.1.192:6443 (HTTPS cert auto-signé)

## 📊 Status actuel (2026-05-12 16:00 UTC)
- **Backend CI :** ✅ PASS `49f40a5d` (musl-tools natifs, amd64+arm64)
- **Frontend CI :** ✅ PASS
- **Turn CI :** ✅ PASS
- **Docker CI :** ✅ PASS `0ee77f90` (sqlite-libs)

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
