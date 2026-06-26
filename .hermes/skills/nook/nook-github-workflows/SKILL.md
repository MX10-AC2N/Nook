---
name: nook-github-workflows
description: Ordonnancer et surveiller les GitHub Actions Nook — Frontend → Backend → Turn → Docker → Test Nook (E2E).
---

# nook-github-workflows

## Règle d'or
Ordre strict: **Frontend → Backend → Turn → Docker → Test Nook (E2E)**
Docker ne se lance JAMAIS avant les 3 autres au vert.
Test Nook (E2E) valide l'image Docker déployée.

## Noms de workflows réels (pas les noms de fichiers)
- `2==> 🎨 Frontend Build & Artifact` (ID 220018364)
- `1==>🏗️ Backend Build & Artifact` (ID 220018362)
- `3 ==> Turn-Server Build and Artifact` (ID 257238341)
- `4==> 🐳 Docker Build & Push` (ID 220018363)
- `4 ==> Test Nook` (ID pour test-nook.yml — E2E validation finale)

## Quand trigger
- Frontend si `frontend/**` a changé
- Backend si `backend/**` a changé
- Turn si `services/turn-rs/**` a changé
- Docker si Backend/Turn/Frontend sont verts **et** qu'un build d'image est nécessaire
- Test Nook (E2E) après Docker vert pour validation finale

## Séquence correcte
1. `git fetch origin`
2. Inspecter `gh run list --limit 5`
3. Trigger manuel dans l'ordre (`gh workflow run "<nom workflow>" --ref develop -R MX10-AC2N/Nook`), attendre chaque fin avant le suivant
4. **Validation finale** : `gh workflow run "4 ==> Test Nook" --ref develop -R MX10-AC2N/Nook` (test-nook.yml — E2E complet sur instance déployée)
5. Ne JAMAIS utiliser de cron ou de schedule

## Commandes essentielles
- `gh run list --workflow="2==> 🎨 Frontend Build & Artifact" --limit 5`
- `gh run list --workflow="1==>🏗️ Backend Build & Artifact" --limit 5`
- `gh run list --workflow="3 ==> Turn-Server Build and Artifact" --limit 5`
- `gh run list --workflow="4==> 🐳 Docker Build & Push" --limit 5`
- `gh run watch <run-id> -R MX10-AC2N/Nook` (surveiller en direct)

## Prérequis environnement
- **`gh` CLI installé ET authentifié** (`gh auth login` → token PAT avec scopes `repo` + `workflow`) — **PREFERRED**
- **Fallback sans gh CLI** : Direct GitHub API calls via Python `urllib` (voir `references/github-token-management.md`)
- Sans `gh` ET sans API token valide → impossible de déclencher/surveiller depuis le container

## Token GitHub — Gestion Critique (session 2026-06-17)
- **Classic PAT (ghp_)** avec scopes `repo` + `workflow` → plus simple, fiable
- **Fine-grained PAT** → nécessite `Contents: Read/Write`, `Workflows: Read/Write`, `Actions: Read`
- **Validation OBLIGATOIRE** avant usage : endpoints `/user` (200 OK + scopes attendus)
- **Stockage** : `/opt/data/.env` (persistant) — **PAS** `/tmp/.git_token` (éphémère)
- Voir `references/github-token-management.md` pour scripts complets API + monitoring

## Réutilisation d'artifacts
Frontend/Backend/Turn uploadent leurs artifacts. Docker les télécharge avec `dawidd6/action-download-artifact` selon `github.event.workflow` filtré par chemin modifié.

## Pièges fréquents
- Tokens GitHub expirés/invalides → créer nouveau PAT fine-grained (scopes `repo` + `workflow`) ou classic (`repo` + `workflow`)
- `gh` absent du container Docker → installer via `apt install gh` (Debian) ou `apk add github-cli` (Alpine)
- Trigger dans le mauvais ordre = Docker échoue (artifacts manquants)
- Noms de workflows avec émojis/espaces → utiliser guillemets doubles exacts
- **OOM sur GitHub Actions (backend arm64/amd64)** → ajouter `CARGO_BUILD_JOBS: "1"` dans l'env du step `cargo build --release` (limite la parallélisation, évite OOM 7GB/14GB runners)
- **git push rejected (local has commits)** → `git fetch origin && git rebase origin/develop` AVANT push (stash local changes if needed)
- **Workflow trigger sur mauvais ref (main au lieu de develop)** → toujours ajouter `-R MX10-AC2N/Nook --ref develop` explicitement

## Backend build tuning (session 2026-06-13)
Le job `cargo build --release --target x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl` consomme trop de RAM en parallèle pur (LTO + opt-level=z).
Fix appliqué dans `.github/workflows/Backend.yml` :
```yaml
env:
  CARGO_BUILD_JOBS: "1"
  CARGO_PROFILE_RELEASE_LTO: "true"
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1"
  CARGO_PROFILE_RELEASE_OPT_LEVEL: "z"
  CARGO_PROFILE_RELEASE_STRIP: "true"
```
Résultat : build passe (6-7 min) sans OOM, artifacts produits pour amd64 + arm64.

## Session 2026-06-15 — Full Pipeline Success (avec fix middleware)
- Frontend: ✅ 16s (clean npm ci + build)
- Backend: ✅ 10m24s (LTO + clippy clean, arm64 + amd64) — **fix middleware state propagation**
- Turn: ✅ 2m36s (nightly rust + musl)
- Docker: ✅ 53s (multi-arch push GHCR)
- Test Nook (E2E): 🔄 En cours (Build Docker image → Start stack → Playwright)

## Session 2026-06-16 — Effect orphan fix + CI/CD rerun
- Fix `effect_orphan` page blanche → déplacé `initCryptoListener()` dans `onMount` de `chat/+page.svelte`
- Frontend rebuild OK après ajout export manquant dans `chatStore.ts` re-export
- Pipeline complet relancé : Frontend → Backend → Turn → Docker sur branche `develop`