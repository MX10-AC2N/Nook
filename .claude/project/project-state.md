# 📂 Project State — Hermes Agent

> Mis à jour : 2026-04-27 (session 52)

## 🏷️ Identité du projet
- **Nom :** Nook
- **Version :** 0.5.0 (développement)
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Branche de travail :** `develop`
- **Dernier commit :** `01c9d842` (docs: BUG-002 E2EE refresh FIXED)

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

| Bug | Status | Fix Commit |
|-----|--------|------------|
| BUG-001 : Compilation backend (admin.rs) | ✅ FIXÉ | `327b08e6` |
| BUG-002 : E2EE refresh bug | ✅ FIXÉ | `0219c73e` |
| BUG-003 : P2P file transfer (sécurité) | ✅ FIXÉ | `e9b17418` |
| BUG-003 : P2P file transfer (tests) | 🔵 EN COURS | `a35f7989` (tests créés) |

## 🔗 Liens utiles
- CI Backend : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- CI Docker : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- Repo : https://github.com/MX10-AC2N/Nook
- Déploiement local : https://192.168.1.192:6443 (HTTPS cert auto-signé)

## 📊 Status actuel
- **Backend CI :** ✅ PASSE (commit `327b08e6`)
- **Frontend :** ✅ Compilation OK (163/163 E2E tests pass)
- **Docker :** ✅ Image build & push
- **P2P File Transfer :** ✅ Sécurité fixée, tests créés, à tester en réel
- **E2EE refresh :** ✅ Fixé (polling robuste `chatStore.svelte.ts`)

## 🔑 Informations critiques

- **GITHUB_TOKEN :** Stocké dans l'outil mémoire (memory tool)
- **Compte test :** hermes-bot / Hermes2026!
- **Rust nightly** utilisé en CI
- **rand 0.9 :** `rng()` pas `thread_rng()`
- **Axum 0.8 :** `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **simple-peer 9.11.1 :** Dépendance obsolète (voir PR #28 `refactor/remove-simple-peer`)

## 📝 Ce qui a été fait cette session (52)

1. ✅ **Fix sécurité P2P** (commit `e9b17418`)
   - `e2ee.ts` : export de l'instance `e2ee`
   - `file-transfer.svelte.ts` : `getGroupKey()` utilise `e2ee.loadGroupKey(convoId)`
   - **CRITICAL FIX** : Plus de clé dérivée insécure

2. ✅ **Tests P2P créés** (commit `a35f7989`)
   - `frontend/tests/p2p-file-transfer.spec.ts`
   - Tests de base pour vérifier la sécurité

3. ✅ **BUG-002 E2EE refresh** marqué FIXED (commit `01c9d842`)
   - Vérification : `_decryptAllIfReady()` appelé APRÈS chargement messages
   - Polling robuste qui ne s'arrête plus à la première tentative

4. ✅ **Structure `.claude/`** (commits précédents)
   - Espace `hermes/` créé (actif au démarrage)
   - Hook de démarrage configuré

## 🔜 Prochaines étapes

1. **Tester P2P file transfer >50 Mo** sur l'homeserver (https://192.168.1.192:6443)
2. **Vérifier E2EE refresh** en conditions réelles (après redéploiement)
3. **simple-peer 9.11.1** : Vérifier si PR #28 est mergé
4. **Créer plus de tests** pour les fonctionnalités critiques
