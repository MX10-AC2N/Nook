# =====================================================
# Dockerfile Nook - Build Statique Ultra-Léger
# =====================================================
# ✅ Binaire complètement statique (MUSL)
# ✅ Image finale ~20-30MB
# ✅ Aucune dépendance runtime
# ✅ Sécurité maximale (distroless static)
# =====================================================

# --- Build Frontend (SPA Svelte 5) ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app

# Installation des dépendances
COPY frontend/package*.json ./
RUN npm ci --only=production && \
    npm cache clean --force

# Copie du code source frontend
COPY frontend/ .

# Build du frontend avec placeholder pour injection dynamique
RUN npm run build && \
    test -f /app/build/index.html && \
    echo "✅ Frontend build réussi"

# --- Cargo Chef (pour cache optimal des dépendances) ---
FROM rust:1.83-alpine AS chef
WORKDIR /app

# Installation des outils de build statique
RUN apk add --no-cache \
    musl-dev \
    sqlite-static \
    openssl-libs-static \
    openssl-dev \
    pkgconfig \
    git

# Installation de cargo-chef pour optimiser le cache
RUN cargo install cargo-chef --locked

# --- Préparation de la recette (analyse des dépendances) ---
FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo chef prepare --recipe-path recipe.json

# --- Compilation des dépendances (mise en cache) ---
FROM chef AS builder-deps
COPY --from=planner /app/recipe.json recipe.json

# Configuration pour build statique MUSL
ENV RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static"

# Compilation des dépendances uniquement
RUN cargo chef cook \
    --release \
    --target x86_64-unknown-linux-musl \
    --recipe-path recipe.json

# --- Build final du backend ---
FROM chef AS backend-builder
WORKDIR /app

# Copier les sources
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

# Copier le cache des dépendances compilées
COPY --from=builder-deps /app/target target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

# Configuration pour build statique
ENV RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static"

# Build release avec optimisations maximales
RUN cargo build \
    --release \
    --target x86_64-unknown-linux-musl \
    --locked && \
    echo "✅ Backend compilé"

# Strip du binaire pour réduire la taille
RUN strip target/x86_64-unknown-linux-musl/release/nook-backend && \
    echo "✅ Binaire optimisé"

# Vérification que le binaire est bien statique
RUN file target/x86_64-unknown-linux-musl/release/nook-backend && \
    (ldd target/x86_64-unknown-linux-musl/release/nook-backend 2>&1 | grep -q "not a dynamic executable" || \
     ldd target/x86_64-unknown-linux-musl/release/nook-backend 2>&1 | grep -q "statically linked") && \
    echo "✅ Binaire 100% statique confirmé"

# --- Image finale : Distroless Static (ultra-sécurisée) ---
FROM gcr.io/distroless/static-debian12:nonroot

# Métadonnées
LABEL maintainer="MX10-AC2N" \
      description="Nook Backend - Build Statique Ultra-Léger" \
      version="1.0"

# Copier les certificats CA (pour HTTPS sortant si nécessaire)
COPY --from=backend-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copier le binaire statique
COPY --from=backend-builder --chown=nonroot:nonroot \
    /app/target/x86_64-unknown-linux-musl/release/nook-backend \
    /app/nook-backend

# Copier le frontend (fichiers statiques)
COPY --from=frontend-builder --chown=nonroot:nonroot \
    /app/build/ \
    /app/static/

# Créer les répertoires de données (volumes)
# Note: distroless ne permet pas mkdir, donc on les crée via VOLUME
VOLUME ["/app/data", "/app/data/uploads"]

# Définir le répertoire de travail
WORKDIR /app

# Utilisateur non-root (intégré à distroless)
USER nonroot:nonroot

# Variables d'environnement
ENV RUST_LOG=info \
    DATABASE_URL=sqlite:/app/data/nook.db \
    PORT=3000

# Exposition du port
EXPOSE 3000

# Point de santé Docker
HEALTHCHECK --interval=30s \
            --timeout=5s \
            --start-period=10s \
            --retries=3 \
    CMD ["/app/nook-backend", "--health"] || exit 1

# Point d'entrée
ENTRYPOINT ["/app/nook-backend"]