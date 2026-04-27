# 🔧 Commandes Nook — Référence Complète

> Fichier de référence pour toutes les commandes spécifiques au projet Nook
> Mis à jour : 2026-04-27
> Usage : Consulter ce fichier AVANT de lancer une commande complexe

---

## 📋 Table des matières
1. [Git & GitHub](#git--github)
2. [Docker & Déploiement](#docker--déploiement)
3. [Backend (Rust/Axum)](#backend-rustaxum)
4. [Frontend (SvelteKit)](#frontend-sveltekit)
5. [Tests E2E (Playwright)](#tests-e2e-playwright)
6. [Base de données (SQLite/SQLx)](#base-de-données-sqlitesqlx)
7. [CI/CD (GitHub Actions)](#cicd-github-actions)
8. [Outils Hermes Agent](#outils-hermes-agent)
9. [WebRTC & Appels](#webrtc--appels)
10. [Dépannage courant](#dépannage-courant)

---

## 🔀 Git & GitHub

### Configuration initiale
```bash
# Configurer l'identité pour les commits
git config user.email "hermes-bot@users.noreply.github.com"
git config user.name "hermes-bot"

# Configurer l'URL distante avec token (PAS de token en clair dans l'historique!)
git remote set-url origin https://hermes-bot:<GITHUB_TOKEN>@github.com/MX10-AC2N/Nook.git
```

### Workflow de développement (RÈGLE : commit d'abord, puis rebase)
```bash
# 1. Vérifier l'état
git status
git diff

# 2. Ajouter et commiter
git add <fichiers>
git commit -m "type(scope): description"

# 3. Récupérer et rebase (NE JAMAIS push direct sans sync)
git pull --rebase origin develop
git push origin develop
```

### Types de commits (Conventional Commits)
- `feat:` nouvelle fonctionnalité
- `fix:` correction de bug
- `refactor:` refactoring
- `docs:` documentation
- `test:` tests
- `ci:` CI/CD
- `chore:` maintenance

### GitHub CLI (si disponible)
```bash
# Lister les runs CI
gh run list --workflow=Backend.yml --limit 3

# Voir un run spécifique
gh run view <run-id>

# Déclencher manuellement
gh workflow run Backend.yml
```

---

## 🐳 Docker & Déploiement

### Construction et démarrage
```bash
# Construire l'image (depuis la racine Nook)
docker build -t nook:latest .

# Démarrer avec docker-compose
docker-compose up -d

# Voir les logs
docker-compose logs -f

# Arrêter
docker-compose down
```

### Multi-architecture (GHCR)
```bash
# Build pour amd64 et arm64 (Zimaboard)
docker buildx create --name multiarch --driver docker-container --use
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/mx10-ac2n/nook:latest \
  --push .
```

### HTTPS local (nginx-alpine :6443)
```bash
# Tester le certificat auto-signé
curl -k https://192.168.1.192:6443

# Dans le navigateur : Accepter l'avertissement "Site non sécurisé"
# → Clic "Avancé" → "Continuer vers 192.168.1.192 (non sécurisé)"
```

---

## 🦀 Backend (Rust/Axum)

### Outillage
```bash
# Installer Rust nightly (utilisé en CI)
rustup toolchain install nightly
rustup default nightly

# Vérifier la version
rustc --version
cargo --version
```

### Compilation et vérification
```bash
# Aller dans le dossier backend
cd /tmp/Nook/backend

# Vérification rapide (OBLIGATOIRE avant push)
cargo check --all-targets

# Linter (OBLIGATOIRE avant push)
cargo clippy -- -D warnings

# Build release (pour déploiement)
cargo build --release --locked

# Formatage
cargo fmt --all
```

### SQLite (sqlx)
```bash
# Préparer les requêtes offline (après nouvelle macro sqlx!)
cargo sqlx prepare --workspace

# Vérifier la base
sqlite3 nook.db ".tables"
sqlite3 nook.db "SELECT * FROM users LIMIT 5;"
```

### Variables d'environnement critiques
```bash
# Requis pour cargo check/clippy en CI
export SQLX_OFFLINE=true
export DATABASE_URL="sqlite://nook.db"

# Requis pour build release
export CARGO_PROFILE_RELEASE_LTO="true"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1"
export CARGO_PROFILE_RELEASE_OPT_LEVEL="z"
export CARGO_PROFILE_RELEASE_STRIP="true"
```

---

## 🎨 Frontend (SvelteKit 5)

### Outillage
```bash
# Aller dans le dossier frontend
cd /tmp/Nook/frontend

# Installer les dépendances
npm install

# Démarrer le serveur de dev
npm run dev

# Build production
npm run build
```

### Vérification (OBLIGATOIRE avant push)
```bash
# Linter
npm run lint

# Formatage
npm run format

# Vérifier les types TypeScript
npx svelte-check
```

### Patterns Svelte 5 Runes (Rappel)
```bash
# ✅ $state sur objet encapsulant (PAS de réassignation directe!)
# ✅ $derived pour expression simple
# ✅ $derived.by(() => { ... }) pour logique complexe
# ✅ Pas de {if} → utiliser {#if}
```

---

## 🧪 Tests E2E (Playwright)

### Exécution
```bash
cd /tmp/Nook/frontend

# Lister les tests
npx playwright test --list

# Lancer TOUS les tests (mode headless)
npx playwright test

# Lancer un fichier spécifique
npx playwright test tests/e2e.spec.ts

# Mode UI (debug)
npx playwright test --ui

# Avec timeout personnalisé
npx playwright test --timeout=60000
```

### Options importantes
```bash
# Ignorer les erreurs HTTPS (cert auto-signé)
npx playwright test --ignore-https-errors

# Navigateur visible (pour debug)
npx playwright test --headed

# Rapport HTML
npx playwright show-report
```

### Tests critiques Nook
- **163/163 tests PASS** (état actuel)
- **Fichier :** `frontend/tests/e2e.spec.ts`
- **Compte test :** hermes-bot / Hermes2026!

---

## 🗄️ Base de données (SQLite/SQLx)

### Conventions Nook
```sql
-- Toujours utiliser create_if_missing(true)
-- Toujours utiliser WAL journal mode
-- Voir : .claude/reference/rust-patterns.md
```

### Migrations
```bash
# Les migrations sont dans backend/migrations/
# Format : 001_*.sql, 002_*.sql, etc.
# Toute nouvelle migration → incrémenter le numéro
# Puis lancer : cargo sqlx prepare
```

---

## 🚀 CI/CD (GitHub Actions)

### Workflows disponibles
| Workflow | Fichier | Déclenchement |
|----------|---------|----------------|
| **Backend** | `.github/workflows/Backend.yml` | Manuel (workflow_dispatch) |
| **Frontend** | `.github/workflows/Frontend.yml` | Manuel |
| **Docker** | `.github/workflows/Docker.yml` | Manuel |
| **E2E** | `.github/workflows/test-nook.yml` | Manuel |

### Règles CI
- **Backend.yml** utilise **Rust nightly** (voir ligne 34)
- **Docker.yml** nécessite **Backend.yml** déclenché en premier pour les changements Rust
- **Ordre de build :** Commit d'abord, puis rebase pour lock updates

### Watch (depuis le repo)
```bash
# Tester la CI en local (nécessite act)
act -j backend --container-architecture linux/amd64
```

---

## 🤖 Outils Hermes Agent

### Commandes slash (définies dans skills/)
| Commande | Skill | Action |
|----------|------|--------|
| `/fini` ou `/nook-fin` | `nook-fin` | Session exit propre, push contexte |
| `/plan-ceo` | `nook-plan-ceo` | Valider une feature (vision produit) |
| `/plan-eng` | `nook-plan-eng` | Plan technique béton |
| `/review` | `nook-review` | Audit avant merge |
| `/ship` | `nook-ship` | Déploiement production |
| `/retro` | `nook-retro` | Bilan session |

### Lecture au démarrage (OBLIGATOIRE)
```bash
# Le hook session-start-tools.md force la lecture de :
.claude/hermes/active-session.md
.claude/hermes/known-issues.md
.claude/hermes/hermes-memory.md
.claude/project/project-state.md
```

### Mémoire (outil memory)
```python
# Ajouter une entrée
memory(action='add', target='memory', content='...')

# Remplacer une entrée
memory(action='replace', target='memory', old_text='...', content='...')

# Supprimer une entrée
memory(action='remove', target='memory', old_text='...')
```

### Skills (outil skill_manage)
```python
# Voir un skill
skill_view(name='nook-fin')

# Patcher un skill (correction mineure)
skill_manage(action='patch', name='...', old_string='...', new_string='...')

# Éditer un skill (réécriture majeure)
skill_manage(action='edit', name='...', content='...')

# Créer un nouveau skill
skill_manage(action='create', name='nook-[domaine]', category='...', content='...')
```

---

## 📹 WebRTC & Appels

### TURN Server (turn-rs)
```bash
# Vérifier que le TURN server tourne
curl http://localhost:3478/health

# Configuration :
# - docker-compose.yml : TURN_SECRET (maintenant via env vars!)
# - turn-rs/config.toml : max-threads >= 4
```

### Appels audio/vidéo
- **Page appel :** `https://192.168.1.192:6443/call/{convId}?type=audio`
- **Contrainte :** 1-to-1 uniquement (pas de groupes pour l'instant)
- **P2P >50 Mo :** Uniquement en 1-to-1 (voir `file-transfer.svelte.ts`)

---

## 🔧 Dépannage courant

### Le backend ne compile pas
```bash
# 1. Vérifier la syntaxe Rust (souvent .map_err())
cargo check 2>&1 | head -20

# 2. Vérifier les parenthèses/accolades
# Utiliser : .map_err(|err| { (...) })?  (pas de (...)? tout seul)

# 3. Vérifier la version rand (0.9 = rng(), pas thread_rng())
```

### Les tests E2E échouent
```bash
# 1. Vérifier que le serveur local tourne
curl -k https://192.168.1.192:6443/api/health

# 2. Vérifier les sélecteurs (Playwright)
# id="username", id="password" (login page)

# 3. Accepter le certificat HTTPS
npx playwright test --ignore-https-errors
```

### La CI rejette le push (secret scanning)
```bash
# NE JAMAIS commiter de token en clair !
# Utiliser l'outil memory pour stocker le token
# Puis : git reset HEAD~1 pour annuler le commit
# Corriger, puis recommiter
```

### Perte de contexte entre sessions
```bash
# Lire SYSTÉMATIQUEMENT au démarrage :
cat .claude/hermes/active-session.md
cat .claude/hermes/known-issues.md
# C'est MON espace de travail !
```

---

## 📝 Notes importantes

### À ne jamais faire
- ❌ Modifier les versions des dépendances dans un commit de fix
- ❌ Commiter des tokens/mots de passe en clair
- ❌ Push sans `cargo check` ou `npm run lint` avant
- ❌ Ignorer `.claude/hermes/` au démarrage

### À toujours faire
- ✅ Vérifier rand 0.9 : `rng()` pas `thread_rng()`
- ✅ Vérifier Axum 0.8 : `{param}` pas `:param`
- ✅ Utiliser SVG icons (pas d'emojis)
- ✅ Commit first, then rebase for lock updates
- ✅ Lire `.claude/hermes/` au démarrage

---

*Fichier généré le 2026-04-27. À mettre à jour quand de nouvelles commandes sont découvertes.*
