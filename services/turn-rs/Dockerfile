# ============================================================
# Turn-Server (TURN/STUN pour WebRTC)
# Build a partir des sources mycrl/turn-rs
# ============================================================
# Zero Google dependency - Alpine builder + Alpine runtime
# ============================================================

# - ETAPE 1 : Builder Alpine 3.21
FROM alpine:3.21 AS builder

# Install protobuf compiler and development files
# Alpine uses 'protobuf-dev' for protoc and headers; ensure it's present
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

# Verify protoc is available and working
RUN set -eux; \\
    test -x /usr/bin/protoc; \\
    /usr/bin/protoc --version; \\
    echo "PROTOC=/usr/bin/protoc"; \\
    echo "PATH=$PATH"

# Install Rust (nightly)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain nightly --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Set PROTOC path for turn-server-sdk build
ENV PROTOC=/usr/bin/protoc
ENV PATH="/usr/local/bin:/usr/bin:/root/.cargo/bin:$PATH"
RUN ln -sf /usr/bin/protoc /usr/local/bin/protoc

# Build turn-server natively for the builder's architecture (musl)
RUN PROTOC=/usr/bin/protoc cargo build --release && \
    cp target/release/turn-server /usr/local/bin/turn-server

# - ETAPE 2 : Runtime Alpine 3.21
FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates

RUN addgroup -g 1000 nook && adduser -D -u 1000 -G nook nook

# Copy the compiled binary from builder stage
COPY --from=builder /usr/local/bin/turn-server /usr/local/bin/turn-server

EXPOSE 3478/udp 3478/tcp

HEALTHCHECK --interval=15s --timeout=5s --retries=3 \
    CMD pgrep turn-server || exit 1

USER nook:nook

CMD ["/usr/local/bin/turn-server", "--config=/etc/turn-server/config.toml"]
