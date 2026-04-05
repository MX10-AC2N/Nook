# ============================================================
# Nook - Backend (build from sources)
# ============================================================
# Zero Google dependency — Alpine builder + Alpine runtime (~15MB)
# ============================================================

# - ETAPE 1 : Builder Alpine 3.21 (musl natif)
FROM alpine:3.21 AS builder

RUN apk add --no-cache \
    rust \
    cargo \
    musl-dev \
    sqlite-dev \
    libsodium-dev \
    pkgconfig

WORKDIR /usr/src/nook

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN CARGO_BUILD_JOBS=$(nproc) cargo build --release \
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
RUN cargo build --release --locked \
    && cp target/release/nook-backend /usr/local/bin/nook-backend

# - ETAPE 2 : Runtime Alpine 3.21 (zero Google)
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    sqlite-libs \
    libsodium \
    ca-certificates

# User non-root (Alpine useradd/addgroup syntax)
RUN addgroup -S nook && adduser -S nook -G nook

# Dossiers applicatifs
RUN mkdir -p /app/data/uploads /app/logs /app/static \
    && chown -R nook:nook /app

# Binaire statique musl (pas besoin de libc)
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
