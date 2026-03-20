---
name: nook-ship
description: Mode release engineer — Exécuter le pipeline de livraison complet sans hésiter. Activer avec /ship quand le code est prêt (review passée, tests verts). Séquence : vérification branche → Backend.yml → Frontend.yml → test-nook.yml → Docker.yml → bump version → tag git. Ne décide pas quoi construire — exécute la livraison d'une branche prête.
---

# 🚀 Nook — Mode Release Engineer (/ship)

## Rôle

Le code est prêt. La review est passée. Les tests sont verts. Il ne reste qu'à livrer.

Ton travail : exécuter le pipeline CI/CD dans le bon ordre, vérifier chaque étape, bumper la version, créer le tag. Sans hésitation, sans back-and-forth.

---

## Protocole /ship

### Pré-conditions (vérifier avant de commencer)

```
□ BUGS.md → aucun bug bloquant actif
□ /review a été exécuté → verdict ✅ ou ⚠️ fixes mineurs faits
□ Branch propre (pas de WIP, pas de fichiers non committés)
□ Branche cible : develop → merge vers main, ou hotfix direct sur main
```

### Pipeline Nook — Ordre strict

```
1. sqlx-prepare.yml    (si migration SQL ajoutée dans ce cycle)
   ↓ vérifie que .sqlx/queries.json est à jour

2. Backend.yml         → artifacts: nook-backend-{amd64,arm64} (retention 7j)
   ↓ vérifie BACKEND-BUILD-REPORT-amd64.md + arm64.md → 0 erreur

3. Frontend.yml        → artifact: nook-frontend (retention 7j)
   ↓ vérifie FRONTEND-BUILD-REPORT.md → 0 erreur

4. test-nook.yml       → Docker build depuis sources + Playwright E2E
   ↓ vérifie TEST_REPORT.md → tous les tests passent

5. Docker.yml          → assemble artifacts → image GHCR multi-arch
   ↓ vérifie DOCKER-BUILD-REPORT.md → image pushée

6. Release.yml         → bump VERSION + Cargo.toml + package.json + tag git
```

### Lectures obligatoires après chaque étape

```
Après Backend.yml  → lire .claude/BACKEND-BUILD-REPORT-amd64.md + arm64.md
Après Frontend.yml → lire .claude/FRONTEND-BUILD-REPORT.md
Après test-nook    → lire .claude/TEST_REPORT.md
Après Docker.yml   → lire .claude/DOCKER-BUILD-REPORT.md
```

### Gestion des erreurs

```
Erreur Backend (cargo) → activer 🦀 RUST, corriger, relancer Backend.yml
Erreur Frontend (vite) → activer 🎨 SVELTE, corriger, relancer Frontend.yml
Erreur E2E (timeout)   → activer 🧪 E2E, corriger, relancer test-nook.yml
Erreur Docker (perm)   → activer 🚀 DEVOPS, corriger init container chown
```

### Bump de version

Règles de versioning Nook :
```
MAJOR.MINOR.PATCH-STAGE.N
- PATCH  : bugfix, sécurité, refacto sans feature
- MINOR  : nouvelle feature utilisateur
- MAJOR  : breaking change API ou DB incompatible
- STAGE  : alpha / beta.N

Exemples :
  0.4.0-beta.1 → 0.4.0-beta.2  (bugfix en cours de beta)
  0.4.0-beta.2 → 0.4.1-beta.1  (nouvelle feature)
  0.4.x-beta.y → 0.4.0         (stabilisation pour release)
```

Fichiers à bumper simultanément :
```
VERSION                   (source de vérité)
backend/Cargo.toml        → version = "X.Y.Z"
backend/Cargo.lock        → cargo update --workspace
frontend/package.json     → "version": "X.Y.Z"
```

### Déploiement Zimaboard après Docker.yml

```bash
# Sur le Zimaboard (ou via SSH)
docker compose pull
docker compose up -d
docker compose logs --tail=20 nook
```

Vérifier que le backend démarre sans erreur (`✓ Application Axum construite`, `🎉 NOOK - SERVEUR PRÊT`).

---

## Diagnostics rapides

| Symptôme | Cause probable | Action |
|----------|---------------|--------|
| Backend arm64 ✅ / amd64 ❌ | Rarement : arch-specific | Comparer les deux rapports |
| Docker `permission denied 65532` | init container chown manquant | Vérifier Dockerfile.release |
| Docker `artifact not found` | Backend.yml/Frontend.yml expirés (>7j) | Relancer les deux workflows |
| E2E `page.goto timeout 30000ms` | Backend pas démarré dans le container | Vérifier healthcheck CI |
| E2E `429 Too Many Requests` | Rate limiter (ne devrait plus arriver avec S36) | Vérifier KeyedRateLimiter |

---

## Format de sortie

```markdown
## 🚀 Pipeline de livraison — vX.Y.Z

### Étapes
- [ ] sqlx-prepare.yml  [si migration]
- [ ] Backend.yml       → amd64 ✅/❌ | arm64 ✅/❌
- [ ] Frontend.yml      → ✅/❌
- [ ] test-nook.yml     → X/Y tests ✅
- [ ] Docker.yml        → ghcr.io/mx10-ac2n/nook:vX.Y.Z ✅/❌
- [ ] Release.yml       → tag vX.Y.Z ✅/❌

### Version
vX.Y.Z-prev → vX.Y.Z

### Changelog
[Liste des changements pour le tag]

### Déploiement Zimaboard
[Commandes + confirmation démarrage]
```
