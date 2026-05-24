# 🚀 Contexte DevOps - Nook

> Mis à jour : 2026-05-24

## Infrastructure

- **Conteneurisation** : Docker multi-arch (amd64/arm64)
- **Base** : Alpine 3.21 (distroless pour Rust)
- **CI/CD** : GitHub Actions (free tier — pas de cron jobs)
- **Registry** : GHCR (GitHub Container Registry)
- **Déploiement** : CasaOS/Zimaboard (docker-compose)
- **HTTPS local** : nginx-alpine:6443 (cert auto-signé) pour WebRTC A/V
- **Test server** : 192.168.1.192:6300 (HTTP) / 6443 (HTTPS)

## GitHub Actions — Workflows

### Ordre d'exécution (manuel seulement, pas de schedule)
```
Frontend.yml → Backend.yml → Turn.yml → Docker.yml
```
Règle : **Docker uniquement après les 3 workflows précédents sont verts**.

### Anti-repeat pattern
- Toujours vérifier `gh run list --limit 5` avant de trigger un workflow
- Filtrer par `head_sha[:7]` pour suivre les runs de SON commit, pas les anciens verts
- Ne jamais re-runner un workflow qui a échoué → analyser les logs, fixer le code, re-commit

### Backend.yml
```yaml
Jobs:
  - Backend (amd64/arm64)
    - cargo check
    - cargo clippy -- -D warnings
    - cargo test
    - cargo build --release --target x86_64-unknown-linux-musl
    - Upload artifact
```

### Frontend.yml
```yaml
Jobs:
  - Frontend
    - npm ci (SANS --omit optional — Rollup a besoin de ses dépendances optionnelles)
    - npm run build
    - Upload artifact
```

### Docker.yml
```yaml
Jobs:
  - Build & Push multi-arch
    - Plateformes: linux/amd64, linux/arm64
    - Conditionné : tous les workflows amont sont verts
    - Base: Alpine 3.21 + musl backend + nginx-alpine frontend
```

## Docker Compose (Production)

```yaml
services:
  nook:
    image: ghcr.io/mx10-ac2n/nook:latest
    ports:
      - "6300:3000"   # HTTP
      - "6443:443"    # HTTPS (nginx)
    environment:
      - NOOK_ENV=production
      - DATABASE_URL=sqlite:/data/nook.db
      - TURN_SECRET=${TURN_SECRET}
    volumes:
      - ./nook-data:/data
    restart: unless-stopped
```

**Note** : `NOOK_ENV=development` sur le serveur de test pour autoriser les cookies HTTP.

## WebSocket — Chemin Frontend

Le backend expose le WebSocket sur `/api/webrtc/ws` (nesté sous `/api`).
**Référence brutale** : si `/webrtc/ws` retourne du HTML (200 avec content-type: text/html) au lieu d'un upgrade WebSocket, c'est que tu as oublié le préfixe `/api`.

## Secrets

| Variable | Usage |
|----------|-------|
| `JWT_SECRET` | Signer les JWT tokens |
| `TURN_SECRET` | Authentification TURN server |
| `ADMIN_INITIAL_PASSWORD` | Mot de passe admin initial |
| `VAPID_PRIVATE_KEY` | Push notifications |
| `VAPID_PUBLIC_KEY` | Push notifications |

**Jamais en dur** — utiliser GitHub Secrets + `.env` local.

## Points Critiques DevOps

### ✅ Corrections Récentes (2026-05-24)
- **Frontend** : 6 fichiers WebSocket corrigés de `/webrtc/ws` → `/api/webrtc/ws`
- **Backend** : Build musl fonctionne, clippy warnings fixés
- **CI** : Ordre strict Frontend → Backend → Turn → Docker, pas de cron
- **Tests serveur** : Réactions ✅, Upload ✅, WebSocket temps réel ✅

### ⚠️ À Surveiller
- **arm64 turn-rs** : BuildSometimes non disponible sur free tier → `continue-on-error`
- **Healthchecks** : Nginx fail si `/health` n'est pas sur la racine — Docker HEALTHCHECK migré
- **Volumes** : Persistence SQLite + uploads sur volume Docker nommé
- **SSG dynamic** : Adapter-static génère HTML, toutes les routes doivent être pré-rendues

## Commandes Utiles

```bash
# Build local Nook
docker-compose build

# Push vers GHCR
docker-compose push

# Déploiement Zimaboard/CasaOS
ssh user@zimaboard "cd /opt/nook && docker-compose up -d"

# Logs temps réel
docker logs nook-backend --tail 200 -f
docker logs nook-frontend --tail 200 -f

# Healthcheck
curl http://localhost:6300/api/health

# Test WebSocket
python3 -c "
import asyncio, websockets, requests, json
r = requests.post('http://localhost:6300/api/auth/login', json={'username':'admin','password':'xxx'})
cookie = '; '.join(f'{c.name}={c.value}' for c in r.cookies)
async with websockets.connect('ws://localhost:6300/api/webrtc/ws', additional_headers={'Cookie': cookie}) as ws:
    await ws.send(json.dumps({'type':'ping'}))
    print(await asyncio.wait_for(ws.recv(), timeout=2))
"

# Git token
export GITHUB_TOKEN=$(cat /tmp/.git_token)
gh workflow run Frontend.yml --repo $GITHUB_REPOSITORY
```

## Outils MCP DevOps
- **GitHub CLI** (`gh`) : Workflow triggers, secrets management
- **SocratiCode** : Infrastructure as code analysis
- **Docker CLI** : Build, push, logs, compose
