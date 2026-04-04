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

---

## Debugging Checklist — Backend CI Failures

| Error | Location | Fix | Verify |
|-------|----------|-----|--------|
| `cannot find tuple struct State` | handler signature | Add `State` to `use axum::{ extract::{..., State} }` | `extract::State` in imports |
| `unresolved imports SystemExt, CpuExt` | sysinfo use | Remove both, keep `use sysinfo::System;` | No `SystemExt` anywhere |
| `no method named disks` | sysinfo 0.32 API | Comment out or use `sys.refresh_all()` | No `sys.disks()` without refresh |
| `variable does not need to be mutable` | unused_mut | Remove `mut` | No `let mut disks` in admin |
| `cannot find value state in scope` | `&state.db` | Ensure `State(state)` not `State(_state)` | Only metrics func uses `_state` |
| `borrow of moved value: user_id` | `tokio::spawn` | Clone at handler start: `let uid = user_id.clone()` | No `user_id` after `async move` |
| `no field db on type fn() -> State` | `&State.db` | Never use `&State.db` (capital S) | Zero occurrences |
| `no such table` | `sqlx::query!` | Use `query_as::<_, (String,)>()` | No `sqlx::query!` for new tables |
| `expected identifier, found keyword use` | import block | Remove standalone `use` inside multi-line bloc | All imports at file top level |
| `if can be collapsed into outer match` | match guards | Use arm guard: `"type" if !x.starts_with() => { return Err }` | No `if !` inside match arms |

---

## Frontend Build Errors

### Missing closing brace — cascade effects
**Symptom**: `Expected ";" but found "async"` at wrong line.
**Root cause**: A method ~5-10 lines earlier is missing its closing `}`. All subsequent methods absorbed.
**Fix**: Track brace balance from start. Find deviation at `public`/`private` declaration.
**Verify**: Count `{` vs `}` — must be 0.

### Switch case inside object literal
**Symptom**: `Expected ":" but found "'case_value'"`.
**Fix**: Close object literal properly (e.g. `}));` for CustomEvent), place case at switch level.

### Duplicate test.describe in same file
**Symptom**: Playwright runs wrong tests or throws duplicate errors.
**Fix**: Scan for repeated describe names, remove flaky duplicates.

### CI: Old runs on stale commits
**Issue**: CI shows errors from commits before your fix.
**Fix**: Get HEAD SHA, cancel ALL in_progress runs NOT on HEAD. Only trust matching SHA.

## Flux inter-agents

```
← 🦀 RUST  : nouvelles migrations SQL → lancer sqlx-prepare.yml AVANT Backend.yml
← 🦀 RUST  : nouvelles env vars → mettre à jour docker-compose.yml + .env.example
→ 🧪 E2E   : docker-compose.ci.yml modifié → tester test-nook.yml
```
