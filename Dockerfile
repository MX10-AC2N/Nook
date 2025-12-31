# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app

COPY frontend/package*.json ./
RUN npm ci   # Plus propre et reproductible que npm install

COPY frontend/ .
RUN npm run build

# --- Cargo Chef : Préparation de la recette ---
FROM rust:1.92-slim-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y libsqlite3-dev libsodium-dev pkg-config && rm -rf /var/lib/apt/lists/*

# --- Analyse des dépendances (recette) ---
FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo chef prepare --recipe-path recipe.json

# --- Cache des dépendances compilées ---
FROM chef AS builder-deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# --- Build final du backend ---
FROM chef AS backend-builder
WORKDIR /app

# Copier les sources
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

# Copier le cache des dépendances
COPY --from=builder-deps /app/target target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

# Build release (la plupart des crates sont déjà compilées !)
RUN cargo build --release --locked

# Vérifier le binaire
RUN test -f target/release/nook-backend

# --- Runtime intermédiaire ---
FROM debian:bookworm-slim AS runtime-builder

RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app

COPY --from=backend-builder --chown=app:app /app/target/release/nook-backend /app/nook-backend
COPY --from=frontend-builder --chown=app:app /app/build/ /app/static/

# Vérification finale
RUN ls -la /app/static && \
    [ -f "/app/static/index.html" ] && echo "✅ index.html présent"

# --- Image finale : Distroless ---
FROM gcr.io/distroless/cc-debian12:latest

COPY --from=runtime-builder /etc/passwd /etc/passwd
COPY --from=runtime-builder /app /app

USER app

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/members.db
ENV PORT=3000

EXPOSE 3000

CMD ["/app/nook-backend"]