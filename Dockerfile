# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app

# Plus besoin de PUBLIC_SITE_URL : injection dynamique via middleware Rust

COPY frontend/package*.json ./
RUN npm install

COPY frontend/ .
RUN npm run build

# --- Cargo Chef : Préparation ---
FROM rust:1.92-slim-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y libsqlite3-dev libsodium-dev libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# --- Analyse des dépendances ---
FROM chef AS planner
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo chef prepare --recipe-path recipe.json

# --- Cache des dépendances ---
FROM chef AS builder-deps
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

# --- Build backend ---
FROM chef AS backend-builder
WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

COPY --from=builder-deps /app/target ./target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

RUN cargo build --release --locked

RUN test -f target/release/nook-backend

# --- Étape intermédiaire : préparation du runtime ---
FROM debian:bookworm-slim AS runtime-prep

# Installation uniquement des libs runtime nécessaires + certificats CA
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        libsodium23 \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Création utilisateur non-root
RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

# Création répertoires avec bonnes permissions
RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app

# Copie binaire et fichiers statiques avec chown
COPY --from=backend-builder --chown=app:app /app/target/release/nook-backend /app/nook-backend
# ← Si votre build Vite sort dans "dist" (défaut Vite), changez en /app/dist/
COPY --from=frontend-builder --chown=app:app /app/build/ /app/static/

# Vérification (optionnelle)
RUN ls -la /app/static && \
    [ -f "/app/static/index.html" ] && echo "✅ index.html présent"

# --- Image finale : Distroless (multi-arch compatible) ---
FROM gcr.io/distroless/cc-debian12

# Copie utilisateur/groupe
COPY --from=runtime-prep /etc/passwd /etc/passwd
COPY --from=runtime-prep /etc/group /etc/group

# Copie certificats CA (utile pour requêtes HTTPS sortantes)
COPY --from=runtime-prep /etc/ssl/certs /etc/ssl/certs

# Copie des bibliothèques dynamiques nécessaires (compatible amd64 ET arm64)
# On utilise un glob récursif ** pour trouver les libs quel que soit le sous-dossier d'archi
# et on les copie "à plat" dans /usr/lib/ → le linker glibc les trouve directement là
COPY --from=runtime-prep /usr/lib/**/libsqlite3.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libsodium.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libssl.so* /usr/lib/
COPY --from=runtime-prep /usr/lib/**/libcrypto.so* /usr/lib/

# Copie application + répertoires (avec permissions conservées)
COPY --from=runtime-prep --chown=1000:1000 /app /app

WORKDIR /app

USER 1000:1000

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV PORT=3000

EXPOSE 3000

CMD ["/app/nook-backend"]