# 🐳 DOCKER.md — Référence Docker & CI du projet Nook

> Fichier dédié aux règles Docker, pièges connus et architecture CI.  
> Né de 3 sessions de debugging intensif (2026-02-21).  
> **Lire avant toute modification de Dockerfile ou workflow.**

---

## 🏗️ Architecture des deux Dockerfiles

### `Dockerfile` — Build depuis les sources
**Utilisé par** : `test-nook.yml` (CI intégration) + `docker-compose` (local/dev)  
**Stratégie** : `cargo-chef` pour le cache des dépendances, compilation complète dans Docker

```dockerfile
FROM rust:1.88-bookworm AS chef      # ← rust:1.88 minimum (voir règle #3)
RUN cargo install cargo-chef --locked
# planner → cook → builder → prep → distroless
```

### `Dockerfile.release` — Binaires pré-compilés
**Utilisé par** : `Docker.yml` (production) + `ci-new2.yml` (CI complet)  
**Stratégie** : pas de Rust dans l'image, binaires livrés par `Backend.yml`

```dockerfile
FROM debian:bookworm-slim AS prep
ARG TARGETARCH
COPY backend/nook-backend-${TARGETARCH} /app/nook-backend  # amd64 ou arm64
```

---

## 🚨 RÈGLES CRITIQUES — À NE JAMAIS VIOLER

### Règle #1 : JAMAIS `--platform=$BUILDPLATFORM` sur le builder Rust

```dockerfile
# ❌ BANNI — cause l'erreur proc-macro async-trait
FROM --platform=$BUILDPLATFORM rust:1.88-bookworm AS builder

# ✅ CORRECT — pas de flag platform sur le builder source
FROM rust:1.88-bookworm AS builder
```

**Pourquoi** : `$BUILDPLATFORM` active le mode multi-plateforme de BuildKit. Les
proc-macros (`async-trait`, `serde_derive`, `tokio-macros`...) sont alors compilées
pour une plateforme différente de la target → incompatibilité au link.

**Uniquement dans `Dockerfile.release`** le multi-plateforme est OK car `TARGETARCH`
est utilisé juste pour sélectionner un binaire déjà compilé, pas pour compiler.

---

### Règle #2 : JAMAIS la technique du "dummy fn main()" pour le cache

```dockerfile
# ❌ BANNI — cause la même erreur proc-macro
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src
COPY backend/ ./
RUN cargo build --release --bin nook-backend  # ← CRASH proc-macro ici

# ✅ CORRECT — utiliser cargo-chef
FROM rust:1.88-bookworm AS chef
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/nook/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY backend/ .
RUN cargo build --release --bin nook-backend
```

**Pourquoi** : `cargo build` sans `--bin` compile les dépendances dans un contexte
de crate différent. Quand le vrai build `--bin nook-backend` arrive, Cargo redétecte
des proc-macros incompatibles avec les artifacts en cache → crash.
`cargo-chef` analyse le graphe réel du projet avant de compiler les deps, garantissant
la cohérence complète.

---

### Règle #3 : Rust minimum 1.88

```dockerfile
# ❌ TROP VIEUX
FROM rust:1.80-bookworm AS builder  # home@0.5.12 exige 1.88
FROM rust:1.85-bookworm AS builder  # home@0.5.12 exige 1.88

# ✅ CORRECT
FROM rust:1.88-bookworm AS builder
```

**Pourquoi** : `home@0.5.12` (dépendance transitive) déclare `rust-version = "1.88"`
dans son manifeste. Cargo refuse de compiler avec une version inférieure.
De plus, `edition2024` (utilisé par certaines dépendances crypto) exige rustc 1.85+.

---

### Règle #4 : JAMAIS `BUILDKIT_INLINE_CACHE` dans docker-compose pour CI

```yaml
# ❌ BANNI dans docker-compose.yml pour les builds CI
build:
  args:
    BUILDKIT_INLINE_CACHE: "1"   # active le mode multi-plateforme → bug proc-macro

# ✅ CORRECT
build:
  context: .
  dockerfile: Dockerfile
  # pas de BUILDKIT_INLINE_CACHE
```

**Pourquoi** : `BUILDKIT_INLINE_CACHE: "1"` active implicitement des comportements
de cache cross-plateforme dans BuildKit, qui réintroduit le même problème que
`--platform=$BUILDPLATFORM`.

---

### Règle #5 : JAMAIS de `ARG` dans les chemins `COPY`

```dockerfile
# ❌ BANNI — BuildKit n'interpole PAS les ARG dans les sources COPY
ARG BACKEND_PATH=backend
COPY ${BACKEND_PATH}/Cargo.toml ./   # cherche '/Cargo.toml' → not found

# ✅ CORRECT — chemins hardcodés
COPY backend/Cargo.toml ./
```

