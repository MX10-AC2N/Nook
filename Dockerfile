# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app

COPY frontend/package*.json ./
RUN npm install

COPY frontend/ .
RUN npm run build

# Ajoute ces lignes pour debug (vérifie le build frontend dans logs Docker)
RUN echo "=== Contenu de /app/build/ ===" && ls -la /app/build/
RUN echo "=== Contenu de /app/build/_app/immutable/ ===" && ls -la /app/build/_app/immutable/ || echo "Dossier _app manquant"

# --- Cargo Chef : Préparation ---
FROM rust:1.92-slim-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y libsqlite3-dev libsodium-dev libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# --- Analyse des dépendances ---
FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo chef prepare --recipe-path recipe.json

# --- Cache des dépendances ---
FROM chef AS builder-deps
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

# --- Build backend ---
FROM chef AS backend-builder
WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations

COPY --from=builder-deps /app/target ./target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

RUN cargo build --release --locked

RUN test -f target/release/nook-backend

# --- Étape intermédiaire : préparation du runtime ---
FROM debian:bookworm-slim AS runtime-prep

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        libsodium23 \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app

COPY --from=backend-builder --chown=app:app /app/target/release/nook-backend /app/nook-backend
COPY --from=frontend-builder --chown=app:app /app/build/ /app/static/

# Debug runtime : vérifie les fichiers static copiés
RUN echo "=== Contenu de /app/static/ dans runtime ===" && ls -la /app/static/
RUN echo "=== Contenu de /app/static/_app/immutable/ ===" && ls -la /app/static/_app/immutable/ || echo "Dossier _app manquant dans static"

RUN ls -la /app/static && \
    [ -f "/app/static/index.html" ] && echo "✅ index.html présent"

# --- Image finale : Distroless ---
FROM gcr.io/distroless/cc-debian12

COPY --from=runtime-prep /etc/passwd /etc/passwd
COPY --from=runtime-prep /etc/group /etc/group

COPY --from=runtime-prep /etc/ssl/certs /etc/ssl/certs

COPY --from=runtime-prep /usr/lib/**/libsqlite3.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libsodium.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libssl.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libcrypto.so* /usr/lib/

COPY --from=runtime-prep --chown=1000:1000 /app /app

WORKDIR /app

USER 1000:1000

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV PORT=3000

EXPOSE 3000

CMD ["/app/nook-backend"]