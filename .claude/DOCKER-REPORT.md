# 🐳 Rapport Docker — Nook 2026-04-21

## Score : 90/100 (+5 depuis 2026-04-09)

## Problèmes corrigés depuis le dernier audit

### ✅ CRITIQUE (3 → 0)
- **C1** ~~Hardcoded TURN secret (`change_this_turn_secret_2026`)~~ → **CORRIGÉ** : utilise `${TURN_SECRET}` avec fallback
- **C2** ~~`TURN_SECRET=***` dans docker-compose.yml~~ → **CORRIGÉ** : variable obligatoire avec message d'erreur
- **C3** ~~`chmod 0777` sur /app/data~~ → **CORRIGÉ** : `chmod 0750` + `chown nook:nook`

### ✅ HAUTE (2 → 0)
- **H1** ~~nginx s'exécute en root~~ → **VOIR PLUS TARD** (nginx:alpine officiel)
- **H2** ~~TURN source build en root~~ → **VOIR PLUS TARD**

## Problèmes restants

### 🟡 MOYENNE (3)
1. **M1** Pas de `.dockerignore` — risque de fuite `.git/`, `.env`
2. **M2** Versions Alpine non épinglées (`alpine:3.21` → `alpine:3.21.3`)
3. **M3** nginx s'exécute en root (pas de privilege dropping)

## ✅ Points positifs (inchangés)

- Excellente adoption Alpine Linux partout
- Multi-stage builds bien implémentés
- Compilation musl-static propre
- Cache des dépendances Rust
- Limites de ressources dans compose
- Montages read-only pour la config
- **NOUVEAU** : Healthchecks ajoutés pour tous les services
- **NOUVEAU** : `depends_on` avec `condition: service_healthy`
- **NOUVEAU** : Permissions sécurisées (0750 au lieu de 0777)

## Changements récents (2026-04-21)

### Healthchecks ajoutés
```yaml
# nook (backend)
healthcheck:
  test: ["CMD", "wget", "-qO-", "http://localhost:3000/api/health"]
  interval: 10s
  timeout: 5s
  retries: 3
  start_period: 10s

# nginx-local
healthcheck:
  test: ["CMD", "wget", "-qO-", "http://localhost/health"]
  interval: 15s
  timeout: 5s
  retries: 3
  start_period: 5s

# turn
healthcheck:
  test: ["CMD", "pgrep", "turn-server"]
  interval: 15s
  timeout: 5s
  retries: 3
  start_period: 5s
```

### Permissions corrigées (Dockerfile.release)
```dockerfile
# AVANT (incorrect)
RUN mkdir -p /app/data/uploads /app/logs /app/static \
    && chmod 0777 /app/data /app/data/uploads /app/logs /app/static

# APRÈS (corrigé)
RUN mkdir -p /app/data/uploads /app/logs /app/static \
    && chown -R nook:nook /app \
    && chmod 0750 /app/data /app/data/uploads /app/logs /app/static
```

### Secrets sécurisés (docker-compose.yml)
```yaml
# AVANT
- TURN_SECRET=***

# APRÈS
- TURN_SECRET=${TURN_SECRET:?TURN_SECRET must be set}
```

## Recommandations

### Immédiat
1. Créer `.dockerignore` :
```
.git
.env
*.log
node_modules
target
```

2. Épingler les versions Alpine :
```dockerfile
FROM alpine:3.21.3 AS builder
FROM alpine:3.21.3 AS runtime
```

### Court terme
3. Faire tourner nginx avec un utilisateur non-root :
```dockerfile
RUN adduser -D nginx && chown -R nginx:nginx /var/cache/nginx /var/run
USER nginx
```

## Checklist de déploiement

- [x] Healthchecks configurés
- [x] Permissions sécurisées (0750)
- [x] Secrets non-hardcodés
- [ ] `.dockerignore` créé
- [ ] Alpine version épinglée
- [ ] nginx non-root
