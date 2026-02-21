# ===============================================
# Dockerfile Nook – Build depuis sources
# Utilisé par : test-nook.yml + docker-compose
# ===============================================

# ===============================================
# ÉTAPE 1 : Build Rust
#
# ⚠️  linux/amd64 explicite (pas $BUILDPLATFORM) :
#     BuildKit avec BUILDKIT_INLINE_CACHE active le
#     mode multi-plateforme. Sans plateforme fixe,
#     les artifacts du cache warmup sont compilés
#     pour le host mais les proc-macros (async-trait,
#     serde_derive...) sont ensuite demandés pour
#     une target différente → crash.
#
# ⚠️  rust:1.88 minimum :
#     home@0.5.12 exige rustc 1.88
# ===============================================
FROM --platform=linux/amd64 rust:1.88-bookworm AS builder

WORKDIR /usr/src/nook

# Cache layer : dummy main pour warm-up des dépendances
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

# Frontend build fourni par le job CI (artifact)
COPY --chown=nonroot:nonroot frontend/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]
