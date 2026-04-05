# ============================================================
# Nook - Backend (build from sources)
# ============================================================
# Zero Google dependency — Alpine runtime (~15MB final)
# ============================================================

# - ETAPE 1 : Compilateur (Debian bookworm -> cible musl statique)
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsodium-dev \
    pkg-config \
    musl-tools \
    && rm -rf /var/lib/apt/lists/* \
    && echo "CC_x86_64_unknown_linux_musl=musl-gcc" >> /etc/environment

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/nook

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    cargo build --release --target x86_64-unknown-linux-musl \
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

# - ETAPE 2 : Runtime Alpine 3.21 (zero Google, ~15MB total)
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    sqlite-libs \
    libsodium \
    libc6-compat \
    ca-certificates

# User non-root
RUN addgroup -S nook && adduser -S nook -G nook

# Dossiers applicatifs (permissifs car bind-mounts en CI)
RUN mkdir -p /app/data/uploads /app/logs /app/static \
    && chown -R nook:nook /app

# Binaire full static musl (pas besoin de libc)
COPY --from=builder /usr/local/bin/nook-backend /app/nook-backend
RUN chmod 0755 /app/nook-backend

WORKDIR /app

EXPOSE 3000

ENV DATA_DIR=/app/data
ENV STATIC_DIR=/app/static
ENV LOG_DIR=/app/logs

# Health check
HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD wget -qO- http://localhost:3000/api/health || exit 1

USER nook

ENTRYPOINT ["/app/nook-backend"]
