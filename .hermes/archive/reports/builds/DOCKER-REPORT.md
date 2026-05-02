# 🐳 Rapport Docker — Nook 2026-04-25

## Score : 92/100 (+2 depuis 2026-04-21)

---

## Problèmes corrigés depuis le dernier audit

### ✅ MOYENNE (M1 → CORRIGÉ dans PR #32)
- **M1** ~~Pas de `.dockerignore`~~ → **CRÉÉ** : protection contre les fuites de secrets
- **Fichier** : `.dockerignore` (nouveau)

### ✅ MOYENNE (M2 → NON APPLICABLE)
- **M2** ~~Versions Alpine non épinglées~~ → **NON APPLICABLE**
- Les Dockerfiles utilisent maintenant **Debian/Distroless** au lieu d'Alpine :
  - `Dockerfile` : `rust:1.88-bookworm` + `debian:bookworm-slim` + `gcr.io/distroless/cc-debian12`
  - `Dockerfile.release` : `debian:bookworm-slim` + `gcr.io/distroless/cc-debian12`

---

## Problèmes restants

### 🟢 MOYENNE (0)
*Aucun problème restant — M3 et M4 vérifiés OK*

---

## ✅ Points positifs (inchangés)

- Excellente adoption Debian/Distroless (plus d'Alpine)
- Multi-stage builds bien implémentés
- Compilation propre avec `distroless/cc-debian12`
- Cache des dépendances Rust
- Limites de ressources dans compose
- Montages read-only pour la config
- **Healthchecks** ajoutés pour tous les services (PR #29)
- **`depends_on`** avec `condition: service_healthy` (PR #29)
- **Permissions sécurisées** (0750 au lieu de 0777) (PR #30)
- **`.dockerignore`** créé (PR #32) — empêche fuite `.git/`, `.env`, etc.
- **M3 (nginx non-root)** → **VÉRIFIÉ OK** (pas de service nginx dans docker-compose, isolation conteneur)
- **M4 (TURN build)** → **VÉRIFIÉ OK** (utilise `gcr.io/distroless/cc-debian12`)

---

## Changements récents (2026-04-25)

### PR #32 (M1, M9, H6)
- ✅ **`.dockerignore`** créé :
  - Exclut `.git/`, `.env*`, `*.log`, `node_modules/`, `target/`
  - Protège contre les fuites de secrets dans les images
  - Réduit la taille des builds

### PR #30 (C1-C4, déjà mergé)
- ✅ Permissions sécurisées (0750)
- ✅ Secrets non-hardcodés

### PR #29 (Healthchecks, déjà mergé)
- ✅ Healthchecks pour tous les services
- ✅ `depends_on` avec `service_healthy`

---

## Recommandations

### Immédiat
- [x] ~~Créer `.dockerignore`~~ → **FAIT** (PR #32)
- [x] ~~Épingler les versions Alpine~~ → **NON APPLICABLE** (plus d'Alpine)

### Court terme
- [x] **M3** Vérifier que nginx ne tourne pas en root → **VÉRIFIÉ OK** (pas de service nginx)
- [x] **M4** Vérifier TURN build → **VÉRIFIÉ OK** (utilise distroless)

---

## Checklist de déploiement

- [x] Healthchecks configurés
- [x] Permissions sécurisées (0750)
- [x] Secrets non-hardcodés
- [x] `.dockerignore` créé
- [x] Alpine → Debian/Distroless (migration complète)
- [x] nginx non-root → **VÉRIFIÉ OK** (pas de service nginx)
- [x] TURN build → **VÉRIFIÉ OK** (distroless)
