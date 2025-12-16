# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install --legacy-peer-deps
COPY frontend/ .
RUN npm run build

# --- Build Backend ---
FROM rust:1.83-slim-bookworm AS backend-builder

# Installer les dépendances système
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    libsqlite3-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copier les dépendances Cargo
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Générer la base de données temporaire pour SQLx
RUN mkdir -p data && \
    sqlite3 data/temp.db "VACUUM;" && \
    chmod 666 data/temp.db

# Variables pour SQLx
ENV DATABASE_URL=sqlite:data/temp.db

# Installer sqlx-cli
RUN cargo install sqlx-cli --version 0.8.2 --no-default-features --features sqlite --locked

# Copier le code source
COPY backend/src ./src

# Build final
RUN cargo build --release

# Vérifier que le binaire existe
RUN test -f target/release/nook-backend

# --- Runtime intermédiaire (pour créer l'utilisateur) ---
FROM debian:bookworm-slim AS runtime-builder

# Créer l'utilisateur non-root
RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

# Créer les dossiers
RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app

# Copier le binaire et les fichiers
COPY --from=backend-builder --chown=app:app /app/target/release/nook-backend /app/nook-backend
COPY --from=frontend-builder --chown=app:app /app/build/ /app/static/

# --- Final : distroless ---
FROM gcr.io/distroless/cc-debian12

# Copier depuis l'étape intermédiaire
COPY --from=runtime-builder /etc/passwd /etc/passwd
COPY --from=runtime-builder /app /app

# Changer d'utilisateur
USER app

# Variables d'environnement
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV PORT=3000

# Exposer le port
EXPOSE 3000

# Point d'entrée
CMD ["/app/nook-backend"]