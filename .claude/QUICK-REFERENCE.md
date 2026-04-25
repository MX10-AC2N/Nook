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

### Common Issues (2026-04-21)

1. **Build fails** → Vérifier les imports manquants (notifyPoll, notifyCalendar, etc.)
2. **Docker unhealthy** → Vérifier les healthchecks (pgrep turn-server, wget backend)
3. **Chess pas de mouvement** → Vérifier `this.myColor` (pas `this.myColor()`)
4. **Emojis petits** → Vérifier `.emoji-only` et inline `.emoji` CSS
5. **Notifications** → AudioContext pour HTTP/LAN, Web Push pour HTTPS
6. **Tests flaky** → Vérifier que les tests sont DANS un describe, pas à la racine
7. **Playwright page undefined** → Tests doivent être DANS un describe, pas à la racine
8. **PGN non affiché** → Vérifier `chessStore.toPgn()` et `move_history.san`
9. **Chart.js 404** → Import dynamique `await import('chart.js')`
10. **Healthcheck failed** → Vérifier `docker compose ps` (doit montrer `(healthy)`)
11. **Secrets en dur** → `git grep -n "change_this\|secret_2026" .`
12. **Permissions 0777** → `ls -la /app/data` doit montrer `drwxr-x---`

---

## Variables d'environnement (.env)

```bash
DATA_DIR=/media/ac2-cloud/volume_docker_nook/nook-data
LOGS_DIR=/media/ac2-cloud/volume_docker_nook/nook-logs
TURN_CONFIG_DIR=/path/to/turn-config
PORT=6300
TZ=Europe/Paris

# ⚠️ SECURITÉ — Ne jamais laisser par défaut !
TURN_SECRET=MonSecretSuperSecureGenereAleatoirement  # openssl rand -base64 32
ADMIN_INITIAL_PASSWORD=UnAutreSecretChangeLePremierLogin  # ou vide (auto-généré)

# Optionnel
GIPHY_API_KEY=  # Laisser vide pour désactiver
VAPID_PRIVATE_KEY=  # Laisser vide pour désactiver Web Push
VAPID_PUBLIC_KEY=
```

**Génération de secrets :**
```bash
# TURN_SECRET (32 octets, base64)
openssl rand -base64 32

# ADMIN_INITIAL_PASSWORD (16 caractères alphanumériques)
openssl rand -base64 16
```

---

## Image Docker

- Backend: `ghcr.io/mx10-ac2n/nook:dev`
- TURN: `ghcr.io/mx10-ac2n/turn-server:dev`
- Toutes Alpine 3.21, UID/GID 1000
- **NOUVEAU** : Healthchecks ajoutés pour tous les services !

---

## Architecture

- Backend: Rust/Axum, SQLite, musl-gcc
- Frontend: SvelteKit 5 Runes, $derived, {#if}
- TURN: turn-rs, config.toml avec `${TURN_SECRET}`
- Zimaboard ARM64
- **NOUVEAU** : Healthchecks pour tous les services
  - nook: `wget http://localhost:3000/api/health`
  - nginx: `wget http://localhost/health` (port 80, sans SSL)
  - turn: `pgrep turn-server`

---

## 🔒 Sécurité (Points critiques corrigés !)

### ✅ Secrets en dur — TOUS CORRIGÉS
- **C1** ~~`secret = "change_this_turn_secret_2026"`~~ → `${TURN_SECRET}` + fallback
- **C2** ~~Log admin password~~ → Supprimé (main.rs:152)
- **C3** ~~`TURN_SECRET=***`~~ → `${TURN_SECRET:?...}` obligatoire
- **C4** ~~`chmod 0777`~~ → `chmod 0750` + `chown nook:nook`

### Tests de sécurité recommandés
```bash
# 1. Vérifier que TURN_SECRET n'est pas en dur
git grep -rn "change_this_turn_secret" .

# 2. Vérifier les permissions des volumes
docker exec nook ls -la /app/data | grep "drwxr-x---"

# 3. Vérifier qu'aucun mot de passe n'est dans les logs
docker logs nook 2>&1 | grep -i "password" | grep -v "change_password"

# 4. Tester avec une vraie clé secrète
openssl rand -base64 32
# Puis mettre à jour .env avec :
# TURN_SECRET=<votre_clé_secrète>

# 5. Vérifier les healthchecks
docker compose ps
# Doit montrer : nook healthy, nook-nginx-local healthy, nook-turn healthy
```

---

## 🐳 Healthchecks (NOUVEAU !)

### Configuration
```yaml
# docker-compose.yml
services:
  nook:
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:3000/api/health"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 10s

  nginx-local:
    depends_on:
      nook:
        condition: service_healthy  # ← NOUVEAU !
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost/health"]
      interval: 15s
      timeout: 5s
      retries: 3
      start_period: 5s

  turn:
    healthcheck:
      test: ["CMD", "pgrep", "turn-server"]
      interval: 15s
      timeout: 5s
      retries: 3
      start_period: 5s
```

### Vérification
```bash
docker compose up -d
docker compose ps
# Attendu : (healthy) pour tous les services

# Test manuel
curl -s http://localhost:3000/api/health  # Backend
curl -s http://localhost/health             # Nginx HTTP
docker exec nook-turn pgrep turn-server  # TURN process
```

---

## 📂 Fichiers importants (.claude/)

| Fichier | Contenu |
|---------|----------|
| `.claude/DOCKER-REPORT.md` | Score 90/100 — Healthchecks ✅, permissions ✅ |
| `.claude/SECURITY-REPORT.md` | Score 92/100 — 0 secret en dur ✅, CSP ✅ |
| `.claude/GLOBAL-AUDIT-2026-04-21.md` | Rapport consolidé (84/100, +9) |
| `.claude/DEPENDENCES-REPORT.md` | Score 72/100 — 4 deps supprimées ✅ |
| `.claude/rules/secrets-management.md` | ⚠️ NOUVEAU — Gestion des secrets |
| `.claude/QUICK-REFERENCE.md` | ⚠️ CETTE PAGE — Référence rapide |

---

## 🔗 Audit en cours
| Domaine | Score | Progression |
|---------|-------|------------|
| 🔒 Sécurité | **92/100** | +10 (H3, H5 fixed) |
| 🐳 Docker | **90/100** | +5 (healthchecks + permissions) |
| 📦 Dépendances | **72/100** | +2 (H6 - 4 deps removed) |
| **GLOBAL** | **84/100** | **+9** |

**Prochaines étapes :**
1. ~~Restreindre CORS en production (H2)~~ → H3, H5, H6 fixed in PR #31!
2. ~~Renforcer CSP — retirer 'unsafe-inline' (H3)~~ ✅ Fixed
3. ~~Sanitiser Icon.svelte — éviter `{@html}` (H5)~~ ✅ Fixed  
4. ~~Supprimer dépendances Rust inutilisées (H6)~~ ✅ Fixed
5. **H2** — Restreindre CORS en production
6. **M1** — Créer `.dockerignore`
7. **M2** — Épingler versions Alpine
8. **M9** — Mettre à jour `chacha20poly1305`