**Uniquement exception** : `COPY backend/nook-backend-${TARGETARCH}` dans
`Dockerfile.release` fonctionne car `TARGETARCH` est un ARG système de BuildKit
(pas un ARG utilisateur), et il est dans la destination, pas la source.

---

### Règle #6 : `npm_net` externe = crash en CI

```yaml
# ❌ BANNI dans docker-compose.yml
networks:
  npm_net:
    external: true   # n'existe pas sur les runners GitHub → crash immédiat
  nook-network:
    driver: bridge

# ✅ CORRECT
networks:
  nook-network:
    driver: bridge
    name: nook-network
```

---

### Règle #7 : `env_file: .env` = crash en CI

```yaml
# ❌ BANNI dans docker-compose.yml pour CI
env_file:
  - .env    # fichier absent sur les runners GitHub → crash

# ✅ CORRECT — variables explicites avec valeurs par défaut
environment:
  - DATABASE_URL=sqlite:/app/data/nook.db
  - STATIC_FILES_DIR=/app/static
  - UPLOADS_DIR=/app/data/uploads
```

---

## 📦 Architecture CI — 5 workflows

### Flow manuel (développement)

```
1. Backend.yml   → artifacts : nook-backend-amd64 + nook-backend-arm64
2. Frontend.yml  → artifact  : nook-frontend
         ↓
3. test-nook.yml → Docker build depuis sources, tests API + Playwright
         ↓ si OK
4. Docker.yml    → assemble 1+2 dans Dockerfile.release → GHCR
```

### Flow automatique (CI sur push)

```
ci-new2.yml : fmt → backend (matrix) + frontend → docker
```

### Flow release

```
release.yml (choix patch/minor/major)
  → bumpe VERSION + Cargo.toml + package.json
  → crée tag git vX.Y.Z
  → puis lancer manuellement les 4 workflows ci-dessus
```

---

## 🏷️ Nommage des artifacts

| Workflow | Artifact uploadé | Chemin dans artifact |
|----------|-----------------|---------------------|
| `Backend.yml` | `nook-backend-x86_64-unknown-linux-gnu` | `nook-backend-amd64` |
| `Backend.yml` | `nook-backend-aarch64-unknown-linux-gnu` | `nook-backend-arm64` |
| `Frontend.yml` | `nook-frontend` | `index.html` + assets |

**Règle** : les noms d'artifacts sont **fixes et stables**. Ne jamais y ajouter
`${{ github.sha }}` ou autre variable — `Docker.yml` les télécharge par nom exact.

---

## 🏷️ Tags Docker produits par `Docker.yml`

```
ghcr.io/mx10-ac2n/nook:v0.5.1      ← version sémantique (badge README)
ghcr.io/mx10-ac2n/nook:v0.5        ← majeur.mineur
ghcr.io/mx10-ac2n/nook:latest      ← uniquement sur branch main
ghcr.io/mx10-ac2n/nook:sha-abc1234 ← SHA court pour traçabilité
```

La version est lue depuis le fichier `VERSION` à la racine du repo.

---

## 🔬 Diagnostic rapide des erreurs Docker

### `cannot produce proc-macro for async-trait`
→ Vérifier : `--platform=$BUILDPLATFORM` présent ? Supprimer.  
→ Vérifier : technique "dummy fn main()" ? Remplacer par cargo-chef.  
→ Vérifier : `BUILDKIT_INLINE_CACHE: "1"` dans docker-compose ? Supprimer.  
→ Vérifier : `Cargo.lock` à jour avec les versions du `Cargo.toml` ? `cargo update`.

### `home@0.5.12 requires rustc 1.88`
→ Changer l'image builder : `rust:1.88-bookworm`

### `failed to calculate checksum... not found`
→ Vérifier les `COPY` avec `ARG` dans le chemin source. Hardcoder les chemins.

### `npm_net` network not found
→ Supprimer le réseau externe de `docker-compose.yml`.

### `env_file .env not found`
→ Supprimer `env_file:` et mettre les variables directement dans `environment:`.

---

## 📊 Image finale — caractéristiques

| Propriété | Valeur |
|-----------|--------|
| Base image | `gcr.io/distroless/cc-debian12:nonroot` |
| Architectures | `linux/amd64`, `linux/arm64` |
| User | `nonroot` (uid 65532) — pas de root |
| Shell | Aucun (distroless) |
| Taille cible | ~8-15 MB |
| Libs incluses | libsqlite3, libsodium, libssl, libcrypto, ca-certificates |
| Healthcheck | `curl -sf http://localhost:3000/health` |
| Port exposé | 3000 (mappé 6300:3000 en local) |
