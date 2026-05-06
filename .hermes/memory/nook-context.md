# 🧠 Nook Context — Hermes Extended Memory

> Fichier de référence structuré pour compléter l'outil `memory`.
> Mis à jour : 2026-05-06

## 📊 État actuel (2026-05-06 15:00 UTC)

- **Frontend CI** : ✅ PASSE (commit `25442320465`)
- **Backend CI** : 🔴 EN COURS (zigbuild, run 25442974059)
- **Turn CI** : ✅ PASSE (commit `abb11c7e`)
- **Docker CI** : ⏳ EN ATTENTE
- **Deployed** : 🔴 Unhealthy (Axum 0.8 panic events.rs:316)

## 🔑 Informations critiques (rappel)

- **GITHUB_TOKEN** : Dans tool `memory` (github: Token valid)
- **Compte test** : hermes-bot / Hermes2026!
- **Repo URL** : https://github.com/MX10-AC2N/Nook
- **Déploiement local** : http://192.168.1.192:6300 | https://192.168.1.192:6443

## 🛠️ Stack technique & Pièges

### Backend (Rust)
- **Axum 0.8** : `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **rand 0.9** : `rng()` pas `thread_rng()`
- **SQLx** : `ENVELOPE_SELECT` alias, `query_as!` macro
- **Compiler** : Désormais `zig` + `cargo-zigbuild` (Backend.yml)
- **Cibles** : `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`

### Frontend (Svelte 5)
- **Runes** : `$state`, `$derived`, `$derived.by`, `$effect`
- **MCP Svelte** : Obligatoire avant livraison
- **Pas de réassignation** directe sur `$state`
- **package-lock.json** : Régénérer avec `npm install` si mismatch

### CI/CD
- **Ordre workflows** : Frontend → Backend → Turn → Docker
- **Jamais** de scheduled workflows (compte GitHub gratuit)
- **Vérifier** `git log --oneline -5` et `gh run list --limit 5` avant de lancer

## 🐛 Bugs connus et historique

| ID | Description | Status | Commit | Solution |
|----|-------------|--------|--------|----------|
| BUG-001 | Compilation backend (admin.rs) | ✅ FIXÉ | `327b08e6` | Correction syntaxe |
| BUG-002 | E2EE refresh bug | ✅ FIXÉ | `0219c73e` | Polling robuste |
| BUG-003 | P2P file transfer (sécurité) | ✅ FIXÉ | `e9b17418` | `getGroupKey()` fix |
| BUG-004 | Axum 0.8 panic (events.rs:316) | 🔵 EN COURS | `7562f847` | Passage stable + zig |
| CI-001 | Frontend npm ci failure | ✅ FIXÉ | `25442320465` | package-lock regen |
| CI-002 | Turn Dockerfile syntax | ✅ FIXÉ | `abb11c7e` | `&&` fix |
| CI-003 | Backend nightly rustc target | ✅ FIXÉ | `7562f847` | Switch stable + zig |

## 📝 Commandes utiles

### Dev local
```bash
# Installer tous les outils manquants
bash .hermes/tools/install-dev-tools.sh

# Backend (cargo zigbuild)
cd backend && cargo zigbuild --release --target x86_64-unknown-linux-musl

# Frontend
cd frontend && npm install && npm run build
```

### Git (conventions)
```bash
# Commit atomique (un fix = un commit)
git add <specific files>
git commit -m "fix(scope): description"

# Ne jamais modifier les versions de dépendances dans un commit de fix
# Un commit de fix ne touche QUE le bug signalé
```

### CI (GitHub Actions)
```bash
# Lancer workflows dans l'ordre
gh workflow run Frontend.yml --ref develop
gh workflow run Backend.yml --ref develop
gh workflow run turn.yml --ref develop
gh workflow run Docker.yml --ref develop

# Vérifier état
gh run list --limit 5 --branch develop
```

## 🔗 Liens rapides

- **CI Backend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **CI Frontend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- **CI Turn** : https://github.com/MX10-AC2N/Nook/actions/workflows/turn.yml
- **CI Docker** : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- **Repo** : https://github.com/MX10-AC2N/Nook
- **Déploiement** : https://192.168.1.192:6443

## 📂 Structure .hermes/ (optimisée)

```
.hermes/
├── hermes/              # Espace perso (active-session, known-issues, preferences)
├── project/             # Référence projet (project-state.md, BUGS.md)
├── reference/           # Patterns (rust-patterns.md, svelte-patterns.md)
├── memory/              # MÉMOIRE ÉTENDUE (nook-context.md, hermes-memory.md)
├── tools/               # Outils (install-dev-tools.sh)
├── skills/              # Skills existants
├── roles/               # Rôles agents
├── rules/               # Règles orchestration
├── workflows/           # Workflows GitHub
├── archive/             # Archives (rapports anciens)
├── reports/             # Rapports récents (BUILD-REPORT, etc.)
├── workspace/           # Espace de travail (scripts, tests)
└── SOUL.md, CLAUDE.md  # Orchestrateurs
```

## 🔜 Prochaines étapes (après CI)

1. **Tester déploiement** (http://192.168.1.192:6300)
2. **Fix Axum 0.8 panic** (events.rs:316) si toujours présent
3. **Optimiser .hermes/workspace/** (scripts Python, tests E2E)
4. **Sécuriser mémorisation** (backup .hermes/ sur repo)
