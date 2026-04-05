# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Zero Google dependency — Alpine builder + Alpine runtime
# ============================================================

FROM alpine:3.21 AS builder

RUN apk add --no-cache \
    curl \
    gcc \
    g++ \
    musl-dev \
    build-base \
    cmake \
    protobuf-dev \
    protobuf \
    git \
    pkgconfig

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain nightly --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

RUN cargo build --release \
    && cp target/release/turn-server /usr/local/bin/turn-server

FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates

EXPOSE 3478/udp 3478/tcp

CMD ["/usr/local/bin/turn-server", "--config=/etc/turn-server/config.toml"]
