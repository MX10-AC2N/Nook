# 🔑 Mémoire CORE - Informations Critiques

> **DERNIÈRE MISE À JOUR** : 2026-05-04
> Mon fichier de référence pour les infos critiques Nook

## 🔐 Accès & Authentification

### GitHub
- **Repo** : `https://github.com/MX10-AC2N/Nook` (branche `develop`)
- **Token** : Stocké dans l'outil mémoire (memory tool)
- **Push** : Utiliser Python subprocess avec token dans URL
- **Compte bot** : `hermes-bot` / `Hermes2026!`

### Serveurs & URLs
- **Local (Zimaboard)** : `https://192.168.1.192:6443` (HTTPS cert auto-signé)
- **Docker Registry** : `ghcr.io/mx10-ac2n/nook`
- **Port Dév** : 5173 (SvelteKit)
- **Port Prod** : 6300

## 🏗️ Projet Nook - État

### Version & Stack
- **Version** : 0.5.0 (développement)
- **Backend** : Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend** : SvelteKit 5 (Runes) + TypeScript
- **E2E Tests** : 163/163 PASS (TURN server OK)

### Dernier Commit
- **Hash** : `327b08e6` 
- **Message** : fix admin.rs map_err
- **Date** : 2026-05-03

## 🚀 CI/CD & Workflows

### GitHub Actions
- **Backend.yml** : Build amd64/arm64 avec Rust nightly (ligne 34)
- **Frontend.yml** : Build SvelteKit
- **Docker.yml** : Build & push image multi-arch
- **CI Workflow IDs** : Backend=220018362, Frontend=220018364, Docker=220018363

### Règles CI
- ✅ Commit d'abord, puis rebase pour lock updates
- ✅ Docker.yml nécessite Backend.yml déclenché en premier pour les changements Rust
- ✅ Toujours vérifier les logs CI bruts avant de faire un push

## 📋 Préférences Utilisateur (de memory tool)

### Communication
- **Langue** : Français
- **Style** : "Soit méticuleux" — minutieux dans l'analyse et les fixes
- **Momentum** : "On continue" — garder le momentum et avancer directement
- **Reset** : "On reprends" — repartir de zéro propre si nécessaire

### Attentes & Règles
- ✅ **Vérification pré-commit OBLIGATOIRE** (surtout rand crate version)
- ✅ **Rapport des logs CI bruts** → diagnostic + push direct
- ✅ **Action directe** (créer PRs) plutôt que d'être guidé
- ✅ **REAL bug fixes** pas de simplification de tests
- ✅ **Utiliser SVG icons** pas d'emojis
- ✅ **Syntaxe Svelte 5** : `$derived.by`, `{#if}` pas `{if}`
- ✅ **Plusieurs erreurs CI** → tout fixer d'un coup

## 🔴 Erreurs à ne plus faire

1. **Modification versions dépendances** dans commits de fix
   - J'avais changé `rustrtc` 0.3.40 → 0.3.39 par erreur
   - **Règle** : Un commit de fix ne touche QUE le bug signalé

2. **Syntaxe `.map_err()` incorrecte**
   - ❌ `.map_err(|_| (...))?`
   - ✅ `.map_err(|_| { (...) })?`

3. **Perte de contexte entre sessions**
   - **Solution** : Lire `active-session.md` et `known-issues.md` à chaque début

## 📂 Structure .hermes

```
.hermes/
├── hermes/              # MON espace perso (ce répertoire parent)
│   ├── memory/          # Mémoire organisée (ce dossier)
│   │   ├── core.md      # CE fichier
│   │   ├── rust.md      # Apprentissages Rust
│   │   ├── svelte.md    # Apprentissages Svelte
│   │   ├── devops.md    # CI/CD, Docker
│   │   ├── security.md  # E2EE, auth, WebRTC
│   │   └── sessions/    # Logs de session
│   ├── active-session.md
│   ├── known-issues.md
│   ├── preferences.md
│   └── tools-state.md
```

## 🔗 Liens Rapides

- **CI Backend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **CI Frontend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- **CI Docker** : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- **Repo** : https://github.com/MX10-AC2N/Nook
- **Déploiement** : https://192.168.1.192:6443

---
*Fichier CORE - À mettre à jour dès qu'une info critique change*
