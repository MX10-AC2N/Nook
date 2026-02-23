# ===============================================
# Dockerfile Nook – Build depuis sources
# Utilisé par : test-nook.yml + docker-compose
# ===============================================

# syntax=docker/dockerfile:1

# ===============================================
# ÉTAPE 1 : Build Rust
# ⚠️ Copie EXPLICITE — on exclut .cargo/config.toml
# qui force le linker x86_64-linux-gnu-gcc et met
# Cargo en mode cross-compilation → crash proc-macros
# ===============================================
FROM rust:1.88-bookworm AS builder

WORKDIR /usr/src/nook

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/.sqlx ./.sqlx

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/nook/target \
    SQLX_OFFLINE=true cargo build --release --bin nook-backend && \
    cp target/release/nook-backend /usr/local/bin/nook-backend

# ===============================================
# ÉTAPE 2 : Préparation des libs et permissions
# ===============================================
FROM debian:bookworm-slim AS prep

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 libsodium23 libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/data /app/static /app/logs

COPY --from=builder /usr/local/bin/nook-backend /app/nook-backend

# ===============================================
# ÉTAPE 3 : Image finale Distroless
# ===============================================
FROM gcr.io/distroless/cc-debian12:latest

COPY --from=prep /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=prep /usr/lib/*/libsqlite3.so* /usr/lib/
COPY --from=prep /usr/lib/*/libsodium.so* /usr/lib/
COPY --from=prep /usr/lib/*/libssl.so* /usr/lib/
COPY --from=prep /usr/lib/*/libcrypto.so* /usr/lib/

COPY --from=prep /app /app

# Frontend build (fourni par le job CI via artifact)
COPY frontend/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]
