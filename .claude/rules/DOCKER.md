# 🐳 DOCKER.md — Règles Docker & CI

> Né de 7 sessions de debugging. Lire avant toute modification Dockerfile ou workflow.

---

## 🏗️ Deux Dockerfiles

| Fichier | Utilisé par | Stratégie |
|---------|-------------|-----------|
| `Dockerfile` | `test-nook.yml` + `docker-compose` local | `cargo-chef` + compilation complète |
| `Dockerfile.release` | `Docker.yml` | Binaires pré-compilés par `Backend.yml` |

---

## 🚨 RÈGLES CRITIQUES — NE JAMAIS VIOLER

### R1 — JAMAIS `--platform=$BUILDPLATFORM` sur le builder Rust
```dockerfile
# ❌ → proc-macro async-trait/serde_derive incompatibles
FROM --platform=$BUILDPLATFORM rust:1.88-bookworm AS builder
# ✅
FROM rust:1.88-bookworm AS builder
```

### R2 — JAMAIS la technique "dummy fn main()" pour le cache
```dockerfile
# ❌ → crash proc-macro au vrai build
RUN echo "fn main() {}" > src/main.rs && cargo build --release
# ✅ → utiliser cargo-chef
```

### R3 — Rust minimum 1.88
```dockerfile
# ❌ home@0.5.12 exige 1.88
FROM rust:1.85-bookworm AS builder
# ✅
FROM rust:1.88-bookworm AS builder
```

### R4 — JAMAIS copier `.cargo/config.toml` dans le build Docker
```dockerfile
# ❌ config.toml spécifie un linker externe non installé dans Docker
#    → Cargo détecte cross-compilation → proc-macros incompatibles
COPY backend/ ./   # copie .cargo/config.toml → CRASH

# ✅ copier explicitement sans .cargo/
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/.sqlx ./.sqlx
# .cargo/ intentionnellement exclu
```

### R5 — JAMAIS `npm_net` réseau externe dans docker-compose
```yaml
# ❌ n'existe pas sur les runners GitHub
networks:
  npm_net:
    external: true
# ✅
networks:
  nook-network:
    driver: bridge
    name: nook-network
```

### R6 — JAMAIS `env_file: .env` dans docker-compose (CI)
```yaml
# ❌ .env absent sur runners → crash
env_file: .env
# ✅ variables explicites avec défauts
environment:
  - DATABASE_URL=sqlite:/app/data/nook.db
```

### R7 — distroless nonroot (uid 65532) ne peut pas écrire dans les volumes root
```yaml
# ❌ Docker crée les volumes avec uid=root → Permission denied
volumes:
  - nook-data:/app/data
# ✅ init container qui chown avant le démarrage
services:
  nook-init:
    image: alpine:3
    command: ["sh", "-c", "chown -R 65532:65532 /app/data /app/logs"]
    volumes:
      - nook-data:/app/data
  nook:
    depends_on:
      nook-init:
        condition: service_completed_successfully
```

### R8 — JAMAIS `CMD-SHELL` healthcheck dans distroless
```yaml
# ❌ distroless n'a pas curl ni bash
healthcheck:
  test: ["CMD-SHELL", "curl -sf http://localhost:3000/health || exit 1"]
# ✅ supprimer le healthcheck (monitoring externe) ou TCP check
```

### R9 — CORS : `allow_credentials(true)` incompatible avec wildcards
```rust
// ❌ panic au démarrage : "Cannot combine credentials with Allow-Headers: *"
CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_credentials(true)
// ✅ lister explicitement
CorsLayer::new()
    .allow_origin(["http://localhost:5173".parse().unwrap(), ...])
    .allow_methods([Method::GET, Method::POST, ...])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT, COOKIE])
    .allow_credentials(true)
```

### R10 — SQLite : toujours `create_if_missing(true)`
```rust
// ❌ SQLITE_CANTOPEN (code 14) si le fichier n'existe pas encore
let pool = SqlitePool::connect(url).await?;
// ✅
let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
let pool = SqlitePool::connect_with(opts).await?;
```

### R11 — Axum 0.8 : routes `{param}` (plus `:param`)
```rust
// ❌ panic : "Path segments must not start with ':'"
.route("/conversations/:id", get(handler))
// ✅
.route("/conversations/{id}", get(handler))
// Les extracteurs Path(id): Path<String> ne changent PAS
```

