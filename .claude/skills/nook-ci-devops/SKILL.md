---
name: nook-ci-devops
description: Skill spécialisé pour la CI GitHub Actions, Docker et le déploiement Zimaboard du projet Nook. Utilise cette skill dès qu'un workflow .yml est impliqué, qu'un Dockerfile est modifié, que docker-compose est touché, que le rapport DOCKER-BUILD-REPORT.md signale des erreurs, ou que des artifacts CI sont manquants. Couvre : Backend.yml, Frontend.yml, test-nook.yml, Docker.yml, Release.yml, sqlx-prepare.yml, Dockerfile, Dockerfile.release, docker-compose.yml, GHCR, déploiement Zimaboard ARM64.
---

# 🚀 Nook — CI/DevOps Skill

## Périmètre

```
.github/workflows/
├── Backend.yml           → Rust compile amd64 + arm64 → artifacts 7j
├── Frontend.yml          → SvelteKit build → artifact 7j
├── test-nook.yml         → Docker build + E2E Playwright
├── Docker.yml            → Assemble artifacts → GHCR multi-arch
├── Release.yml           → Bump VERSION + tag git
├── sqlx-prepare.yml      → Régénère .sqlx/queries.json
└── e2e-targeted.yml      → Debug un seul test E2E

Dockerfile                → Build depuis sources (cargo-chef) — test + dev
Dockerfile.release        → Binaires pré-compilés — prod
docker-compose.yml        → Production Zimaboard
docker-compose.ci.yml     → Override CI (E2E_SETUP=1, named volumes)
```

## Ordre de la pipeline CI — respecter scrupuleusement

```
1. sqlx-prepare.yml     ← SI migration SQL ajoutée (sinon skip)
2. Backend.yml          → artifacts: nook-backend-{amd64,arm64} (7j)
3. Frontend.yml         → artifact: nook-frontend (7j)
4. test-nook.yml        → compile Dockerfile + E2E Playwright
5. Docker.yml           → assemble artifacts → GHCR (dawidd6/action-download-artifact@v6)
6. ghcr-cleanup.yml     → auto après Docker.yml
7. Release.yml          → bump VERSION + Cargo.toml + package.json + tag git
```

> Si Docker.yml échoue avec "artifact not found" → vérifier que Backend.yml + Frontend.yml ont tourné et que les artifacts ne sont pas expirés (TTL 7j).

## Deux Dockerfiles — ne pas confondre

### `Dockerfile` — build complet depuis sources
- Utilise `cargo-chef` pour le cache des deps
- ⚠️ **NE PAS** `COPY .cargo/config.toml` dans le container : contient des linkers cross (musl-gcc, aarch64-linux-gnu-gcc) qui n'existent pas dans le container → crash linker
- ✅ `COPY backend/src ...` (pas `COPY backend/ .`)
- Utilisé par : `test-nook.yml`

### `Dockerfile.release` — binaires pré-compilés
- Copie les binaires depuis les artifacts `Backend.yml`
- Image finale : `gcr.io/distroless/cc-debian12`
- User : 65532 (nonroot distroless)
- ⚠️ Volumes montés → **init container `alpine:3`** obligatoire pour `chown -R 65532:65532`
- ⚠️ Pas de healthcheck `CMD-SHELL` (distroless = pas de shell)
- Utilisé par : `Docker.yml` → GHCR

## Règles d'or CI

### Heredoc dans les workflows
```yaml
# ❌ EOF dans run: avec indentation → le heredoc capture l'indentation
- run: |
    cat > file.md << EOF
      contenu indenté  # ← espaces capturés dans le fichier !
    EOF

# ✅ ENDOFMD (marqueur unique) + pas d'indentation dans le heredoc
- run: |
    cat > file.md << ENDOFMD
    contenu sans indentation parasite
    ENDOFMD
```

### Artifacts cross-workflow
```yaml
# ✅ seule action qui supporte les artifacts cross-workflow
- uses: dawidd6/action-download-artifact@v6
  with:
    workflow: Backend.yml
    name: nook-backend-aarch64-unknown-linux-gnu
```

### Commits CI sans boucle infinie
```yaml
# Pattern validé pour les commits CI (rapports, queries.json)
- name: Commit rapport
  run: |
    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add .claude/BACKEND-BUILD-REPORT-${ARCH}.md
    if git diff --staged --quiet; then
      echo "Rien à committer"
    else
      git commit -m "ci: rapport [skip ci]"
      git pull --rebase --autostash origin develop
      git push --force-with-lease origin HEAD:develop
    fi
```

## Docker distroless — pièges

```
✅ Image finale sans shell → jamais de docker exec nook sh
✅ Init container chown OBLIGATOIRE pour volumes bind-mounted
✅ Pas de CMD-SHELL healthcheck
✅ User 65532 doit pouvoir écrire dans /app/data + /app/data/uploads
❌ NE PAS copier .cargo/ dans les stages Docker
❌ NE PAS mettre E2E_SETUP=1 dans docker-compose.yml prod
```

## Variables d'environnement critiques

| Variable | Prod (Zimaboard) | CI |
|----------|------------------|----|
| `E2E_SETUP` | `0` ← JAMAIS `1` | `1` |
| `DATA_DIR` | `/media/ac2n-cloud/volume_docker_Nook/nook-data` | `./data` |
| `ALLOWED_ORIGINS` | `http://192.168.X.X:6300,https://nook.mondomaine.com` | `http://localhost:6300` |
| `PUBLIC_SITE_URL` | `http://192.168.X.X:6300` | `http://localhost:6300` |

## Diagnostics rapides

| Erreur CI | Cause | Fix |
|-----------|-------|-----|
| `permission denied 65532` | chown init container manquant | Ajouter init container alpine:3 |
| `artifact not found` (dawidd6) | Backend.yml ou Frontend.yml n'a pas tourné | Lancer dans l'ordre |
| `linker error aarch64` | `.cargo/config.toml` copié dans Docker | Ne jamais COPY .cargo/ |
| `no such file: STATIC_FILES_DIR` | Chemin mal configuré | Vérifier env var dans compose |
| Clippy `-D warnings` échoue | Warning = erreur | Corriger tous les warnings avant push |
| `cargo clippy` OK, `cargo build` OK mais artifact vide | Binary strip a échoué | Vérifier le chemin `target/*/release/nook-backend` |

## Nginx Proxy Manager → Nook (WAN)

```
Forward Hostname/IP : localhost (ou IP Zimaboard)
Forward Port       : 6300
Websockets Support : ON  ← obligatoire pour /ws
SSL                : Let's Encrypt recommandé
```

## Flux inter-agents

```
← 🦀 RUST  : nouvelles migrations SQL → lancer sqlx-prepare.yml AVANT Backend.yml
← 🦀 RUST  : nouvelles env vars → mettre à jour docker-compose.yml + .env.example
→ 🧪 E2E   : docker-compose.ci.yml modifié → tester test-nook.yml
```
