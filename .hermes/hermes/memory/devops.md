# 🚀 Mémoire DEVOPS - CI/CD, Docker, Deploy

> **DERNIÈRE MISE À JOUR** : 2026-05-04
> CI/CD, Docker, Zimaboard, GitHub Actions

## 🏗️ Infrastructure

### Serveur de Production
- **Machine** : Zimaboard 832
- **URL** : `https://192.168.1.192:6443`
- **Certificat** : Auto-signé (accepter le risque dans le navigateur)
- **OS** : Linux (Ubuntu/Debian based)

### Docker Registry
- **URL** : `ghcr.io/mx10-ac2n/nook`
- **Images** :
  - `ghcr.io/mx10-ac2n/nook:latest` (multi-arch)
  - `ghcr.io/mx10-ac2n/nook:dev` (development)
  - `ghcr.io/mx10-ac2n/turn-server:dev` (TURN server)

## 🔄 GitHub Actions Workflows

### Backend.yml
- **ID** : 220018362
- **Fonction** : Build Rust amd64/arm64
- **Rust version** : nightly (ligne 34 du workflow)
- **Artifacts** : Binary compilé
- **Trigger** : Push sur develop, PR

### Frontend.yml
- **ID** : 220018364
- **Fonction** : Build SvelteKit
- **Node version** : v20.x
- **Artifacts** : Static build

### Docker.yml
- **ID** : 220018363
- **Fonction** : Build & push image multi-arch
- **Dépendance** : Nécessite Backend.yml en premier pour les changements Rust
- **Plateformes** : linux/amd64, linux/arm64

### Autres Workflows
- **turn.yml** : Build turn-server pour both arches
- **sqlx-prepare.yml** : Génère query.sql (échoue actuellement)
- **e2e-single.yml** : Tests E2E (106 skippés)

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

### Installés dans l'image Docker
- **git** : 2.47.3
- **node** : v20.19.2
- **npm** : 9.2.0
- **rustc** : 1.85.0 (mais CI utilise nightly)
- **cargo** : 1.85.0
- **gh** : 2.46.0

### Manquants / Problématiques
- ❌ **wasm-pack** : Échec compilation (needs Rust > 1.86)
- ❌ **docker** : Non disponible (utiliser GitHub Actions)
- ❌ **docker-compose** : Non disponible (utiliser GitHub Actions)

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

- Tous les workflows GitHub sont ✅ GREEN (2026-05-03)
- Rust nightly 1.97.0 installé en CI
- hermes-web-ui en état critique (à fixer en priorité)
- Pas de tests automatisés en CI (P1)

---
*Mettre à jour après chaque modification CI/CD ou déploiement*
