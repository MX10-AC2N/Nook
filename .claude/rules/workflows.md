# 🔄 Workflows — Nook

> Docker, CI/CD, déploiement homeserver, procédures

---

## 🐳 Docker — Deux Dockerfiles

| Fichier | Usage | Quand |
|---------|-------|-------|
| `Dockerfile` | Build depuis sources (cargo-chef) | CI test-nook.yml + dev local |
| `Dockerfile.release` | Binaires pré-compilés | Docker.yml (prod GHCR) |

### docker-compose.yml (prod homeserver)

```bash
# Zimaboard 832 — déploiement standard
docker compose pull && docker compose up -d

# Vérifier
curl http://localhost:6300/api/health
# → "OK"
```

### Variables d'environnement requises

```bash
# .env (copier depuis .env.example)
DATABASE_URL=sqlite:./data/nook.db
ALLOWED_ORIGINS=http://192.168.x.x:6300,https://nook.mondomaine.com
JWT_SECRET=<générer aléatoirement>
UPLOAD_DIR=./uploads
MAX_FILE_SIZE=52428800        # 50 Mo
UPLOAD_TTL_HOURS=48

# LAN seulement
BACKEND_PORT=6300

# WAN (Nginx Proxy Manager)
# Ajouter X-Forwarded-Proto: https dans la config Nginx
```

---

## 🚀 Workflows GitHub Actions

Tous les workflows sont **manuels** (déclenchement via `workflow_dispatch`).

### Ordre recommandé pour une release

1. **Backend.yml** → compile amd64 + arm64 → artifacts 7j
2. **Frontend.yml** → build SvelteKit → artifact 7j
3. **test-nook.yml** → intégration Docker + E2E Playwright
4. **Docker.yml** → assemble les artifacts → push GHCR
5. **Release.yml** → bump VERSION + tag git

### Backend.yml — Matrix amd64/arm64

```yaml
# Deux jobs parallèles → deux fichiers rapport séparés (évite race condition)
# BACKEND-BUILD-REPORT-amd64.md
# BACKEND-BUILD-REPORT-arm64.md
# ⚠️ NE PAS réunifier en un seul fichier → race condition git garantie
```

### test-nook.yml — Healthcheck correct

```bash
# ❌ /health → ServeDir fallback → toujours 200 même si backend crash
until curl -sf http://localhost:6300/health; do sleep 3; done

# ✅ /api/health → handler Axum → "OK" uniquement si backend UP
until curl -sf http://localhost:6300/api/health | grep -q "OK"; do sleep 3; done
```

### Docker.yml — dawidd6

```yaml
# Cross-workflow artifacts via dawidd6/action-download-artifact@v6
# (pas actions/download-artifact qui ne supporte pas cross-workflow)
```

---

## 🏠 Déploiement Homeserver (Zimaboard 832)

### Mise à jour depuis GHCR

```bash
docker compose pull
docker compose up -d
docker compose logs -f
```

### Vérification post-déploiement

```bash
# Backend health
curl http://localhost:6300/api/health

# Logs en temps réel
docker compose logs -f backend

# Taille DB
ls -lh ./data/nook.db
```

### Backup DB

```bash
# Avant toute migration majeure
sqlite3 ./data/nook.db ".backup './data/nook-backup-$(date +%Y%m%d).db'"
```

---

## 🧪 Playwright E2E — Lancer en local

```bash
cd frontend

# Installer Playwright (une seule fois)
npx playwright install

# Lancer les tests (avec serveur Docker déjà démarré)
CI=true npx playwright test

# Lancer avec UI interactive
npx playwright test --ui

# Lancer un test spécifique
npx playwright test -g "Auth - Login valide"
```

### CI (test-nook.yml)

```
1. Build Dockerfile depuis sources
2. docker compose -f docker-compose.yml -f docker-compose.ci.yml up
3. Attendre /api/health → "OK"
4. E2E_SETUP=1 → backend crée user e2e_ci automatiquement
5. npx playwright test --reporter=list
6. Commit TEST_REPORT.md dans .claude/
```

---

## 📝 Procédure — Nouvelle session de travail

1. Lire `CLAUDE.md` (ce fichier de routing)
2. Lire `BUGS.md` (bugs actifs à ne pas réintroduire)
3. Lire `SESSIONS.md` (dernière session pour contexte)
4. Si `USER_TEST.md` a été mis à jour → le lire en priorité
5. Fetcher les fichiers source concernés via Raw GitHub
6. Intervenir avec contenu complet
7. En fin de session : mettre à jour `SESSIONS.md` et `BUGS.md`

---

## 🆕 Nouveaux workflows (session 24)

| Fichier | Agent | Usage |
|---------|-------|-------|
| `sqlx-prepare.yml` | 🦀 RUST | Régénère `.sqlx/queries.json` après migration SQL |
| `bundle-analysis.yml` | 🎨 SVELTE | Rapport tailles chunks + alerte DT-01 libsodium |
| `e2e-targeted.yml` | 🧪 E2E | Lance 1 test par nom (debug rapide, évite 10 min de CI) |
| `generate-android-instruction.yml` | 📐 ARCHITECT | Auto-génère `.claude/ANDROID-INSTRUCTION.md` |

## 📋 Catalogue complet des workflows (session 46)

> **20 workflows au total.** Voir détails complets et recommandations de nettoyage dans
> [`.claude/WORKFLOW-CATALOG.md`](../WORKFLOW-CATALOG.md).

### Tests E2E — 3 fichiers de tests

| Fichier | Tests | Description |
|---------|-------|-------------|
| `tests/user.spec.ts` | 49 | Flux user complet (auth, chat, reactions, upload, polls, chess, calendar, settings, E2EE, push, rate limit) |
| `tests/admin.spec.ts` | 18 | Flux admin (login, users, invites, analytics, isolation) |
| `tests/api-sanity.spec.ts` | 48 | 48 routes protégées → 401 sans auth |
| `tests/chess-extended.spec.ts` | 27 | Scénarios avancés chess (promotion, timer, IA×5, resign, humain, UI, noir) |
| `tests/webrtc.spec.ts` | 14 | WebRTC API, WebSocket auth, page call, upload audio/vidéo |
| **Total** | **156** | Tous passent ✅ |

## 🔄 Séquence complète recommandée (feature shipping)

```
1. sqlx-prepare.yml    (si migration SQL modifiée)
2. Backend.yml         (build + clippy)
3. Frontend.yml        (build + bundle-analysis.yml)
4. test-nook.yml       (suite E2E complète — 156 tests)
5. Docker.yml          (image distroless)
6. ghcr-cleanup.yml    (auto après Docker.yml)
7. generate-android-instruction.yml  (si VERSION ou BUGS.md changés)
```
