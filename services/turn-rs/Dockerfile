# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Zero Google dependency — Alpine runtime
# ============================================================

# ── ETAPE 1 : Compileur (nightly requis -- edition 2024) ────────
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

# ── ETAPE 2 : Runtime Alpine 3.21 (zero Google) ─────────────
FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates

EXPOSE 3478/udp 3478/tcp

CMD ["/usr/local/bin/turn-server", "--config=/etc/turn-server/config.toml"]
