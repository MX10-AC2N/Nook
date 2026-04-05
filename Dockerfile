# ============================================================
# Nook — Backend (build from sources)
# ============================================================
# Compatible Alpine — zero Google dependency
# ============================================================

# ── ÉTAPE 1 : Compilateur (Debian bookworm → cible musl) ──────
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    musl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Ajouter les cibles musl (cross-compilation depuis glibc builder vers musl)
RUN rustup target add x86_64-unknown-linux-musl \
    && rustup target add aarch64-unknown-linux-musl

WORKDIR /usr/src/nook

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
# Dummy build pour le cache deps — cible musl
RUN cargo build --target x86_64-unknown-linux-musl --release \
    && rm -f target/x86_64-unknown-linux-musl/release/nook-backend

# Build réel
COPY backend/.sqlx ./.sqlx
COPY backend/migrations ./migrations/
COPY backend/src ./src/

ENV SQLX_OFFLINE=true
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=z
ENV CARGO_PROFILE_RELEASE_STRIP=true
RUN cargo build --release --locked --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/nook-backend /usr/local/bin/nook-backend

# ── ÉTAPE 2 : Runtime Alpine 3.21 ─────────────────────────────
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    ca-certificates

# SQLite et libsodium (runtime only)
RUN apk add --no-cache \
    sqlite-libs \
    libsodium

# Dossiers applicatifs
RUN mkdir -p /app/data /app/static /app/logs

# Binaire compilé en musl (compatible Alpine nativement)
COPY --from=builder /usr/local/bin/nook-backend /app/nook-backend

# Frontend build (copié depuis l'artefact Frontend.yml)
# COPY frontend/build /app/static  ← fait par le workflow CI

WORKDIR /app

EXPOSE 3000

ENV DATA_DIR=/app/data
ENV STATIC_DIR=/app/static
ENV LOG_DIR=/app/logs

# Health check
HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD wget -qO- http://localhost:3000/api/health || exit 1

# Nook tourne sans privileges (Alpine nobody = 65534)
RUN addgroup -S nook && adduser -S nook -G nook
USER nook:nook

ENTRYPOINT ["/app/nook-backend"]
