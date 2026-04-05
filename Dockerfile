# ============================================================
# Nook - Backend (build from sources)
# ============================================================
# Zero Google dependency
# ============================================================

# - ETAPE 1 : Compilateur (Debian bookworm)
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsodium-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/nook

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN cargo build --release \
    && rm -f target/release/nook-backend

# Build reel
COPY backend/.sqlx ./.sqlx
COPY backend/migrations ./migrations/
COPY backend/src ./src/

ENV SQLX_OFFLINE=true
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=z
ENV CARGO_PROFILE_RELEASE_STRIP=true
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --locked \
    && cp target/release/nook-backend /usr/local/bin/nook-backend

# - ETAPE 2 : Runtime Debian bookworm-slim (zero Google)
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    libsodium23 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# User non-root
RUN addgroup --system nook && adduser --system --ingroup nook nook

# Dossiers applicatifs
RUN mkdir -p /app/data/uploads /app/logs /app/static \
    && chown -R nook:nook /app

# Binaire compile (avec +crt-static donc minimal deps)
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
