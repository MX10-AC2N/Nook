# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Compatible distroless — meme base que Nook (gcr.io/distroless/cc)
# Configuration via variables d'environnement Docker Compose :
#   TURN_REALM, TURN_SECRET, TURN_PORT, TURN_MAX_BANDWIDTH
# ============================================================

# ── ÉTAPE 1 : Compileur (nightly requis — edition 2024) ────────
FROM rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends     git cmake protobuf-compiler pkg-config     && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Build avec toutes les features necessaires
# Les features default sont : udp, tcp, ssl, api, prometheus
RUN cargo build --release     && cp target/release/turn-server /usr/local/bin/turn-server

# ── ÉTAPE 2 : Configuration (prepare le fichier de config) ──────
FROM debian:bookworm-slim AS config-stamp

# Generer un config par defaut avec placeholders
RUN mkdir -p /etc/turn-server && cat > /etc/turn-server/turn-server.toml << 'CONF'
# ── Configuration Turn-Server ──
# Les variables ci-dessous sont remplacees au demarrage via
# le script entrypoint.sh qui fait un sed sur ce fichier.

[server]
interfaces = []
max_threads = 4
max_bandwidth = 1000000  # 1 Gbps en bits/s

[realm]
name = "__TURN_REALM__"
secret = "__TURN_SECRET__"

[stun]
port = 3478

[tcp]
enabled = true
port = 3478

[udp]
enabled = true
port = 3478
max_bandwidth = 1000000

[ssl]
enabled = false

[api]
enabled = false

[prometheus]
enabled = false
port = 9090
CONF

# ── ÉTAPE 3 : Entrypoint (remplace les placeholders + lance) ───
FROM gcr.io/distroless/cc-debian12:latest

# Copie du binaire compile
COPY --from=builder /usr/local/bin/turn-server /usr/local/bin/turn-server

# Copie du template de config
COPY --from=config-stamp /etc/turn-server/turn-server.toml /etc/turn-server/turn-server.toml

EXPOSE 3478/udp 3478/tcp

# Env par defaut (surcharges par docker-compose)
ENV TURN_REALM=nook.local
ENV TURN_SECRET=changeme2026
ENV TURN_PORT=3478
ENV TURN_MAX_BANDWIDTH=1000000

# Entrypoint : remplace les placeholders dans le config puis lance
ENTRYPOINT ["/usr/local/bin/turn-server"]
