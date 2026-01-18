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

# DEBUG: Afficher où sont réellement les bibliothèques
RUN echo "🔍 Recherche des bibliothèques installées..." && \
    find /usr/lib -name "libsqlite3.so*" -o -name "libsodium.so*" -o -name "libssl.so*" -o -name "libcrypto.so*" | sort

# Copier les bibliothèques dans un emplacement centralisé
# Support multi-architecture (amd64 et arm64)
# Note: dpkg retourne "amd64" mais le chemin est "x86_64-linux-gnu"
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
    ls -la ${LIB_DIR}/ | head -20 && \
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

# Création de la structure de répertoires
RUN mkdir -p /app/data /app/static /app/data/uploads

# Arguments pour les artifacts
ARG BACKEND_PATH
ARG FRONTEND_PATH

# Copie du backend pré-compilé
COPY ${BACKEND_PATH} /app/nook-backend

# Rendre le binaire exécutable
RUN chmod +x /app/nook-backend

# Installer ldd pour la vérification (uniquement pour cette étape)
RUN apt-get update && \
    apt-get install -y --no-install-recommends libc-bin && \
    rm -rf /var/lib/apt/lists/*

# Copier temporairement les libs pour le test ldd
RUN ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
        amd64) LIB_ARCH="x86_64-linux-gnu" ;; \
        arm64) LIB_ARCH="aarch64-linux-gnu" ;; \
        *) LIB_ARCH="unknown" ;; \
    esac && \
    LIB_DIR="/usr/lib/${LIB_ARCH}" && \
    mkdir -p ${LIB_DIR} && \
    cp /tmp/libs/* ${LIB_DIR}/ 2>/dev/null || true

# Vérifier les dépendances du binaire
RUN echo "🔍 Vérification des dépendances du binaire:" && \
    ldd /app/nook-backend && \
    if ldd /app/nook-backend | grep "not found"; then \
      echo "❌ ERREUR: Dépendances manquantes !"; \
      exit 1; \
    fi && \
    echo "✅ Toutes les dépendances sont satisfaites"

# Copie du frontend pré-buildé
COPY ${FRONTEND_PATH}/ /app/static/

# Appliquer les bons droits
RUN chown -R app:app /app

# Vérification finale
RUN set -e && \
    if [ ! -f "/app/nook-backend" ]; then \
      echo "❌ Backend absent"; exit 1; \
    fi && \
    if [ ! -f "/app/static/index.html" ]; then \
      echo "❌ Frontend absent"; exit 1; \
    fi && \
    echo "✅ Backend: $(ls -lh /app/nook-backend | awk '{print $5}')" && \
    echo "✅ Frontend: $(du -sh /app/static | cut -f1)"

# ===============================================
# ÉTAPE 3 : Image finale Distroless
# ===============================================
FROM gcr.io/distroless/cc-debian12:latest

# Métadonnées
LABEL maintainer="MX10-AC2N" \
      description="Nook - Messagerie familiale chiffrée E2EE" \
      version="0.5.0" \
      org.opencontainers.image.source="https://github.com/MX10-AC2N/Nook"

# Copie des fichiers système
COPY --from=app-prep /etc/passwd /etc/passwd
COPY --from=app-prep /etc/group /etc/group

# Copie des certificats SSL
COPY --from=libs-extractor /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# ⚠️ CRITIQUE: Copie des bibliothèques partagées
# Distroless cc-debian12 contient déjà:
# - glibc (libc.so.6)
# - libgcc_s.so.1
# - libm.so.6
# - libpthread (intégré dans glibc 2.34+)
#
# Les bibliothèques sont copiées dans /usr/lib/ (sans sous-répertoire)
# Le linker dynamique les trouvera automatiquement via LD_LIBRARY_PATH
COPY --from=libs-extractor /tmp/libs/*.so* /usr/lib/

# Copie de l'application complète
COPY --from=app-prep --chown=1000:1000 /app /app

WORKDIR /app

# Utilisateur non-root
USER 1000:1000

# Variables d'environnement
ENV RUST_LOG=info \
    DATABASE_URL=sqlite:/app/data/nook.db \
    PORT=3000 \
    LD_LIBRARY_PATH=/usr/lib

EXPOSE 3000

# Point d'entrée
CMD ["/app/nook-backend"]