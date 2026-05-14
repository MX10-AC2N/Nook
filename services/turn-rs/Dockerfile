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
RUN apk add --no-cache \\\
    curl \\\
    gcc \\\
    g++ \\\
    musl-dev \\\
    build-base \\\
    cmake \\\
    protobuf-dev \\\
    protobuf \\\
    git \\\
    pkgconfig \\\
  && command -v protoc \\\
  && protoc --version \\\
  && ls -l /usr/bin/protoc

# Also install static protoc binary from official release as fallback
RUN curl -L -o /usr/local/bin/protoc https://github.com/protocolbuffers/protobuf/releases/download/v28.2/protoc-28.2-linux-aarch_64.tar.gz | tar xz -C /usr/local/bin bin/protoc && chmod +x /usr/local/bin/protoc && protoc --version

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain nightly --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /src
RUN git clone --depth 1 https://github.com/mycrl/turn-rs.git /src/turn-rs
WORKDIR /src/turn-rs

# Set PROTOC path for turn-server-sdk build
ENV PROTOC=/usr/bin/protoc
ENV PATH="/usr/bin:/usr/local/bin:/root/.cargo/bin:$PATH"
RUN ln -sf /usr/bin/protoc /usr/local/bin/protoc

# Verify protoc is usable by cargo build environment
RUN command -v protoc && protoc --version && ls -l /usr/bin/protoc
RUN echo "PATH=$PATH" && echo "PROTOC=$PROTOC" && env | sort | grep -i proto

RUN PROTOC=/usr/bin/protoc cargo build --release && cp target/release/turn-server /usr/local/bin/turn-server

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
