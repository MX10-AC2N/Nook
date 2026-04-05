# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Compatible Alpine — zero Google dependency
# ============================================================

# ── ÉTAPE 1 : Compilateur (musl cible) ────────────────────────
FROM rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    cmake \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# Ajouter la cible musl
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Build avec features par defaut (udp, tcp, ssl, api, prometheus)
# Cible musl pour compatibilite Alpine
RUN cargo build --release --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/turn-server /usr/local/bin/turn-server

# ── ÉTAPE 2 : Runtime Alpine 3.21 ─────────────────────────────
FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    ca-certificates

EXPOSE 3478/udp 3478/tcp

# Le fichier de config est monte en volume par docker-compose
CMD ["/usr/local/bin/turn-server", "--config=/etc/turn-server/config.toml"]
