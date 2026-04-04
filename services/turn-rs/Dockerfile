# ── Turn-Server (TURN/STUN pour WebRTC) ───────────────────────────────
# Build a partir des sources mycrl/turn-rs
# Usage : docker compose up -d  (le service 'turn' est construit ici)

FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs
RUN cargo build --release && cp target/release/turn-server /usr/local/bin/turn-server

# ── Runtime minimal ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/turn-server /usr/local/bin/turn-server

EXPOSE 3478/udp 3478/tcp

ENTRYPOINT ["/usr/local/bin/turn-server"]
