# gh CLI — Install & Auth sur Container/Debian

## Installation (Debian/Ubuntu)

```bash
# En root dans le container
apt update && apt install -y gh

# Vérifier
gh --version
# gh version X.Y.Z (Debian X.Y.Z-1)
```

## Authentification

```bash
# Option 1: PAT classic (scopes repo + workflow)
echo "ghp_xxxxxxxxxxxxxxxxxxxx" | gh auth login --with-token

# Option 2: PAT fine-grained (repo + workflow scopes + installation access)
echo "github_pat_xxxxxxxxxxxxxxxxxxxx" | gh auth login --with-token

# Option 3: Interactive (nécessite browser)
gh auth login

# Vérifier
gh auth status
# github.com
#   ✓ Logged in to github.com account MX10-AC2N
```

## Utilisation depuis CI/CD (container)

```bash
# Trigger workflow
gh workflow run "1==>🏗️ Backend Build & Artifact" --ref develop -R MX10-AC2N/Nook

# Surveiller
gh run watch <run-id> -R MX10-AC2N/Nook

# Lister
gh run list --workflow="1==>🏗️ Backend Build & Artifact" --limit 5 -R MX10-AC2N/Nook
```

## Notes importantes

- **Ne PAS stocker le token dans le Dockerfile** → passer via `--build-arg` ou injection runtime
- En container root, `gh` stocke config dans `/root/.config/gh/hosts.yml`
- Token expiry: PAT classic 90j max (recommander rotation), fine-grained configurable
- Scopes minimaux: `repo` (contents) + `workflow` (dispatch)
- Si `gh` absent après rebuild container → le Dockerfile doit l'installer (`RUN apt update && apt install -y gh`)

Session 2026-06-13 : `gh` absent post-docker-update → installé manuellement, authentifié, CI chain complète déclenchée.