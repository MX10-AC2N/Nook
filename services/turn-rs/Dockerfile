# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Zero Google dependency — Alpine builder + Alpine runtime
# ============================================================

# - ETAPE 1 : Builder Alpine 3.21
FROM alpine:3.21 AS builder

RUN apk add --no-cache \
    rust \
    cargo \
    musl-dev \
    cmake \
    protobuf-dev \
    protobuf \
    git \
    pkgconfig

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Build avec features par defaut (udp, tcp, ssl, api, prometheus)
RUN cargo build --release \
    && cp target/release/turn-server /usr/local/bin/turn-server

# - ETAPE 2 : Runtime Alpine 3.21 (zero Google)
FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates

EXPOSE 3478/udp 3478/tcp

CMD ["/usr/local/bin/turn-server", "--config=/etc/turn-server/config.toml"]
