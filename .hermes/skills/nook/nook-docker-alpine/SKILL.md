---
name: nook-docker-alpine
category: "devops"
description: "Docker Alpine 3.21 patterns, multi-arch builds, and deployment for Nook on Zimaboard ARM64."
---

# 🐳 Docker Alpine — Nook

## Principles
- **Zero Google**: Pas de gcr.io, pas de Distroless, pas de libprotobuf-dev Debian
- **Alpine 3.21**: Base pour TOUS les services
- **UID/GID 1000**: Tous les containers en non-root (user nook)
- **Multi-arch**: amd64 + arm64 via musl-gcc et zig cc

## Dockerfile Pattern
```dockerfile
FROM alpine:3.21 AS runtime
RUN apk add --no-cache ca-certificates
RUN addgroup -S -g 1000 nook && adduser -S -u 1000 -G nook nook
USER nook
```

## Healthchecks
- **turn-server**: `HEALTHCHECK CMD pgrep turn-server || exit 1`
- **nook-backend**: `HEALTHCHECK CMD wget -qO- http://localhost:3000/health || exit 1`

## Build Commands
```bash
# Frontend
docker build -f Dockerfile.dev -t ghcr.io/mx10-ac2n/nook:dev .

# Backend (multi-arch)
docker buildx build --platform linux/amd64,linux/arm64   -f Dockerfile.release   -t ghcr.io/mx10-ac2n/nook-backend:dev   --push .
```

## Deployment (Zimaboard)
```bash
# Pull + restart
docker compose pull
docker compose up -d

# Check status
docker compose ps
docker compose logs --tail=50 nook
docker compose logs --tail=50 turn

# Test TURN
curl -v http://localhost:3478
```

## Common Issues
1. **Container unhealthy** → Check HEALTHCHECK in Dockerfile
2. **Permission denied** → Verify UID/GID 1000
3. **Config not mounted** → Check volume mount path
4. **User root** → Use su-exec in entrypoint for privilege drop
5. **Build fails** → Check Alpine dependencies (apk add)

## Volumes
- `DATA_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-data`
- `LOGS_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-logs`
- `TURN_CONFIG_DIR=/path/to/turn-config`

## Checklist
- [ ] Dockerfile Alpine 3.21
- [ ] UID/GID 1000 (addgroup + adduser)
- [ ] HEALTHCHECK fonctionnel
- [ ] Volumes montés correctement
- [ ] Zero Google dependencies
- [ ] Multi-arch builds (amd64 + arm64)
