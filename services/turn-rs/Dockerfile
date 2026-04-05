# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Zero Google dependency
# ============================================================

# - ETAPE 1 : Compilateur (nightly requis - edition 2024)
FROM rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    git cmake protobuf-compiler libprotobuf-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Build avec features par defaut (udp, tcp, ssl, api, prometheus)
RUN cargo build --release \
    && cp target/release/turn-server /usr/local/bin/turn-server

# - ETAPE 2 : Runtime Debian bookworm-slim (zero Google)
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3478/udp 3478/tcp

# Le fichier de config est monte en volume par docker-compose
CMD ["--config=/etc/turn-server/config.toml"]
