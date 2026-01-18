# ===============================================
# Dockerfile Distroless optimisé pour production
# Image ultra-légère et sécurisée
# Compatible avec les artifacts GitHub Actions
# ===============================================

ARG BACKEND_PATH=backend-artifact/nook-backend
ARG FRONTEND_PATH=frontend-artifact

# ===============================================
# ÉTAPE 1 : Extraction des bibliothèques
# ===============================================
FROM debian:bookworm-slim AS libs-extractor

# Installation des bibliothèques nécessaires
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        libsodium23 \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Créer un répertoire pour collecter toutes les libs
RUN mkdir -p /tmp/libs

# Copier les bibliothèques dans un emplacement centralisé
# Support multi-architecture (amd64 et arm64)
RUN ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
        amd64) LIB_ARCH="x86_64-linux-gnu" ;; \
        arm64) LIB_ARCH="aarch64-linux-gnu" ;; \
        *) echo "❌ Architecture non supportée: $ARCH" && exit 1 ;; \
    esac && \
    LIB_DIR="/usr/lib/${LIB_ARCH}" && \
    echo "📦 Architecture dpkg: ${ARCH}" && \
    echo "📦 Architecture libs: ${LIB_ARCH}" && \
    echo "📂 Répertoire libs: ${LIB_DIR}" && \
    cp -P ${LIB_DIR}/libsqlite3.so* /tmp/libs/ && \
    cp -P ${LIB_DIR}/libsodium.so* /tmp/libs/ && \
    cp -P ${LIB_DIR}/libssl.so* /tmp/libs/ && \
    cp -P ${LIB_DIR}/libcrypto.so* /tmp/libs/ && \
    echo "✅ Bibliothèques collectées:" && \
    ls -lh /tmp/libs/

# ===============================================
# ÉTAPE 2 : Préparation de l'application
# ===============================================
FROM debian:bookworm-slim AS app-prep

# Copier les bibliothèques depuis l'extracteur
COPY --from=libs-extractor /tmp/libs /tmp/libs

# Création de l'utilisateur non-root
RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

# Création de la structure de répertoires avec permissions minimales
RUN mkdir -p /app/data /app/static /app/data/uploads && \
    chown -R app:app /app && \
    chmod 755 /app && \
    chmod 700 /app/data && \
    chmod 755 /app/static

# Arguments pour les artifacts
ARG BACKEND_PATH
ARG FRONTEND_PATH

# Copie du backend pré-compilé avec vérification
COPY --chown=app:app --chmod=755 ${BACKEND_PATH} /app/nook-backend

# Vérification du binaire
RUN echo "🔍 Vérification du binaire final:" && \
    ls -la /app/nook-backend && \
    file /app/nook-backend | grep -q "ELF 64-bit LSB.*executable" || (echo "❌ Format de binaire invalide" && exit 1) && \
    [ -x /app/nook-backend ] || (echo "❌ Binaire non exécutable" && exit 1) && \
    echo "✅ Binaire vérifié"

# Copie du frontend pré-buildé
COPY --chown=app:app ${FRONTEND_PATH}/ /app/static/

# Vérification finale de l'intégrité
RUN set -e && \
    echo "🔍 Vérification finale de l'application:" && \
    [ -f "/app/nook-backend" ] || (echo "❌ Backend absent" && exit 1) && \
    [ -f "/app/static/index.html" ] || (echo "❌ Frontend absent" && exit 1) && \
    echo "📊 Taille du backend: $(stat -c%s /app/nook-backend | numfmt --to=iec)" && \
    echo "📊 Taille du frontend: $(du -sh /app/static | cut -f1)" && \
    echo "📚 Bibliothèques requises:" && \
    ldd /app/nook-backend 2>/dev/null | grep "=> /" | awk '{print $1}' | sort || echo "⚪ Binaire statique" && \
    echo "✅ Application prête pour Distroless"

# ===============================================
# ÉTAPE 3 : Image finale Distroless
# ===============================================
FROM gcr.io/distroless/cc-debian12:nonroot

# Métadonnées
LABEL maintainer="MX10-AC2N" \
      description="Nook - Messagerie familiale chiffrée E2EE" \
      version="0.5.0" \
      org.opencontainers.image.source="https://github.com/MX10-AC2N/Nook"

# Copier les certificats SSL
COPY --from=libs-extractor /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copier les bibliothèques partagées
COPY --from=libs-extractor /tmp/libs/*.so* /usr/lib/

# Copier l'application complète
COPY --from=app-prep --chown=nonroot:nonroot /app /app

WORKDIR /app

# Variables d'environnement sécurisées
ENV RUST_LOG=warn \
    DATABASE_URL=sqlite:/app/data/nook.db \
    PORT=3000 \
    USER=nonroot \
    HOME=/app

EXPOSE 3000

# Point d'entrée sécurisé
ENTRYPOINT ["/app/nook-backend"]