#!/usr/bin/env bash
# =====================================================
# Script de Build Nook - Build Statique
# =====================================================

set -euo pipefail

# Couleurs pour les logs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fonctions utilitaires
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Variables
IMAGE_NAME="${IMAGE_NAME:-nook}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
FULL_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}"

# Banner
echo "======================================================"
echo "  🚀 Nook Build Statique Ultra-Léger"
echo "======================================================"
echo ""

# Vérifications pré-build
log_info "Vérification de l'environnement..."

# Vérifier Docker
if ! command -v docker &> /dev/null; then
    log_error "Docker n'est pas installé"
    exit 1
fi
log_success "Docker installé"

# Vérifier BuildKit
if ! docker buildx version &> /dev/null; then
    log_warning "BuildKit non disponible, utilisation du builder classique"
else
    log_success "BuildKit disponible"
    export DOCKER_BUILDKIT=1
fi

# Vérifier la structure du projet
log_info "Vérification de la structure du projet..."

if [ ! -d "backend" ] || [ ! -d "frontend" ]; then
    log_error "Structure du projet invalide (backend/ et frontend/ requis)"
    exit 1
fi

if [ ! -f "backend/Cargo.toml" ]; then
    log_error "backend/Cargo.toml introuvable"
    exit 1
fi

if [ ! -f "frontend/package.json" ]; then
    log_error "frontend/package.json introuvable"
    exit 1
fi

log_success "Structure du projet OK"

# Nettoyer les anciens builds (optionnel)
read -p "Voulez-vous nettoyer les anciens builds Docker ? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "Nettoyage des anciens builds..."
    docker system prune -f
    log_success "Nettoyage terminé"
fi

# Build de l'image
log_info "Démarrage du build de ${FULL_IMAGE}..."
echo ""

# Mesurer le temps de build
START_TIME=$(date +%s)

# Build avec cache
docker build \
    --tag "${FULL_IMAGE}" \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --progress=plain \
    .

BUILD_STATUS=$?
END_TIME=$(date +%s)
BUILD_DURATION=$((END_TIME - START_TIME))

echo ""
if [ $BUILD_STATUS -eq 0 ]; then
    log_success "Build réussi en ${BUILD_DURATION}s"
else
    log_error "Échec du build"
    exit 1
fi

# Informations sur l'image
log_info "Informations sur l'image:"
docker images "${FULL_IMAGE}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"

# Taille de l'image
IMAGE_SIZE=$(docker images "${FULL_IMAGE}" --format "{{.Size}}")
log_success "Taille de l'image: ${IMAGE_SIZE}"

# Vérification du binaire statique (optionnel)
log_info "Vérification du binaire..."
docker run --rm "${FULL_IMAGE}" /app/nook-backend --health && \
    log_success "Binaire fonctionnel" || \
    log_warning "Impossible de vérifier le binaire"

# Proposer de lancer un conteneur de test
echo ""
read -p "Voulez-vous lancer un conteneur de test ? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "Démarrage du conteneur de test..."
    
    # Créer les volumes si nécessaire
    mkdir -p ./data ./data/uploads
    
    docker run -d \
        --name nook-test \
        -p 6300:3000 \
        -v "$(pwd)/data:/app/data" \
        "${FULL_IMAGE}"
    
    sleep 2
    
    if docker ps | grep -q nook-test; then
        log_success "Conteneur démarré"
        log_info "Accessible sur: http://localhost:6300"
        log_info "Logs: docker logs -f nook-test"
        log_info "Arrêter: docker stop nook-test && docker rm nook-test"
    else
        log_error "Échec du démarrage du conteneur"
        docker logs nook-test
    fi
fi

# Résumé
echo ""
echo "======================================================"
log_success "Build terminé avec succès !"
echo "======================================================"
echo ""
echo "📦 Image: ${FULL_IMAGE}"
echo "💾 Taille: ${IMAGE_SIZE}"
echo "⏱️  Durée: ${BUILD_DURATION}s"
echo ""
echo "Commandes utiles:"
echo "  • Lancer: docker-compose up -d"
echo "  • Logs: docker-compose logs -f"
echo "  • Stop: docker-compose down"
echo ""