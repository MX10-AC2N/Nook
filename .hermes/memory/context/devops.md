# 🚀 Contexte DevOps - Nook

> Mis à jour : 2026-05-12

## Infrastructure

- **Conteneurisation** : Docker multi-arch (amd64/arm64)
- **Base** : Alpine 3.20 (runtime minimal)
- **CI/CD** : GitHub Actions
- **Registry** : GHCR (GitHub Container Registry)
- **Déploiement** : Zimaboard (docker-compose)
- **HTTPS local** : nginx:6443 (cert auto-signé)

## Build Backend (Backend.yml)

### Architecture — Deux jobs séparés (plus fiable qu'une matrix)

| Job | Runner | Musl target | Artifact name |
|-----|--------|-------------|---------------|
| backend-amd64 | ubuntu-latest | x86_64-unknown-linux-musl | nook-backend-x86_64-unknown-linux-musl |
| backend-arm64 | ubuntu-24.04-arm | aarch64-unknown-linux-musl | nook-backend-aarch64-unknown-linux-musl |

### Steps par job
- cargo check
- cargo clippy
- cargo test
- cargo build --release --target <musl-target>
- Upload artifact → `artifacts/nook-backend-<amd64|arm64>`

### ⚠️ Piège : noms d'artefacts GitHub vs noms de binaires
- **Artifact GitHub** : `nook-backend-x86_64-unknown-linux-musl` (zip)
- **Binaire dedans** : `nook-backend-amd64` (fichier réel)
- Docker.yml doit chercher les noms d'artefacts GitHub, mais le `Verify context` cherche les noms de binaires

## Build Turn-Server (turn.yml)

### Architecture — Matrix amd64/arm64
| Arch | Runner | Artifact name |
|------|--------|---------------|
| amd64 | ubuntu-latest | nook-turn-server-amd64 |
| arm64 | ubuntu-24.04-arm | nook-turn-server-arm64 |

Binaire construit dans Docker Alpine builder, extrait via `docker create` + `docker start`.

## Build Frontend (Frontend.yml)

```yaml
Jobs:
  - Frontend
    - npm ci (SANS --omit optional)
    - npm run build
    - Upload artifact: nook-frontend
```

## Docker Build & Push (Docker.yml)

### Téléchargement inter-workflow (dawidd6/action-download-artifact@v6)
1. backend amd64 → `nook-backend-x86_64-unknown-linux-musl`
2. backend arm64 → `nook-backend-aarch64-unknown-linux-musl`
3. turn amd64 → `nook-turn-server-amd64`
4. turn arm64 → `nook-turn-server-arm64`
5. frontend → `nook-frontend`

### Dockerfile.release
- Base: `alpine:3.20`
- Dépendances: `sqlite-libs ca-certificates libsodium`
- ⚠️ Pas `libsqlite3` (nom Debian) → sur Alpine c'est `sqlite-libs`
- Multi-arch: linux/amd64, linux/arm64
- Tags: `v<VERSION>` + `latest` (main) ou `dev` (develop) ou `<branch>`

## Docker Compose (Zimaboard)

```yaml
services:
  nook-backend:
    image: ghcr.io/mx10-ac2n/nook-backend:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:nook.db
      - JWT_SECRET=${JWT_SECRET}
      
  nook-frontend:
    image: ghcr.io/mx10-ac2n/nook-frontend:latest
    ports:
      - "6300:3000"
      
  turn-server:
    image: ghcr.io/mx10-ac2n/turn-rs:latest
    ports:
      - "3478:3478/udp"
      - "3478:3478/tcp"
```

## Points Critiques

### ✅ Corrections Récentes
- **Frontend workflow** : Supprimé `--omit optional` (Rollup needs optional deps)
- **Cron Docker** : Supprimé (aucun cron actif)
- **package-lock.json** : Régénéré avec TOUTES les dépendances

### ⚠️ À Surveiller
- **Secrets** : Utiliser GitHub Secrets, jamais en dur
- **Multi-arch** : Tester amd64 ET arm64
- **Healthchecks** : Configurés sur tous les services
- **Volumes** : Persistence des données (SQLite, uploads)

## Commandes Utiles

```bash
# Build local multi-arch
docker buildx build --platform linux/amd64,linux/arm64 -t nook-backend .

# Push vers GHCR
docker push ghcr.io/mx10-ac2n/nook-backend:latest

# Déploiement Zimaboard
ssh user@zimaboard "cd /opt/nook && docker-compose pull && docker-compose up -d"

# Logs
docker logs nook-backend --tail 100 -f
```

## Secrets GitHub

```
JWT_SECRET              # Pour signer les JWT
TURN_SECRET             # Secret TURN server
DATABASE_URL            # SQLite URL
```

## Monitoring

- **Healthchecks** : `/health` sur chaque service
- **Logs** : `docker logs` ou centralisation (Loki/Promtail)
- **Métriques** : À implémenter (Prometheus?)

## Connexions MCP

- **SocratiCode** : Analyse infrastructure as code
- **GitHub CLI** (`gh`) : Gestion workflows, secrets