### R12 — `download-artifact@v4` ne peut pas télécharger cross-workflow
```yaml
# ❌ cherche uniquement dans le workflow courant
- uses: actions/download-artifact@v4
  with:
    name: nook-backend-x86_64-unknown-linux-gnu
# ✅ utiliser dawidd6 pour cross-workflow
- uses: dawidd6/action-download-artifact@v6
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    workflow: Backend.yml
    branch: ${{ github.ref_name }}
    name: nook-backend-x86_64-unknown-linux-gnu
```

---

## 📦 Architecture CI/CD

### Flow manuel (développement / release)

```
1. Backend.yml   → nook-backend-amd64 + nook-backend-arm64  (retention 7j)
2. Frontend.yml  → nook-frontend                             (retention 7j)
         ↓
3. test-nook.yml → Dockerfile (cargo-chef) + docker-compose.ci.yml
                 → tests API + Playwright E2E
         ↓ si OK
4. Docker.yml    → dawidd6 télécharge 1+2 → Dockerfile.release → GHCR
```

### test-nook.yml — setup E2E

```yaml
# Utiliser le compose CI override qui ajoute E2E_SETUP=1
docker compose -f docker-compose.yml -f docker-compose.ci.yml up -d --build

# Attendre /api/health (pas /health qui retourne du HTML via fallback)
timeout 180s bash -c 'until curl -sf http://localhost:6300/api/health; do sleep 3; done'
```

**E2E_SETUP=1** → `check_initial_admin` crée le user `e2e_ci` (approved=1, sans changement mdp)  
→ Playwright se connecte directement avec `e2e_ci / E2eTest123!`

### Flow release

```
Release.yml (patch/minor/major)
  → bumpe VERSION + Cargo.toml + package.json
  → crée tag git vX.Y.Z
  → déclencher manuellement Backend.yml → Frontend.yml → test-nook.yml → Docker.yml
```

---

## 🏷️ Artifacts

| Workflow | Artifact | Contenu | Retention |
|----------|---------|---------|-----------|
| `Backend.yml` | `nook-backend-x86_64-unknown-linux-gnu` | `nook-backend-amd64` | 7j |
| `Backend.yml` | `nook-backend-aarch64-unknown-linux-gnu` | `nook-backend-arm64` | 7j |
| `Frontend.yml` | `nook-frontend` | `index.html` + assets | 7j |

---

## 🏷️ Tags Docker

```
ghcr.io/mx10-ac2n/nook:v0.5.0   ← version sémantique (badge README)
ghcr.io/mx10-ac2n/nook:latest   ← uniquement sur branch main
ghcr.io/mx10-ac2n/nook:sha-abc  ← SHA court traçabilité
```

---

## 🔬 Diagnostic rapide

| Erreur | Cause | Fix |
|--------|-------|-----|
| `cannot produce proc-macro for async-trait` | `--platform=$BUILDPLATFORM` ou `.cargo/config.toml` copié | R1 + R4 |
| `SQLITE_CANTOPEN (code 14)` | `SqlitePool::connect` sans `create_if_missing` | R10 |
| `Path segments must not start with ':'` | Routes axum 0.7 style | R11 |
| `Cannot combine credentials with *` | CORS wildcard + credentials | R9 |
| `Permission denied /app/data` | Volume Docker root + user nonroot 65532 | R7 |
| `Artifact not found` | download-artifact@v4 cross-workflow | R12 |
| `home@0.5.12 requires rustc 1.88` | Image Rust trop ancienne | R3 |
| `Login admin failed: 401` | Ne pas utiliser curl pour le setup E2E → utiliser E2E_SETUP=1 | voir test-nook.yml |

---

## 📊 Image distroless

| Propriété | Valeur |
|-----------|--------|
| Base | `gcr.io/distroless/cc-debian12:nonroot` |
| User | `nonroot` uid 65532 |
| Shell | Aucun |
| Port | 3000 interne → 6300 mappé |
| Libs | libsqlite3, libssl, libcrypto (copiées depuis debian:bookworm-slim) |
| Healthcheck | Aucun (distroless sans curl) — monitoring externe |
