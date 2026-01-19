# ===============================================
# Dockerfile Distroless optimisé pour production
# Compatible avec les artifacts organisés
# Support multi-architecture (amd64/arm64)
# ===============================================

ARG BACKEND_PATH=backend
ARG FRONTEND_PATH=frontend

# ===============================================
# ÉTAPE 1 : Extraction des bibliothèques
# ===============================================
FROM --platform=$BUILDPLATFORM debian:bookworm-slim AS libs-extractor

# Installation minimale des bibliothèques nécessaires
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        libsodium23 \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Collecter les bibliothèques avec support multi-arch
RUN ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
        amd64) LIB_ARCH="x86_64-linux-gnu" ;; \
        arm64) LIB_ARCH="aarch64-linux-gnu" ;; \
        *) echo "❌ Architecture non supportée: $ARCH" && exit 1 ;; \
    esac && \
    mkdir -p /tmp/libs && \
    LIB_DIR="/usr/lib/${LIB_ARCH}" && \
    cp -P ${LIB_DIR}/libsqlite3.so* /tmp/libs/ 2>/dev/null || true && \
    cp -P ${LIB_DIR}/libsodium.so* /tmp/libs/ 2>/dev/null || true && \
    cp -P ${LIB_DIR}/libssl.so* /tmp/libs/ 2>/dev/null || true && \
    cp -P ${LIB_DIR}/libcrypto.so* /tmp/libs/ 2>/dev/null || true

# ÉTAPE 2 : Préparation sécurisée
FROM --platform=$BUILDPLATFORM debian:bookworm-slim AS app-prep

# Utilisateur non-root
RUN addgroup --system --gid 1000 app && \
    adduser --system --uid 1000 --ingroup app app

# Structure de répertoires sécurisée
RUN mkdir -p /app/data /app/static && \
    chown -R app:app /app && \
    chmod 755 /app && \
    chmod 700 /app/data && \
    chmod 755 /app/static

# Arguments d'architecture
ARG TARGETARCH
ARG BACKEND_PATH
ARG FRONTEND_PATH

# Copier le backend avec le bon nom d'architecture
COPY --chown=app:app --chmod=755 ${BACKEND_PATH}/nook-backend-${TARGETARCH} /app/nook-backend

# Copier le frontend
COPY --chown=app:app ${FRONTEND_PATH}/ /app/static/

# Vérification finale sécurisée
RUN set -e && \
    echo "✅ Vérification de l'application:" && \
    echo "🏗️  Architecture cible: ${TARGETARCH}" && \
    [ -x "/app/nook-backend" ] || (echo "❌ Backend non exécutable" && ls -la /app/ && exit 1) && \
    [ -f "/app/static/index.html" ] || (echo "❌ Frontend incomplet - index.html manquant" && ls -la /app/static/ && exit 1) && \
    [ -d "/app/static/_app" ] || (echo "❌ Frontend incomplet - dossier _app manquant" && ls -la /app/static/ && exit 1) && \
    [ -f "/app/static/_app/version.json" ] || (echo "❌ Frontend incomplet - version.json manquant" && exit 1) && \
    echo "📊 Backend: $(stat -c%s /app/nook-backend | numfmt --to=iec)" && \
    echo "📊 Frontend: $(du -sh /app/static | cut -f1)" && \
    echo "📊 Fichiers JS: $(find /app/static/_app -name '*.js' | wc -l) fichiers" && \
    echo "👤 Permissions backend: $(ls -l /app/nook-backend)" && \
    echo "👤 Permissions data: $(ls -ld /app/data)"

# ===============================================
# ÉTAPE 3 : Image finale Distroless
# ===============================================
FROM gcr.io/distroless/cc-debian12:nonroot

# Métadonnées
LABEL maintainer="MX10-AC2N" \
      description="Nook - Messagerie familiale chiffrée E2EE" \
      org.opencontainers.image.source="https://github.com/MX10-AC2N/Nook"

# Certificats SSL et bibliothèques
COPY --from=libs-extractor /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=libs-extractor /tmp/libs/ /usr/lib/

# Application
COPY --from=app-prep --chown=nonroot:nonroot /app /app

WORKDIR /app

# Variables d'environnement sécurisées
ENV RUST_LOG=warn \
    DATABASE_URL=sqlite:/app/data/nook.db \
    PORT=3000

EXPOSE 3000

ENTRYPOINT ["/app/nook-backend"]