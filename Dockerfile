# =====================================================
# Dockerfile Nook - Build Statique (Ultra-Léger)
# =====================================================
# ✅ Binaire complètement statique (MUSL)
# ✅ Image finale = 20-30MB (vs 100MB+)
# ✅ Aucune dépendance runtime
# =====================================================

# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app

COPY frontend/package*.json ./
RUN npm ci --only=production

COPY frontend/ .
RUN npm run build && \
    test -f /app/build/index.html

# --- Build Backend Statique ---
FROM rust:1.83-alpine AS backend-builder
WORKDIR /app

# Installer les dépendances pour compilation statique
RUN apk add --no-cache \
    musl-dev \
    sqlite-static \
    openssl-libs-static \
    openssl-dev \
    pkgconfig

# Installer cargo-chef
RUN cargo install cargo-chef --locked

# Préparer la recette
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Compiler les dépendances
FROM backend-builder AS deps
COPY --from=backend-builder /app/recipe.json .
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build final statique
FROM backend-builder AS builder
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY --from=deps /app/target target
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Flags pour build statique
ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN cargo build --release --target x86_64-unknown-linux-musl --locked && \
    strip target/x86_64-unknown-linux-musl/release/nook-backend

# Vérifier que c'est bien statique
RUN file target/x86_64-unknown-linux-musl/release/nook-backend && \
    ldd target/x86_64-unknown-linux-musl/release/nook-backend 2>&1 | grep -q "not a dynamic executable"

# --- Image finale : Scratch ou Distroless Static ---
FROM gcr.io/distroless/static-debian12:nonroot

# Copier les certificats CA
COPY --from=backend-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Créer la structure (distroless static n'a rien)
COPY --from=backend-builder --chown=nonroot:nonroot \
    /app/target/x86_64-unknown-linux-musl/release/nook-backend /app/nook-backend

COPY --from=frontend-builder --chown=nonroot:nonroot /app/build/ /app/static/

WORKDIR /app
USER nonroot:nonroot

ENV RUST_LOG=info \
    DATABASE_URL=sqlite:/app/data/nook.db \
    PORT=3000

EXPOSE 3000

ENTRYPOINT ["/app/nook-backend"]