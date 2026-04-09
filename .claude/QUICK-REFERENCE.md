# 🚀 Quick Reference — Nook

## Commandes essentielles

### Build & Test
```bash
# Frontend
cd frontend && npm run build && npm run lint && npx playwright test --list

# Backend
cd backend && cargo check --quiet && cargo test --quiet

# Docker
docker compose pull && docker compose up -d
docker compose ps
docker compose logs -f <service>
```

### Git Workflow
```bash
git fetch origin develop
git pull origin develop
git add -A && git commit -m "message"
git push origin develop
```

### Common Issues
1. **Build fails** → Vérifier les imports manquants (notifyPoll, notifyCalendar, etc.)
2. **Docker unhealthy** → Vérifier les healthchecks (pgrep turn-server, wget backend)
3. **Chess pas de mouvement** → Vérifier `this.myColor` (pas `this.myColor()`)
4. **Emojis petits** → Vérifier `.emoji-only` et inline `.emoji` CSS
5. **Notifications** → AudioContext pour HTTP/LAN, Web Push pour HTTPS

## Variables d'environnement (.env)
```
DATA_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-data
LOGS_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-logs
TURN_CONFIG_DIR=/path/to/turn-config
PORT=6300
TZ=Europe/Paris
```

## Image Docker
- Backend: `ghcr.io/mx10-ac2n/nook:dev`
- TURN: `ghcr.io/mx10-ac2n/turn-server:dev`
- Toutes Alpine 3.21, UID/GID 1000

## Architecture
- Backend: Rust/Axum, SQLite, musl-gcc
- Frontend: SvelteKit 5, $derived, {#if}
- TURN: turn-rs, config.toml
- Zimaboard ARM64
