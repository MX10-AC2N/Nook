# 📂 Project State — Hermes Agent

> Mis à jour : 2026-04-27

## 🏷️ Identité du projet
- **Nom :** Nook
- **Version :** 0.5.0 (développement)
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Branche de travail :** `develop`
- **Dernier commit :** `327b08e6` (fix admin.rs map_err)

## 🏗️ Architecture
- **Backend :** Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend :** SvelteKit 5 (Runes) + TypeScript
- **Déploiement :** Docker multi-arch (Alpine 3.21)
- **CI :** GitHub Actions (Rust nightly, see Backend.yml line 34)

## 🔧 Outils CI disponibles
- **Backend.yml** : Build amd64/arm64 avec Rust nightly
- **Frontend.yml** : Build SvelteKit
- **Docker.yml** : Build & push image multi-arch
- **E2E tests** : Playwright (voir test-nook.yml)

## 🚨 Problèmes connus (à résoudre)
1. ❌ **Compilation backend** - admin.rs map_err (EN COURS)
2. ⚠️ **P2P file transfer** - limité à 1-to-1 (pas de groupes)
3. ⚠️ **E2EE refresh bug** - messages chiffrés visibles après refresh
4. ⚠️ **simple-peer 9.11.1** - non maintenu (frontend WebRTC)

## 🔗 Liens utiles
- CI Backend : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- CI Docker : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- Repo : https://github.com/MX10-AC2N/Nook
- Déploiement local : https://192.168.1.192:6443 (HTTPS cert auto-signé)

## 📊 Status actuel
- **Backend CI :** À vérifier (après commit 327b08e6)
- **Frontend :** Compilation OK (163/163 E2E tests pass)
- **Docker :** À vérifier
- **P2P File Transfer :** Code présent, à tester
## 🔑 Informations critiques

- **GITHUB_TOKEN :** Stocké dans l'outil mémoire (memory tool)
- **Compte test :** hermes-bot / Hermes2026!
- Rust nightly utilisé en CI
- rand 0.9 : `rng()` pas `thread_rng()`
- Axum 0.8 : `{param}` pas `:param`, `Utf8Bytes` pas `String`
