# 🚀 Rôle : Ingénieur CI/DevOps — Nook

> Spécialiste GitHub Actions, Docker, déploiement Zimaboard.
> Activer ce rôle pour : workflows CI, Dockerfiles, compose, build artifacts, GHCR.

---

## 🎯 Périmètre exclusif

```
.github/workflows/
├── Backend.yml           → Rust compile amd64 + arm64 → artifacts 7j
├── Frontend.yml          → SvelteKit build → artifact 7j
├── test-nook.yml         → Intégration Docker + E2E Playwright
├── Docker.yml            → Assemble artifacts → GHCR (Dockerfile.release)
├── Release.yml           → Bump VERSION + tag git
└── update-frontend-lock.yml → Régénère package-lock.json

Dockerfile                → Build sources (cargo-chef) — test + dev
Dockerfile.release        → Binaires pré-compilés — prod
docker-compose.yml        → Production Zimaboard
docker-compose.ci.yml     → Override CI (E2E_SETUP, named volumes)
```

---

## 🐳 Architecture Docker — Deux Dockerfiles

### `Dockerfile` — Build depuis sources

```dockerfile
# Utilise cargo-chef pour cache des dépendances
# Stages : chef (planner) → builder (compilation) → runtime (distroless)
# ⚠️ NE PAS copier .cargo/config.toml dans le container
#    .cargo/config.toml contient les linkers cross (musl-gcc, aarch64-linux-gnu-gcc)
#    Ces linkers n'existent PAS dans le container → build crash
# ✅ COPY backend/src ... (pas COPY backend/ .)
```

### `Dockerfile.release` — Binaires pré-compilés

```dockerfile
# Copie les binaires pré-compilés par Backend.yml
# Beaucoup plus rapide (~30s vs ~10min)
# Image finale : gcr.io/distroless/cc-debian12
# User : 65532 (distroless nonroot)
# Volumes montés → nécessitent init container pour chown
```

### `docker-compose.yml` — Production

```yaml
# Zimaboard 832 — configuration stable
# Bind mounts (pas named volumes) pour faciliter backup
# Ports : 6300:3000 (hôte:container)
# ⚠️ Pas de healthcheck CMD-SHELL (distroless n'a pas de shell)
# ⚠️ Pas d'E2E_SETUP (jamais en prod)
```

### `docker-compose.ci.yml` — Override CI

```yaml
# Extension du compose prod pour la CI
# Ajoute : E2E_SETUP=1, named volumes, init container alpine:3
# Usage : docker compose -f docker-compose.yml -f docker-compose.ci.yml up
```

---

## 🔄 Workflows GitHub Actions

### Règles d'or — JAMAIS violer

```yaml
# ❌ JAMAIS déclencher automatiquement (on: push)
# ✅ TOUJOURS manuel
on:
  workflow_dispatch:

# ❌ JAMAIS un seul fichier rapport pour la matrix amd64/arm64
# → Race condition garantie (deux jobs commitent simultanément)
# ✅ Fichiers séparés par architecture
BACKEND-BUILD-REPORT-amd64.md
BACKEND-BUILD-REPORT-arm64.md

# ❌ JAMAIS actions/download-artifact pour cross-workflow
# → Ne supporte pas les artifacts d'autres workflows
# ✅ dawidd6/action-download-artifact@v6
- uses: dawidd6/action-download-artifact@v6
  with:
    workflow: Backend.yml
    name: backend-amd64
```

### Heredoc dans les workflows — Format obligatoire

```yaml
# ❌ Heredoc indenté → Markdown cassé (espaces en tête de ligne)
- run: |
    cat << 'EOF' >> $GITHUB_STEP_SUMMARY
      # Rapport
      - item
    EOF

# ✅ Heredoc NON indenté
- run: |
    cat << 'EOF' >> $GITHUB_STEP_SUMMARY
# Rapport
- item
EOF
```

### Healthcheck CI — Endpoint correct

```bash
# ❌ /health → ServeDir fallback → retourne index.html → toujours 200
until curl -sf http://localhost:6300/health; do sleep 3; done

# ✅ /api/health → handler Axum → "OK" uniquement si backend UP
until curl -sf http://localhost:6300/api/health | grep -q "OK"; do
  sleep 3
  ((attempts++))
  if [ $attempts -ge 30 ]; then echo "Timeout"; exit 1; fi
done
```

---

## 📋 Backend.yml — Matrix cross-compilation

```yaml
strategy:
  matrix:
    include:
      - arch: amd64
        target: x86_64-unknown-linux-gnu
        linker: x86_64-linux-gnu-gcc
        report: BACKEND-BUILD-REPORT-amd64.md
      - arch: arm64
        target: aarch64-unknown-linux-gnu
        linker: aarch64-linux-gnu-gcc
        report: BACKEND-BUILD-REPORT-arm64.md

# ⚠️ .cargo/config.toml contient les linkers cross
# NE PAS le copier dans Dockerfile ou l'image finale
# Note: cible x86_64 supprimée du config (c'est la cible native du runner)
```

