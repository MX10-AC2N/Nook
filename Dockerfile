# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm install && npm run build

# --- Build Backend ---
FROM rust:1.83-alpine AS backend-builder

# Installer les dépendances nécessaires pour le build
RUN apk add --no-cache musl-dev sqlite-dev sqlite

WORKDIR /app

# Copier les fichiers backend
COPY backend/ .

# Corriger Cargo.toml - retirer la feature "offline"
RUN if grep -q '"offline"' Cargo.toml; then \
      sed -i 's/features = \["runtime-tokio-rustls", "sqlite", "offline"\]/features = ["runtime-tokio-rustls", "sqlite"]/' Cargo.toml || true; \
    fi

# 1. Créer la base de données temporaire
RUN mkdir -p data && sqlite3 data/temp.db "VACUUM;"

# 2. Solution alternative : installer sqlx-cli depuis le projet actuel
# D'abord, ajouter sqlx-cli comme dépendance temporaire
RUN echo '[dependencies]' > /tmp/sqlx-cli.toml && \
    echo 'sqlx-cli = { version = "0.7.3", default-features = false, features = ["sqlite"] }' >> /tmp/sqlx-cli.toml

# 3. Exécuter les migrations si elles existent - version simplifiée sans sqlx-cli
# Si vous n'avez pas de migrations, vous pouvez sauter cette étape
# RUN echo "Skipping migrations in Docker build..."

# 4. Générer le cache SQLx SANS sqlx-cli (utilisation directe de cargo sqlx)
# Cette méthode évite l'installation de sqlx-cli
RUN cargo install --version 0.7.3 sqlx-cli --no-default-features --features sqlite || \
    (echo "Fallback: using older version" && \
     cargo install --version 0.7.2 sqlx-cli --no-default-features --features sqlite)

# 5. Générer le cache de requêtes (maintenant sqlx-cli est installé)
RUN DATABASE_URL="sqlite:data/temp.db" cargo sqlx prepare

# 6. Désinstaller sqlx-cli pour réduire la taille de l'image
RUN cargo uninstall sqlx-cli

# 7. Build en mode release
RUN cargo build --release

# --- Runtime ---
FROM alpine:3.19

# Installer SQLite runtime
RUN apk add --no-cache \
    sqlite \
    ca-certificates \
    libgcc

# Créer utilisateur non-root
RUN addgroup -g 1000 -S app && \
    adduser -u 1000 -S app -G app

# Créer structure de dossiers
RUN mkdir -p /app/data && chown -R app:app /app

WORKDIR /app

# Copier le binaire backend
COPY --from=backend-builder --chown=app:app /app/target/release/nook-backend /app/

# Copier les fichiers frontend buildés
COPY --from=frontend-builder --chown=app:app /app/build /app/static

# Créer d'abord le dossier migrations (peut être vide)
RUN mkdir -p /app/migrations
COPY --from=backend-builder --chown=app:app /app/migrations/. /app/migrations/ 2>/dev/null || :

# Définir l'utilisateur
USER app

# Variables d'environnement
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV STATIC_FILES_DIR=/app/static

EXPOSE 3000

# Créer la base de données au démarrage si elle n'existe pas
CMD ["sh", "-c", "if [ ! -f /app/data/nook.db ]; then sqlite3 /app/data/nook.db 'VACUUM;'; fi && /app/nook-backend"]