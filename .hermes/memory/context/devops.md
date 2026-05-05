# 🚀 Contexte DevOps - Nook

> Mis à jour : 2026-05-05

## Infrastructure

- **Conteneurisation** : Docker multi-arch (amd64/arm64)
- **Base** : Alpine 3.21 (distroless pour Rust)
- **CI/CD** : GitHub Actions
- **Registry** : GHCR (GitHub Container Registry)
- **Déploiement** : Zimaboard (docker-compose)
- **HTTPS local** : nginx-alpine:6443 (cert auto-signé)

## Workflows GitHub Actions

### Backend.yml
```yaml
Jobs:
  - Backend (amd64/arm64)
    - cargo check
    - cargo clippy
    - cargo test
    - cargo build --release
    - Upload artifact
```

### Frontend.yml
```yaml
Jobs:
  - Frontend (node_modules en cache)
    - npm ci (SANS --omit optional)
    - npm run build
    - Upload artifact
```

### Docker.yml
```yaml
Jobs:
  - Build & Push multi-arch
    - Plateformes: linux/amd64, linux/arm64
    - Base: Alpine 3.21 distroless
```

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
