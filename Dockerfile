# ===============================================
# Dockerfile Nook – Build depuis sources
# Utilisé par : test-nook.yml + docker-compose
#
# Utilise cargo-chef pour le cache des dépendances.
# La technique "dummy fn main()" cause une
# incompatibilité de proc-macros (async-trait) car
# Cargo compile les deps dans un contexte différent
# de celui du vrai build --bin.
# cargo-chef résout ce problème proprement.
# ===============================================

# ===============================================
# ÉTAPE 0 : cargo-chef – préparation du recipe
# ===============================================
FROM rust:1.88-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /usr/src/nook

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

# ===============================================
# ÉTAPE 1 : Build des dépendances (layer cacheable)
# ===============================================
FROM chef AS builder
COPY --from=planner /usr/src/nook/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build réel (seul le code source change ici)
COPY backend/ .
RUN cargo build --release --bin nook-backend

# ===============================================
# ÉTAPE 2 : Préparation de l'image finale
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

COPY --chown=nonroot:nonroot frontend/build /app/static

WORKDIR /app
EXPOSE 3000

ENV RUST_LOG=info \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/nook.db \
    STATIC_FILES_DIR=/app/static \
    UPLOADS_DIR=/app/data/uploads

ENTRYPOINT ["/app/nook-backend"]
