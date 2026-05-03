# 🔑 Mémoire Core - Informations Critiques

> Dernière mise à jour: 2026-05-03
> À lire ABSOLUMENT à chaque session

## 🔐 GitHub & Accès

- **Repo** : `https://github.com/MX10-AC2N/Nook`
- **Branche** : `develop`
- **Compte test** : `hermes-bot` / `Hermes2026!`
- **URL locale** : `https://192.168.1.192:6443` (HTTPS cert auto-signé)
- **Raw base** : `https://raw.githubusercontent.com/MX10-AC2N/Nook/develop/`

## 🔧 Outils Essentiels

### Disponibles
- ✅ `git` (configuré: Hermes Bot <hermes-bot@nook.app>)
- ✅ `node` / `npm`
- ✅ `jq`
- ✅ `curl`
- ✅ `make`
- ✅ `gcc` / `g++`
- ✅ `pkg-config`
- ✅ `rustc` (1.85.0 nightly requis pour Nook)
- ✅ `cargo` (1.85.0)
- ✅ `gh` (2.46.0) - nécessite auth manuelle via token

### À installer (si manquants)
- Voir `../tools-state.md` pour l'état complet

## 📦 Versions Projet Nook

- **Version** : 0.5.0 (développement)
- **Backend** : Rust + Axum 0.8 + SQLx 0.8.6 + SQLite
- **Frontend** : SvelteKit 5 (Runes) + TypeScript
- **Dernier audit** : 75.4/100 (Session 53, 2026-04-28)

## 🚀 CI/CD Workflows

| Workflow | ID | Fichier | Déclenchement |
|----------|-----|---------|----------------|
| Backend | 220018362 | `.github/workflows/Backend.yml` | Manulement (push) |
| Frontend | 220018364 | `.github/workflows/Frontend.yml` | Manulement (push) |
| Turn | 257238341 | `.github/workflows/Turn.yml` | Manulement (push) |
| Docker | 220018363 | `.github/workflows/Docker.yml` | Après F/E/T success |

**Règle CI** : Backend + Frontend + Turn simultanément → attendre tous succès → Docker

**Commande déclenchement** :
```bash
gh workflow run <ID> --ref develop
```

## 📋 Commandes Git Critiques

```bash
# Naviguer vers le repo
cd /opt/data/home/.hermes/Nook

# Vérifier état
git status
git log --oneline -5

# Commit et push (avec token dans remote)
git add .
git commit -m "feat/fix: description"
git push origin develop
```

## 🔴 ERREURS CRITIQUES À NE PLUS FAIRE

1. **Modifier les versions dépendances dans commits de fix**
   - ✅ Fix unique = bug signalé uniquement
   - ❌ Pas de changement `rustrtc` ou autres versions

2. **Perdre le contexte entre sessions**
   - ✅ Lire `active-session.md` au début
   - ✅ Consulter `known-issues.md`
   - ✅ Vérifier `memory/core.md` (ce fichier)

3. **Syntaxe `.map_err()` incorrecte**
   - ❌ `.map_err(|_| (...))?`
   - ✅ `.map_err(|_| { (...) })?`

4. **Oublier vérification pré-commit**
   - ✅ Toujours vérifier `rand` crate version (0.9+)
   - ✅ `rng()` pas `thread_rng()`, `distr::` pas `distributions::`

## 📂 Structure Rapide

```
.hermes/
├── hermes/          # MON espace (lu en premier)
├── skills/          # Skills (automatisé)
├── roles/           # Rôles agents
├── rules/           # Règles projet
└── tools/           # Scripts et références
```

---
*Éditer ce fichier avec toute info critique nouvelle*
