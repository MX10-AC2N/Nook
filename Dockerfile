# ===============================================
# Dockerfile Nook – Build depuis sources
# Utilisé par : test-nook.yml + docker-compose
#
# ⚠️  PAS de --platform=$BUILDPLATFORM sur le builder :
#     ce flag cause une incompatibilité de proc-macros
#     (async-trait, serde_derive...) entre le cache
#     warmup et le vrai build quand on ne fait pas
#     de cross-compilation explicite.
#
# ⚠️  rust:1.88 minimum requis :
#     - home@0.5.12 exige rustc 1.88
#     - edition2024 des dépendances crypto exige rustc 1.85+
# ===============================================

# ===============================================
# ÉTAPE 1 : Build Rust depuis les sources
# ===============================================
FROM rust:1.88-bookworm AS builder

WORKDIR /usr/src/nook

# Cache layer : compile un dummy main pour warm up les deps
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src

# Build réel
COPY backend/ ./
RUN cargo build --release --bin nook-backend

# ===============================================
# ÉTAPE 2 : Extraction libs + Préparation
# ===============================================
FROM debian:bookworm-slim AS prep

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 libsodium23 libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

RUN mkdir -p /app/data /app/static && chown -R app:app /app

COPY --from=builder --chown=app:app /usr/src/nook/target/release/nook-backend /app/nook-backend

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

# Frontend build fourni par le job frontend (artifact CI)
COPY --chown=nonroot:nonroot frontend/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]
