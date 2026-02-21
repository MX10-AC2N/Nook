# ===============================================
# Dockerfile Nook – Build complet + Distroless
# ===============================================

ARG BACKEND_PATH=backend
ARG FRONTEND_PATH=frontend

# ===============================================
# ÉTAPE 1 : Build Rust
# ===============================================
FROM --platform=$BUILDPLATFORM rust:1.80-bookworm AS builder

WORKDIR /usr/src/nook

# Cache dependencies
COPY ${BACKEND_PATH}/Cargo.toml ${BACKEND_PATH}/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {{}}" > src/main.rs && cargo build --release && rm -rf src

# Build réel
COPY ${BACKEND_PATH}/ ./
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

# Copie du frontend build (monté via volume ou build context)
COPY --from=prep --chown=nonroot:nonroot ${FRONTEND_PATH}/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]