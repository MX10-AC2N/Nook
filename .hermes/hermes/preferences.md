# 🎯 Préférences Utilisateur — Nook

> Ce fichier résume les attentes et préférences de l'utilisateur
> Source : memory tool (user profile)

## 🗣️ Communication

- **Langue :** Français (communique toujours en français)
- **Projet :** Nook — messagerie familiale self-hostée
  - Backend : Rust/Axum
  - Frontend : SvelteKit 5 + SQLite
  - Déploiement : Zimaboard ARM64

## 📋 Style de travail

### Attitudes attendues
- ✅ **"Soit méticuleux"** — être minutieux dans l'analyse et les fixes
- ✅ **"On continue"** — garder le momentum et avancer directement
- ✅ **"On reprends"** — repartir de zéro propre si nécessaire
- ✅ **Action directe** — créer les PRs toi-même, pas d'instructions
- ✅ **Rapport des logs CI bruts** → diagnostic + push direct

### Approche de développement
- ✅ **Vérification pré-commit OBLIGATOIRE** (surtout version rand crate)
- ✅ **REAL bug fixes** pas de simplification de tests
  - *"Le but n'est pas de simplifier, mais de développer"*
- ✅ **Plusieurs erreurs CI** → tout fixer d'un coup et vérifier chaque fix
- ✅ **Utiliser SVG icons** pas d'emojis
- ✅ **Syntaxe Svelte 5 :** `$derived.by`, `{#if}` pas `{if}`

## 🔧 Préférences techniques

### Rust / Backend
- **rand 0.9 :** `rng()` pas `thread_rng()`, `distr::` pas `distributions::`
- **Axum 0.8 :** `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **CI :** Commit d'abord, puis rebase pour lock updates
- **Docker.yml :** Nécessite Backend.yml déclenché en premier

### Svelte / Frontend
- **Svelte 5 Runes :** Respecter `$state`, `$derived`, `$effect`
- **Stores :** Pas de réassignation directe d'objets `$state`
- **MCP Svelte :** Obligatoire avant toute intervention sur du code Svelte

### CI/CD
- **Pas de modification des versions** dans les commits de fix
- **Un commit de fix ne touche QUE le bug signalé**
- **Partager les logs CI bruts** pour diagnostic rapide

## 🚨 Points de vigilance

1. **Jamais modifier les versions des dépendances** dans les commits de fix
2. **Toujours vérifier cargo check** avant push (ou déléguer à Claude Code)
3. **Lire le contexte** (BUGS.md, memory, active-session) à chaque début
4. **Préserver l'historique** — archiver, ne pas supprimer

## 📂 Attentes sur .claude

- ✅ **Intégrer .claude comme MON répertoire** (Hermes Agent)
- ✅ **Avoir toutes les infos systématiquement** (pas de pertes de mémoire)
- ✅ **Restructurer selon MES besoins** pour conserver l'important
- ✅ **Séparer :** espace perso (`hermes/`), références (`reference/`), archives (`archive/`)

## 🔗 Comptes & Accès

- **Compte test :** hermes-bot / Hermes2026!
- **URL locale :** https://192.168.1.192:6443
- **GitHub Token :** Dans le fichier `hermes-memory.md` (pas ici pour sécurité)
