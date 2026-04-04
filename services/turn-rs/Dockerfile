# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Compatible distroless — meme base que Nook (gcr.io/distroless/cc)
# Configuration montée en volume par docker-compose
# ============================================================

# ── ÉTAPE 1 : Compileur (nightly requis — edition 2024) ────────
FROM rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    git cmake protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Build avec features par defaut (udp, tcp, ssl, api, prometheus)
RUN cargo build --release     && cp target/release/turn-server /usr/local/bin/turn-server

# ── ÉTAPE 2 : Entrypoint distroless ────────────────────────────
FROM gcr.io/distroless/cc-debian12:latest

COPY --from=builder /usr/local/bin/turn-server /usr/local/bin/turn-server

EXPOSE 3478/udp 3478/tcp

# Le fichier de config est monte en volume par docker-compose
CMD ["--config=/etc/turn-server/config.toml"]
