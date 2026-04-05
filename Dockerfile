# ============================================================
# Nook - Backend (build from sources)
# ============================================================
# Compatible Alpine - zero Google dependency
# ============================================================

# ── ETAPE 1 : Compilateur (Debian bookworm -> cible musl) ──────
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    musl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Ajouter les cibles musl
RUN rustup target add x86_64-unknown-linux-musl \
    && rustup target add aarch64-unknown-linux-musl

WORKDIR /usr/src/nook

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    cargo build --target x86_64-unknown-linux-musl --release \
    && rm -f target/x86_64-unknown-linux-musl/release/nook-backend

# Build reel
COPY backend/.sqlx ./.sqlx
COPY backend/migrations ./migrations/
COPY backend/src ./src/

ENV SQLX_OFFLINE=true
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=z
ENV CARGO_PROFILE_RELEASE_STRIP=true
RUN CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    cargo build --release --locked --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/nook-backend /usr/local/bin/nook-backend

# ── ETAPE 2 : Runtime Alpine 3.21 ─────────────────────────────
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    ca-certificates \
    sqlite-libs \
    libsodium

# Dossiers applicatifs (seront recrees au demarrage si bind-mount)
RUN mkdir -p /app/data/uploads /app/static /app/logs \
    && chmod 0777 /app/data /app/data/uploads /app/logs /app/static \
    && ln -sf /app/data/nook.db /app/nook.db 2>/dev/null || true

# Binaire compile en musl
COPY --from=builder /usr/local/bin/nook-backend /app/nook-backend
RUN chmod 0755 /app/nook-backend

WORKDIR /app

EXPOSE 3000

ENTRYPOINT ["/app/nook-backend"]
