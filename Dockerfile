# --- Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend/ .
# Clean install et build
RUN npm ci && npm run build

# --- Build Backend ---
FROM rust:1.83-alpine AS backend-builder

# Installer les dépendances nécessaires pour le build
RUN apk add --no-cache musl-dev sqlite-dev sqlite

WORKDIR /app

# Copier les fichiers backend
COPY backend/ .

# Vérifier et corriger Cargo.toml si nécessaire
RUN if grep -q '"offline"' Cargo.toml; then \
      sed -i 's/features = \["runtime-tokio-rustls", "sqlite", "offline"\]/features = ["runtime-tokio-rustls", "sqlite"]/' Cargo.toml || true; \
    fi

# Assurer que le cache SQLx existe
RUN if [ ! -d ".sqlx" ]; then \
      echo "Creating SQLx cache..."; \
      mkdir -p .sqlx; \
      echo '{"db":"SQLite","queries":{}}' > .sqlx/query.json; \
    fi

# Créer base de données temporaire
RUN mkdir -p data && sqlite3 data/temp.db "VACUUM;"

# Build en mode release
RUN cargo build --release

# --- Runtime ---
# Utiliser Alpine léger avec SQLite
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
COPY --from=frontend-builder --chown=app:app /app/dist /app/static

# Copier les migrations si elles existent
COPY --from=backend-builder --chown=app:app /app/migrations /app/migrations 2>/dev/null || echo "No migrations to copy"

# Définir l'utilisateur
USER app

# Variables d'environnement
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/nook.db
ENV STATIC_FILES_DIR=/app/static

EXPOSE 3000

# Créer la base de données au démarrage si elle n'existe pas
CMD ["sh", "-c", "if [ ! -f /app/data/nook.db ]; then sqlite3 /app/data/nook.db 'VACUUM;'; fi && /app/nook-backend"]