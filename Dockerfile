# ===============================================
# Dockerfile Nook – Build depuis sources
# Utilisé par : test-nook.yml + docker-compose
#
# Stratégie : BuildKit cache mounts pour ~/.cargo
# et target/ — plus fiable que cargo-chef ou
# dummy main, zéro problème de proc-macros.
# ===============================================

# syntax=docker/dockerfile:1
FROM rust:1.88-bookworm AS builder

WORKDIR /usr/src/nook

# Copie des fichiers de dépendances
COPY backend/Cargo.toml backend/Cargo.lock ./

# Copie des sources
COPY backend/ ./

# Build avec cache mount BuildKit
# --mount=type=cache garde ~/.cargo/registry et target/
# entre les builds sans jamais mélanger les contextes de compilation
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/nook/target \
    cargo build --release --bin nook-backend && \
    # Copier le binaire hors du cache avant que le layer soit finalisé
    cp target/release/nook-backend /usr/local/bin/nook-backend

# ===============================================
# ÉTAPE 2 : Préparation
# ===============================================
FROM debian:bookworm-slim AS prep

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 libsodium23 libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

RUN mkdir -p /app/data /app/static && chown -R app:app /app

COPY --from=builder --chown=app:app /usr/local/bin/nook-backend /app/nook-backend

# ===============================================
# ÉTAPE 3 : Image finale Distroless
# ===============================================
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=prep /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=prep /usr/lib/*/libsqlite3.so* /usr/lib/
COPY --from=prep /usr/lib/*/libsodium.so* /usr/lib/
COPY --from=prep /usr/lib/*/libssl.so* /usr/lib/
COPY --from=prep /usr/lib/*/libcrypto.so* /usr/lib/

COPY --from=prep --chown=nonroot:nonroot /app /app

COPY --chown=nonroot:nonroot frontend/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]
