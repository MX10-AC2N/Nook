# ============================================================
# Nook - Backend (build from sources)
# ============================================================
# Zero Google dependency — Alpine builder + Alpine runtime (~15MB)
# ============================================================

# - ETAPE 1 : Builder Alpine 3.21 (musl natif)
FROM alpine:3.21 AS builder

RUN apk add --no-cache \
    curl \
    gcc \
    g++ \
    musl-dev \
    build-base \
    sqlite-dev \
    libsodium-dev \
    pkgconfig

# Nightly rust via rustup (edition2024 deps comme home-0.5.12 exigent Cargo >= 1.85)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain nightly --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

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
RUN cargo build --release --locked \
    && cp target/release/nook-backend /usr/local/bin/nook-backend

# - ETAPE 2 : Runtime Alpine 3.21 (zero Google)
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    sqlite-libs \
    libsodium \
    ca-certificates

RUN addgroup -S nook && adduser -S nook -G nook

RUN mkdir -p /app/data/uploads /app/logs \
    && chown -R nook:nook /app

WORKDIR /app

# Binaire compile (full static musl)
COPY --from=builder /usr/local/bin/nook-backend /app/nook-backend
RUN chmod 0755 /app/nook-backend

# Frontend build (injecte par download-artifact en CI)
COPY frontend/build /app/static
RUN chown -R nook:nook /app/static \
    && find /app/static -type d -exec chmod 0755 {} \; \
    && find /app/static -type f -exec chmod 0644 {} \;

EXPOSE 3000

ENV DATA_DIR=/app/data
ENV STATIC_DIR=/app/static
ENV LOG_DIR=/app/logs

# Health check
HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD wget -qO- http://localhost:3000/api/health || exit 1

USER nook

ENTRYPOINT ["/app/nook-backend"]
