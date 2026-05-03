#!/bin/bash
# check-tools.sh - Vérification rapide des outils pour Nook
# Usage: bash check-tools.sh

echo "🔧 === VÉRIFICATION OUTILS NOOK ==="
echo ""

# Couleurs
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Compteur
OK=0
KO=0

check() {
    if command -v "$1" &> /dev/null; then
        VERSION=$($2 2>/dev/null | head -1 || echo "unknown")
        echo -e "${GREEN}✅ $1${NC} - $VERSION"
        ((OK++))
        return 0
    else
        echo -e "${RED}❌ $1${NC} - MANQUANT"
        ((KO++))
        return 1
    fi
}

# Outils essentiels
echo "📦 Outils Essentiels:"
check "git" "git --version"
check "node" "node --version"
check "npm" "npm --version"
check "jq" "jq --version"
check "curl" "curl --version"
check "make" "make --version"
check "gcc" "gcc --version"
check "g++" "g++ --version"
check "pkg-config" "pkg-config --version"
echo ""

# Rust toolchain
echo "🦀 Rust Toolchain:"
check "rustc" "rustc --version"
check "cargo" "cargo --version"
echo ""

# GitHub CLI
echo "🐙 GitHub CLI:"
check "gh" "gh --version"
echo ""

# Résumé
echo "📊 === RÉSUMÉ ==="
echo -e "${GREEN}$OK outils présents${NC}"
if [ $KO -gt 0 ]; then
    echo -e "${RED}$KO outils manquants${NC}"
    echo ""
    echo "🔧 Pour installer (en root):"
    echo "  apt-get update"
    echo "  apt-get install -y rustc cargo gh"
    exit 1
else
    echo -e "${GREEN}✅ Tous les outils sont installés${NC}"
    exit 0
fi
