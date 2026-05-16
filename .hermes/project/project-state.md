# 📂 Project State — Hermes Agent

> Mis à jour : 2026-05-16

## 🏷️ Identité du projet
- **Nom :** Nook
- **Version :** 0.5.0 (développement)
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Branche de travail :** `develop`
- **Dernier commit CI stable :** `f0a8c8d1` (fix(e2ee/encrypt): try/catch par destinataire dans encryptForRecipients)

## 🏗️ Architecture
- **Backend :** Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend :** SvelteKit 5 (Runes) + TypeScript
- **Déploiement :** Docker multi-arch (Alpine 3.20, musl)
- **CI :** GitHub Actions (Rust stable, musl-tools sur runners natifs) — ordre strict : Frontend → Backend → Turn → Docker

## 🚨 Problèmes connus

| Bug | Status | Fix Commit | Notes |
|-----|--------|------------|-------|
| BUG-001 : Compilation backend (admin.rs) | ✅ FIXÉ | `327b08e6` | Parenthèses `.map_err()` |
| BUG-002 : E2EE refresh bug | ✅ FIXÉ | `0219c73e` | Polling `_decryptAllIfReady` |
| BUG-003 : P2P file transfer sécurité | ✅ FIXÉ | `e9b17418` | `loadGroupKey` |
| BUG-004 : Axum 0.8 panic (events.rs:316) | ✅ FIXÉ | dans le code | |
| BUG-005 : E2EE clé publique désynchronisée | ✅ FIXÉ | `36eefe5c` | `await registerPublicKeyOnServer` avant `ready=true` |
| BUG-006 : E2EE encryptForRecipients casse total si 1 destinataire invalide | ✅ FIXÉ | `f0a8c8d1` | try/catch par destinataire |
| BUG-007 : Frontend npm ci failure | ✅ FIXÉ | package-lock regen | |
| BUG-008 : Backend build musl | ✅ FIXÉ | `49f40a5d` | musl-tools natifs |
| BUG-009 : Turn arm64 download manquant | ✅ FIXÉ | `087eee5f` | |
| CI-001 | ✅ FIXÉ | `0ee77f90` | Docker sqlite-libs |
| CI-002 | ✅ FIXÉ | `471cee80` | Artifact names |
| E2EE anciens messages | ⚠️ STRUCTUREL | N/A | Indéchiffrables après rotation de clé — normal |
| CI-003 | ✅ FIXÉ | `0ee77f90` | Docker libsqlite3 → sqlite-libs Alpine |

### Notes E2EE critiques
- **Rotation de clé** : Si `users.public_key` change entre deux sessions, les anciens messages deviennent indéchiffrables (clé de session chiffrée avec ancienne clé X25519).
- **Fix racine** (`36eefe5c`) : `unlockCrypto` attendait `registerPublicKeyOnServer` en fire-and-forget → `cryptoStore.ready=true` avant synchro.
- **Fix encrypt** (`f0a8c8d1`) : `encryptForRecipients` avait une boucle sans try/catch → un seul destinataire invalide cassait tout le chiffrement → `encrypted_keys:{}`.
- **_FAILED_DECRYPT_IDS** Fix : Les champs E2EE (`encrypted`, `nonce`, `sender_public_key`) ne sont plus mutilés en cas d'échec → re-déchiffrement automatique futur possible.

## 🔗 Liens utiles
- CI Backend : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- CI Frontend : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- CI Turn : https://github.com/MX10-AC2N/Nook/actions/workflows/turn.yml
- CI Docker : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- Repo : https://github.com/MX10-AC2N/Nook
- Test server : http://192.168.1.192:6300 (HTTP) / https://192.168.1.192:6443 (HTTPS nginx local)
- Développement local : NOOK_ENV=development autorise cookies HTTP session

## 📊 Status actuel (2026-05-16)
- **Backend CI :** ✅ PASS (musl, multi-arch stable)
- **Frontend CI :** ✅ PASS
- **Turn CI :** ✅ PASS
- **Docker CI :** ✅ PASS
- **Serveur CasaOS :** 192.168.1.192:6300 — rebuild local `docker compose up -d --build`, tous services healthy
- **E2EE** : Nouveaux messages POST-fix chiffrés correctement (`encrypted_keys` non vide par destinataire traité indépendamment)

## 🔑 Informations critiques

- **Compte test serveur** : hermes-bot / Hermes2026!
- **GITHUB_TOKEN** : /tmp/.git_token (93 chars PAT)
- **NOOK_ENV=development** pour autoriser cookies HTTP session en local
- **Pas de scheduled workflows** — free GitHub account, déclenchements manuels uniquement
- **Ordre workflow strict** : Frontend → Backend → Turn → Docker (Docker seulement après 3 verts)
- **Port 8080** sur 192.168.1.192 = scanservjs, PAS Nook
- **Svelte 5** : éviter `<form onsubmit>` → utiliser `<button type="button" onclick>`
- **Rust 0.9** : `rng()` pas `thread_rng()`, `distr::` pas `distributions::`
- **Axum 0.8** : `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **Clippy** : Obligatoire clean pre-push

## 📝 Commandes utiles

### Tests E2EE
```
# Naviguer sur serveur
open http://192.168.1.192:6300
# Login: hermes-bot / Hermes2026!
# Vérifier console: decryptSessionKey logs, encryptForRecipients console.warn par destinataire
```

### CI Orchestration
```bash
cd /opt/data/Nook
gh run list --limit 5 --json number,headSha,status,workflowName --jq '.[] | [.number, .headSha[:7], .status, .workflowName] | join(" | ")'
# Puis déclencher dans l'ordre :
gh workflow run 2==> Frontend Build & Artifact.yml
gh workflow run 1==> Backend Build & Artifact.yml
gh workflow run 3==> Turn-Server Build and Artifact.yml
# Attendre 3 verts → gh workflow run 4==> Docker Build & Push.yml
```

### Redéploiement CasaOS
```bash
ssh root@192.168.1.192   # (si SSH accessible)
git pull
docker compose down -v --rmi all --remove-orphans
docker compose up -d --build
```

## 🔜 Prochaines étapes
1. Valider E2EE sur serveur CasaOS post-redéploiement — `encrypted_keys` non vide pour nouveaux messages
2. Playwright E2E validation complète sur http://192.168.1.192:6300
3. Nettoyage `_FAILED_DECRYPT_IDS` periodic garbage collection
4. Rotation de clé documentée dans les guides utilisateur