---

## 🚢 Docker.yml — Pipeline prod

```
Ordre obligatoire :
1. Download artifact backend-amd64 (depuis Backend.yml)
2. Download artifact backend-arm64 (depuis Backend.yml)
3. Download artifact frontend (depuis Frontend.yml)
4. Build image multi-arch avec Dockerfile.release
5. Push vers ghcr.io/mx10-ac2n/nook
6. Tag : latest + VERSION (ex: 0.3.0-beta.2)
```

---

## 🏠 Déploiement Zimaboard 832

### Mise à jour standard

```bash
# Sur le Zimaboard
cd /opt/nook  # ou chemin du docker-compose.yml
docker compose pull
docker compose up -d
docker compose logs -f --tail=50

# Vérification
curl http://localhost:6300/api/health
```

### Init container — Pourquoi obligatoire

```
distroless user = 65532 (nonroot)
Docker crée les volumes avec owner = root
→ Le process nook ne peut pas écrire dans /data ou /uploads
→ Init container alpine:3 fait chown 65532:65532 avant le démarrage

# Dans docker-compose.yml
services:
  init:
    image: alpine:3
    command: chown -R 65532:65532 /data /uploads
    volumes:
      - nook_data:/data
      - nook_uploads:/uploads
  backend:
    depends_on:
      init:
        condition: service_completed_successfully
```

---

## 🔐 Variables d'environnement

| Variable | Obligatoire | Exemple |
|----------|-------------|---------|
| `DATABASE_URL` | ✅ | `sqlite:./data/nook.db` |
| `ALLOWED_ORIGINS` | ✅ | `http://192.168.1.x:6300,https://nook.domaine.com` |
| `JWT_SECRET` | ✅ | 64 chars random |
| `UPLOAD_DIR` | ✅ | `./uploads` |
| `MAX_FILE_SIZE` | ❌ | `52428800` (50 Mo) |
| `UPLOAD_TTL_HOURS` | ❌ | `48` |
| `BACKEND_PORT` | ❌ | `3000` (interne container) |
| `E2E_SETUP` | ❌ | `1` (CI uniquement, JAMAIS prod) |

---

## 🐛 Diagnostics CI fréquents

| Symptôme | Cause | Fix |
|----------|-------|-----|
| `non-fast-forward` sur rapport | Deux jobs committent en parallèle | Fichiers séparés par arch |
| Build Rust OK, Docker crash | `.cargo/config.toml` copié → linker manquant | COPY sélectif, exclure `.cargo/` |
| Container démarre mais DB inaccessible | Permissions volumes distroless | Init container chown 65532 |
| healthcheck timeout | `/health` → ServeDir 200 trompeur | Utiliser `/api/health` |
| `artifact not found` | cross-workflow avec mauvaise action | `dawidd6/action-download-artifact@v6` |
| E2E tests 401 sur login | E2E_SETUP=1 absent en CI | Ajouter dans docker-compose.ci.yml |
| Heredoc Markdown cassé | Heredoc indenté | Aligner EOF à la colonne 0 |

---

## 🤝 Flux inter-agents

```
← 🦀 RUST / 🎨 SVELTE / 🔐 CRYPTO : nouvelles env vars, secrets, deps
→ 🧪 E2E                           : stack Docker opérationnelle, healthcheck passant
→ Tous                              : rapports CI (BUILD-REPORT-*.md)
```

**Checklist clôture DEVOPS** : env vars dans .env.example ✓ | queries.json régénéré si sqlx! ✓ | pas de `on: push` ajouté ✓

---

## 📚 Apprentissages

> *Section mise à jour à chaque session.*

### [APP-DEVOPS-01] .cargo/config.toml dans Docker = crash linker — Session 4
→ **Promu** dans section principale.

### [APP-DEVOPS-02] distroless user 65532 + volumes root — Session 4
→ **Promu** dans section principale.

### [APP-DEVOPS-03] Matrix amd64/arm64 race condition git — Session 20
→ **Promu** dans Règles d'or.

### [APP-DEVOPS-04] Heredoc indenté casse le Markdown — Session 20
→ **Promu** dans Règles d'or.

### [APP-DEVOPS-05] dawidd6 pour cross-workflow artifacts — Session 7
→ **Promu** dans Docker.yml — Pipeline prod.

### [APP-DEVOPS-06] docker-compose healthcheck CMD-SHELL + distroless — Session 7

`docker-compose.yml` healthcheck avec `CMD-SHELL` → `sh` absent dans distroless → fail.
→ Supprimer le healthcheck compose, utiliser le healthcheck CI séparé.
Status : Résolu. Ne pas réintroduire de CMD-SHELL healthcheck en prod.
