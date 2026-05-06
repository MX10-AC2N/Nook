#!/bin/bash
# .hermes/tools/install-dev-tools.sh
# Script d'installation des outils manquants pour le développement Nook
# Usage: bash .hermes/tools/install-dev-tools.sh

set -e

echo "🔧 Installation des outils de développement Nook..."

# Mise à jour des paquets
sudo apt-get update

# Outils de base
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsodium-dev

# Protobuf (pour turn-rs)
sudo apt-get install -y protobuf-compiler

# Zig (pour cross-compilation Rust musl)
if ! command -v zig &> /dev/null; then
    sudo apt-get install -y zig
    echo "✅ zig installé"
else
    echo "✅ zig déjà présent"
fi

# Cargo tools (si cargo disponible)
if command -v cargo &> /dev/null; then
    # cargo-zigbuild pour cross-compile robuste
    if ! cargo zigbuild --version &> /dev/null; then
        cargo install cargo-zigbuild
        echo "✅ cargo-zigbuild installé"
    else
        echo "✅ cargo-zigbuild déjà présent"
    fi
fi

# Node.js + npm (pour frontend)
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt-get install -y nodejs
    echo "✅ Node.js 22 installé"
else
    echo "✅ Node.js déjà présent ($(node --version))"
fi

# Docker (pour test déploiement local)
if ! command -v docker &> /dev/null; then
    echo "🐳 Installation de Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    rm get-docker.sh
    echo "✅ Docker installé"
else
    echo "✅ Docker déjà présent ($(docker --version))"
fi

echo ""
echo "✅ Tous les outils de développement Nook sont installés."
echo "📋 Vérifions les versions :"
echo "  - Zig: $(zig version 2>/dev/null || echo 'non installé')"
echo "  - Node: $(node --version 2>/dev/null || echo 'non installé')"
echo "  - Protoc: $(protoc --version 2>/dev/null || echo 'non installé')"
echo "  - Cargo: $(cargo --version 2>/dev/null || echo 'non installé')"
echo "  - Docker: $(docker --version 2>/dev/null || echo 'non installé')"
