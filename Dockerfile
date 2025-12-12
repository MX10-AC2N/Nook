# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
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

# Copier les fichiers de configuration Cargo (pour cache)
COPY backend/Cargo.toml backend/ ./

# Génère Cargo.lock
RUN cargo generate-lockfile

# Créer un build dummy pour mettre en cache les dépendances
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/nook*

# Copier le code source réel
COPY backend/src ./src
COPY backend/migrations ./migrations

# Créer la base de données temporaire pour SQLx
RUN mkdir -p data && \
    sqlite3 data/temp.db "VACUUM;" && \
    chmod 666 data/temp.db

# Variable d'environnement pour SQLx
ENV DATABASE_URL=sqlite:data/temp.db

# Installer sqlx-cli (version stable et compatible)
RUN cargo install sqlx-cli \
    --version 0.8.2 \
    --no-default-features \
    --features sqlite \
    --locked

# Exécuter les migrations si elles existent
RUN if [ -d migrations ] && [ "$(ls -A migrations 2>/dev/null)" ]; then \
        echo "📦 Running migrations..." && \
        sqlx migrate run --database-url "$DATABASE_URL"; \
    else \
        echo "⚠️  No migrations found"; \
    fi

# Préparer le cache SQLx (optionnel mais recommandé)
RUN cargo sqlx prepare --database-url "$DATABASE_URL" || \
    echo "⚠️  SQLx prepare skipped (not critical)"

# Build final en mode release
RUN cargo build --release

# Vérifier que le binaire existe
RUN ls -lh target/release/ && \
    test -f target/release/nook-backend || \
    (echo "❌ Binary not found!" && exit 1)

# --- Runtime ---
FROM debian:bookworm-slim

# Installer les dépendances runtime minimales
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    sqlite3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Créer utilisateur non-root
RUN useradd -m -u 1000 -s /bin/bash app

# Créer la structure de dossiers
RUN mkdir -p /app/data /app/static /app/migrations && \
    chown -R app:app /app

WORKDIR /app

# Copier le binaire backend
COPY --from=backend-builder --chown=app:app \
    /app/target/release/nook-backend ./nook-backend

# Vérifier que le binaire est exécutable
RUN chmod +x /app/nook-backend && \
    ls -lh /app/nook-backend

# Copier les fichiers frontend buildés
COPY --from=frontend-builder --chown=app:app \
    /app/build ./static

# Copier les migrations
COPY --from=backend-builder --chown=app:app \
    /app/migrations ./migrations

# Script d'initialisation
COPY --chmod=755 <<'EOF' /app/init.sh
#!/bin/bash
set -e

echo "======================================"
echo "🌿 Nook - Initialisation"
echo "======================================"

# Vérifier que le binaire existe
if [ ! -f /app/nook-backend ]; then
    echo "❌ Binary not found at /app/nook-backend"
    exit 1
fi

# Créer le répertoire data s'il n'existe pas
mkdir -p /app/data

# Créer la base de données si elle n'existe pas
if [ ! -f /app/data/nook.db ]; then
    echo "📦 Creating new database..."
    sqlite3 /app/data/nook.db "VACUUM;"
    echo "✅ Database created at /app/data/nook.db"
else
    echo "✅ Database already exists"
fi

# Générer le token admin au premier lancement
if [ ! -f /app/data/admin.token ]; then
    echo "🔐 Generating admin token..."
    TOKEN=$(openssl rand -hex 32)
    echo "$TOKEN" > /app/data/admin.token
    chmod 600 /app/data/admin.token
    echo "✅ Admin token generated and saved"
    echo "📝 Your admin token: $TOKEN"
    echo "⚠️  Save this token securely!"
else
    echo "✅ Admin token already exists"
fi

# Afficher les informations de démarrage
echo "======================================"
echo "🚀 Starting Nook..."
echo "📊 Environment:"
echo "   - Database: $DATABASE_URL"
echo "   - Static files: $STATIC_FILES_DIR"
echo "   - Port: $PORT"
echo "   - Log level: $RUST_LOG"
echo "======================================"

# Lancer l'application
exec /app/nook-backend
EOF

# Définir l'utilisateur
USER app

# Variables d'environnement
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV STATIC_FILES_DIR=/app/static
ENV PORT=3000

# Exposer le port
EXPOSE 3000

# Volume pour la persistance des données
VOLUME ["/app/data"]

# Healthcheck pour Docker/Kubernetes
HEALTHCHECK --interval=30s --timeout=3s --start-period=15s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Point d'entrée
ENTRYPOINT ["/app/init.sh"]