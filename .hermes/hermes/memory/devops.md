# 🚀 Mémoire DEVOPS - CI/CD, Docker, Deploy — Nook

> **DERNIÈRE MISE À JOUR** : 2026-05-16
> CI/CD, Docker, Zimaboard, GitHub Actions

## 🏗️ Infrastructure

### Serveur de Production
- **Machine** : Zimaboard 832 + CasaOS
- **URL** : `http://192.168.1.192:6300` (HTTP) | `https://192.168.1.192:6443` (HTTPS nginx local)
- **Certificat** : Auto-signé (navigateur = provisionner exception)
- **OS** : Linux (CasaOS sur Zimaboard)
- **Port 8080** : scanservjs, pas Nook

### Docker Registry
- **URL** : `ghcr.io/mx10-ac2n/nook`
- **Images** :
  - `ghcr.io/mx10-ac2n/nook:dev` (développement — tag sur `develop`)
  - `ghcr.io/mx10-ac2n/nook:latest` (production — tag sur `main`)
  - `ghcr.io/mx10-ac2n/turn-server:dev`

### Déploiement local (CasaOS)
```bash
ssh root@192.168.1.192  # non autorisé dans ma config actuelle — préférer navigateur direct
cd /opt/data/Nook && git pull
docker compose down -v --rmi all --remove-orphans
docker compose up -d --build
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

## 🔄 GitHub Actions Workflows (ordre STRICT)

### ⚠️ Règles CI (free GitHub account)
1. **PAS de scheduled workflows** (pas de cron, pas d'auto-trigger sur schedule)
2. **Déclenchement MANUEL uniquement** : `gh workflow run "<nom>"`
3. **Ordre EXIGÉ** :
   - Etape 1 → Frontend (2==> 🎨 Frontend Build & Artifact)
   - Etape 2 → Backend (1==>🏗️ Backend Build & Artifact)
   - Etape 3 → Turn (3==> Turn-Server Build and Artifact)
   - Etape 4 → Docker (4==> 🐳 Docker Build & Push) — **SEULEMENT** après les 3 verts
4. **Vérifier gh run list** avant chaque déclenchement : filtrer par `headSha[:7]` pour éviter de confondre runs anciennes/succès
5. **Docker ne se lance JAMAIS avant que les 3 autres workflows soient `completed success`**

### Workflow Tracking
- Identifier les runs par `head_sha[:7]` — ne pas récupérer le status du top run (ancien success) au lieu du run de la commit actuelle
- Polling snapshot : attendre 35s-2min par workflow avant vérification (pas de sleep aveugle)

### Frontend.yml
- **Nom YAML** : `2==> 🎨 Frontend Build & Artifact`
- **ID** : 220018364
- **Node** : v20.x
- **Artifacts** : Static build vers `/build`
- **Durée typique** : ~35s

### Backend.yml
- **Nom YAML** : `1==>🏗️ Backend Build & Artifact`
- **ID** : 220018362
- **Rust** : stable (plus nightly)
- **Targets** : amd64 + arm64 (musl)
- **Durée typique** : ~4-9min

### Turn Build
- **Nom YAML** : `3==> Turn-Server Build and Artifact`
- **Build** : cargo build --target x86_64-unknown-linux-musl depuis `services/turn-rs/`

### Docker.yml
- **Nom YAML** : `4==> 🐳 Docker Build & Push`
- **ID** : 220018363
- **Plateformes** : linux/amd64, linux/arm64
- **Push** : ghcr.io/mx10-ac2n/nook:dev (sur develop)
- **Dépend de** : Backend.yml artifacts (binaire Rust)

## 🐳 Docker & Containers

### Services Nook
```yaml
# docker-compose.yml (extraits)
services:
  nook:
    image: ghcr.io/mx10-ac2n/nook:dev
    ports:
      - "6300:6300"
    
  turn:
    image: ghcr.io/mx10-ac2n/turn-server:dev
    ports:
      - "3478:3478/udp"
      - "3478:3478/tcp"
    volumes:
      - ${TURN_CONFIG_DIR:-./turn-config}:/etc/turn-server
```

### Problèmes Connus
- ❌ `hermes-web-ui` ne démarre pas (entrypoint-webui fail)
- ❌ Pollution `/opt/data` par multiples installs
- ⚠️ `turn-config` directory doit exister pour le volume mount

## 🔧 Outils CI

### Installés dans l'image Docker CI
- **git** : 2.47.3
- **node** : v20.x
- **npm** : 9.x
- **rustc** : stable
- **cargo** : stable
- **gh** : 2.x
- **musl-tools** : natifs sur runners (pas dans Dockerfile)

### Manquants / Problématiques
- ❌ **wasm-pack** : Échec compilation (nécessite Rust > 1.86)
- ❌ **docker** : Non disponible (utiliser GitHub Actions runners)
- ❌ **docker-compose** : Non disponible (utiliser GitHub Actions runners)
- ⚠️ **SSH CasaOS** : root SSH password/key non configuré dans ma session actuelle → navigateur direct utilisé

## 📦 Release & Versioning

### Stratégie
- **Version actuelle** : 0.5.0-beta.2
- **Branche dev** : `develop`
- **Releases** : Via GitHub Releases (⚠️ P2 - pas de releases automatiques)

### Création de Release
```bash
# Tag une release
git tag -a v0.5.0 -m "Release 0.5.0"
git push origin v0.5.0

# Créer release GitHub
gh release create v0.5.0 --title "v0.5.0" --notes "Release notes"
```

## 🔐 Secrets & Variables

### GitHub Secrets (à configurer)
- **GITHUB_TOKEN** : Automatiquement fourni par GitHub
- **DOCKER_TOKEN** : Pour push vers ghcr.io (si nécessaire)

### Variables d'Environnement
```bash
# Nook
export NOOK_ENV=development
export NOOK_DB_PATH=/opt/data/nook.db

# Rust
export RUSTUP_HOME=/opt/rust
export CARGO_HOME=/opt/cargo
```

## 📊 Monitoring & Health

### VérificationServices
```bash
# Health check (si endpoint disponible)
curl https://192.168.1.192:6443/health

# Logs (via SSH sur Zimaboard)
docker logs nook-app
docker logs nook-turn
```

### Métriques (P1 - à implémenter)
- Pas de monitoring actuel
- Pas de métriques exposées
- Voir `nook-monitoring-specialist.md` pour implémentation

## 🔄 Procédure de Déploiement

### Manuel (actuel)
1. Push sur `develop`
2. GitHub Actions build automatiquement
3. Image push vers `ghcr.io/mx10-ac2n/nook:dev`
4. SSH sur Zimaboard : `docker compose pull && docker compose up -d`
5. Vérifier les logs

### Automatisé (à faire)
- [ ] Webhook pour déploiement auto
- [ ] Blue-green deployment
- [ ] Rollback automatique si échec

## 📝 Notes de Session

- Tous les workflows GitHub sont ✅ GREEN (2026-05-16)
- CI stable sur Rust stable (plus nightly depuis session 50)
- E2EE fixes validés sur serveur CasaOS (commits 36eefe5c + f0a8c8d1)
- SSH CasaOS bloqué — navigateur direct utilisé pour diagnostic
- Pas de tests automatisés en CI (P1)

---
*Mettre à jour après chaque modification CI/CD ou déploiement*
