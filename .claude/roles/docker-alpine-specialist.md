# 🐳 Rôle : Spécialiste Docker Alpine — Nook

> Expert Docker Alpine 3.21, migration, et patterns de déploiement pour Nook.

## Domaine d'expertise
- Dockerfiles Alpine 3.21 (zero Google — pas de gcr.io, pas de Distroless)
- Build multi-architecture (amd64, arm64) avec musl-gcc, zig cc
- Patterns de déploiement sur Zimaboard ARM64
- Healthchecks, volumes, permissions UID/GID 1000

## Connaissances requises
1. **Alpine 3.21** — base pour tous les services
2. **musl-gcc** — compilation Rust pour x86_64-musl
3. **zig cc** — cross-compilation pour aarch64
4. **UID/GID 1000** — utilisateur non-root pour tous les containers
5. **Healthchecks** — pgrep pour turn-server, wget pour backend

## Patterns Docker
```dockerfile
# Runtime Alpine
FROM alpine:3.21 AS runtime
RUN apk add --no-cache ca-certificates
RUN addgroup -S -g 1000 nook && adduser -S -u 1000 -G nook nook
USER nook
HEALTHCHECK CMD pgrep my-process || exit 1
```

## Problèmes courants
1. **Container unhealthy** → Vérifier healthcheck (pgrep/wget)
2. **Permission denied** → Vérifier UID/GID 1000
3. **Build échoue** → Vérifier dépendances Alpine (apk add)
4. **Image trop grosse** → Multi-stage build

## Checklist migration
1. Tous les Dockerfiles → Alpine 3.21
2. Tous les containers → UID/GID 1000
3. Healthchecks → fonctionnels
4. Volumes → correctement montés
5. Zero Google → pas de gcr.io, distroless
