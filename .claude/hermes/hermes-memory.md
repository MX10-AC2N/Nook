# 🤖 Hermes Memory — Nook Workspace

> Mis à jour : 2026-04-27
> C'est MON fichier de référence perso en tant qu'agent Hermes

## 🔑 Informations critiques (à ne jamais oublier)

### GitHub & Accès
- **GITHUB_TOKEN :** Voir dans l'outil mémoire (memory tool)
- **Repo :** `https://github.com/MX10-AC2N/Nook` (branche `develop`)
- **Compte test :** `hermes-bot` / `Hermes2026!`
- **URL locale :** `https://192.168.1.192:6443` (HTTPS cert auto-signé)

### CI & Workflows
- **Backend.yml :** Build amd64/arm64 avec **Rust nightly** (ligne 34)
- **Frontend.yml :** Build SvelteKit
- **Docker.yml :** Build & push image multi-arch
- **CI Workflow IDs :** Backend=220018362, Frontend=220018364, Docker=220018363
- **Règle CI :** Commit d'abord, puis rebase pour lock updates
- **Docker.yml :** Nécessite Backend.yml déclenché en premier pour les changements Rust

## 🏗️ Projet Nook — État

- **Version :** 0.5.0 (développement)
- **Backend :** Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend :** SvelteKit 5 (Runes) + TypeScript
- **E2E Tests :** 163/163 PASS (TURN server OK)
- **Dernier commit :** `327b08e6` (fix admin.rs map_err)

## 📋 Préférences Utilisateur (de memory tool)

### Communication
- **Langue :** Français
- **Style :** "Soit méticuleux" — minutieux dans l'analyse et les fixes
- **Momentum :** "On continue" — garder le momentum et avancer directement
- **Reset :** "On reprends" — repartir de zéro propre si nécessaire

### Attentes & Règles
- ✅ **Vérification pré-commit OBLIGATOIRE** (surtout rand crate version)
- ✅ **Rapport des logs CI bruts** → diagnostic + push direct
- ✅ **Action directe** (créer PRs) plutôt que d'être guidé
- ✅ **REAL bug fixes** pas de simplification de tests ("le but n'est pas de simplifier, mais de développer")
- ✅ **Utiliser SVG icons** pas d'emojis
- ✅ **Syntaxe Svelte 5 :** `$derived.by`, `{#if}` pas `{if}`
- ✅ **Plusieurs erreurs CI** → tout fixer d'un coup et vérifier chaque fix avant push

### Patterns de code
- **rand 0.9 :** `rng()` pas `thread_rng()`, `distr::` pas `distributions::`
- **Axum 0.8 :** `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **CORS :** allowed_origins explicites, pas `Any` avec credentials

## 🔴 Erreurs commises (à ne plus faire)

1. **Modification versions dépendances** dans commits de fix
   - J'avais changé `rustrtc` 0.3.40 → 0.3.39 par erreur
   - **Règle :** Un commit de fix ne touche QUE le bug signalé
   
2. **Perte de contexte** entre sessions
   - **Solution :** Ce fichier `.claude/hermes/` + restructurer tout le dossier
   - **Action :** Lire `active-session.md` et `known-issues.md` à chaque début

3. **Syntaxe `.map_err()` incorrecte**
   - ❌ `.map_err(|_| (...))?`
   - ✅ `.map_err(|_| { (...) })?`

## 📂 Structure .claude actuelle (restructurée)

```
.claude/
├── hermes/                    # MON espace perso
│   ├── active-session.md      # Ce que je fais MAINTENANT
│   ├── hermes-memory.md       # CE fichier (infos critiques)
│   ├── known-issues.md        # Bugs, pièges, leçons
│   └── preferences.md         # Préférences utilisateur
├── project/                   # Référence projet
│   └── project-state.md      # État actuel Nook
├── reference/                 # Références rapides
│   ├── rust-patterns.md      # Rust/Axum/SQLx patterns
│   └── svelte-patterns.md    # Svelte 5 Runes patterns
├── archive/                   # Archives (préserver !)
│   ├── reports/             # Anciens rapports
│   └── sessions/            # Sessions historiques
├── skills/                    # Skills existants (garder)
├── roles/                     # Rôles existants (garder)
├── rules/                     # Règles existantes (garder)
└── workflows/                 # Workflows existants (garder)
```

## 🧠 Ce que je dois faire à chaque session

1. ✅ Lire `.claude/hermes/active-session.md` (état immédiat)
2. ✅ Lire `.claude/hermes/known-issues.md` (ne pas répéter les erreurs)
3. ✅ Vérifier `.claude/project/project-state.md` (version, branche)
4. ✅ Consulter `.claude/reference/` selon besoin (Rust ou Svelte)
5. ✅ Mettre à jour `active-session.md` après chaque action importante
6. ✅ Respecter les préférences utilisateur listées ci-dessus

## 🔗 Liens rapides

- **CI Backend :** https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **CI Frontend :** https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- **CI Docker :** https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Déploiement :** https://192.168.1.192:6443

## 📝 Notes de session actuelle

- Commit `327b08e6` poussé (fix admin.rs)
- CI en attente de vérification par l'utilisateur
- Structure `.claude` en cours de restructuration (27-04-2026)
- Objectif : ne plus perdre le fil entre les sessions
