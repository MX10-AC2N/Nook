# ===============================================
# BUILD FRONTEND (Svelte 5 + SvelteKit)
# ===============================================
FROM node:20-alpine AS frontend-builder
WORKDIR /app

# Copie uniquement les fichiers de dépendances d'abord (cache Docker)
COPY frontend/package.json frontend/package-lock.json ./

# Installation PROPRE avec npm ci (lockfile strict)
RUN npm ci --prefer-offline --no-audit --loglevel=error

# Copie du code source complet
COPY frontend/ ./

# Build avec gestion d'erreur explicite
RUN set -e && \
    echo "🔨 Démarrage du build frontend..." && \
    npm run build && \
    echo "✅ Build terminé avec succès"

# Vérification CRITIQUE que le build a produit les fichiers
RUN set -e && \
    echo "📁 Vérification de l'existence du répertoire build..." && \
    if [ ! -d "build" ]; then \
      echo "❌ ERREUR FATALE: Le répertoire 'build' n'existe pas !"; \
      echo "Le build SvelteKit a échoué silencieusement."; \
      exit 1; \
    fi && \
    echo "✅ Répertoire build trouvé" && \
    if [ ! -f "build/index.html" ]; then \
      echo "❌ ERREUR FATALE: build/index.html absent !"; \
      exit 1; \
    fi && \
    echo "✅ index.html présent"

# Debug optionnel : afficher la structure (commentez après debug)
RUN echo "📦 Structure du build (premiers fichiers):" && \
    find build -type f | head -20 && \
    echo "📊 Taille totale du build:" && \
    du -sh build/

# ===============================================
# CARGO CHEF : Préparation des dépendances Rust
# ===============================================
FROM rust:1.92-slim-bookworm AS chef
WORKDIR /app

# Installation de cargo-chef pour le cache des dépendances
RUN cargo install cargo-chef --locked

# Installation des bibliothèques système nécessaires
RUN apt-get update && \
    apt-get install -y \
        libsqlite3-dev \
        libsodium-dev \
        libssl-dev \
        pkg-config && \
    rm -rf /var/lib/apt/lists/*

# ===============================================
# ANALYSE DES DÉPENDANCES RUST
# ===============================================
FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo chef prepare --recipe-path recipe.json

# ===============================================
# CACHE DES DÉPENDANCES RUST
# ===============================================
FROM chef AS builder-deps
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

# ===============================================
# BUILD BACKEND (Rust)
# ===============================================
FROM chef AS backend-builder
WORKDIR /app

# Copie des fichiers de configuration Cargo
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations

# Copie du cache des dépendances pré-compilées
COPY --from=builder-deps /app/target ./target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

# Compilation du backend en mode release
RUN cargo build --release --locked

# Vérification que le binaire a bien été créé
RUN test -f target/release/nook-backend && \
    echo "✅ Backend compilé avec succès ($(ls -lh target/release/nook-backend | awk '{print $5}'))" || \
    (echo "❌ ERREUR: Le binaire nook-backend est absent !" && exit 1)

# ===============================================
# PRÉPARATION DU RUNTIME
# ===============================================
FROM debian:bookworm-slim AS runtime-prep

# Installation des bibliothèques runtime minimales
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        libsodium23 \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Création de l'utilisateur non-root pour la sécurité
RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

# Création de la structure de répertoires
RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app

# Copie du backend compilé avec les bons droits
COPY --from=backend-builder --chown=app:app \
    /app/target/release/nook-backend /app/nook-backend

# Rendre le binaire exécutable
RUN chmod +x /app/nook-backend

# Copie du frontend buildé avec les bons droits
COPY --from=frontend-builder --chown=app:app \
    /app/build/ /app/static/

# Vérification finale critique
RUN set -e && \
    echo "🔍 Vérification finale des fichiers..." && \
    ls -lah /app/static && \
    if [ ! -f "/app/static/index.html" ]; then \
      echo "❌ ERREUR FATALE: Frontend absent après copie !"; \
      exit 1; \
    fi && \
    echo "✅ Frontend copié avec succès"

# ===============================================
# IMAGE FINALE : Distroless (sécurité maximale)
# ===============================================
FROM gcr.io/distroless/cc-debian12

# Métadonnées de l'image
LABEL maintainer="MX10-AC2N"
LABEL description="Nook - Messagerie familiale chiffrée E2EE"
LABEL version="0.5.0"

# Copie des fichiers système nécessaires
COPY --from=runtime-prep /etc/passwd /etc/passwd
COPY --from=runtime-prep /etc/group /etc/group
COPY --from=runtime-prep /etc/ssl/certs /etc/ssl/certs

# Copie des bibliothèques partagées nécessaires au runtime
COPY --from=runtime-prep /usr/lib/**/libsqlite3.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libsodium.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libssl.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libcrypto.so* /usr/lib/

# Copie de l'application complète (backend + frontend + data)
COPY --from=runtime-prep --chown=1000:1000 /app /app

WORKDIR /app

# Utilisation de l'utilisateur non-root (sécurité)
USER 1000:1000

# Variables d'environnement
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV PORT=3000

# Exposition du port de l'application
EXPOSE 3000

# Commande de démarrage
CMD ["/app/nook-backend"]