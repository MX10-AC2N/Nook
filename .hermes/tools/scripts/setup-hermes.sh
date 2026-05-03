#!/bin/bash
# setup-hermes.sh - Initialisation environnement Hermes pour Nook
# À exécuter au démarrage du Docker ou manuellement

set -e

echo "🚀 === INITIALISATION HERMES POUR NOOK ==="
echo ""

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Répertoire de base
NOOK_DIR="/opt/data/home/.hermes/Nook"
HERMES_DIR="$NOOK_DIR/.hermes"

echo "📂 Répertoire Nook: $NOOK_DIR"
echo "📂 Répertoire Hermes: $HERMES_DIR"
echo ""

# 1. Vérifier que le repo est présent
if [ ! -d "$NOOK_DIR/.git" ]; then
    echo -e "${RED}❌ Erreur: Repo Nook non trouvé dans $NOOK_DIR${NC}"
    echo "Clonez le repo d'abord:"
    echo "  git clone https://github.com/MX10-AC2N/Nook $NOOK_DIR"
    exit 1
fi
echo -e "${GREEN}✅ Repo Nook trouvé${NC}"

# 2. Se placer sur la branche develop
cd "$NOOK_DIR"
git checkout develop 2>/dev/null || true
git pull origin develop 2>/dev/null || true
echo -e "${GREEN}✅ Branche develop à jour${NC}"

# 3. Vérifier/Git config
if [ "$(git config user.name)" != "Hermes Bot" ]; then
    git config user.name "Hermes Bot"
    git config user.email "hermes-bot@nook.app"
    echo -e "${GREEN}✅ Git configuré${NC}"
else
    echo -e "${GREEN}✅ Git déjà configuré${NC}"
fi

# 4. Vérifier les outils essentiels
echo ""
echo "🔧 === VÉRIFICATION OUTILS ==="

check_tool() {
    if command -v "$1" &> /dev/null; then
        VERSION=$($2 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✅ $1: $VERSION${NC}"
        return 0
    else
        echo -e "${RED}❌ $1 manquant${NC}"
        return 1
    fi
}

check_tool "git" "git --version"
check_tool "node" "node --version"
check_tool "npm" "npm --version"
check_tool "jq" "jq --version"
check_tool "curl" "curl --version | head -1"
check_tool "make" "make --version | head -1"
check_tool "gcc" "gcc --version | head -1"
check_tool "g++" "g++ --version | head -1"
check_tool "pkg-config" "pkg-config --version"
check_tool "rustc" "rustc --version"
check_tool "cargo" "cargo --version"
check_tool "gh" "gh --version"

# 5. Installer outils manquants si root
echo ""
if [ "$EUID" -eq 0 ]; then
    echo "🔧 === INSTALLATION OUTILS MANQUANTS ==="
    
    MISSING=""
    for tool in rustc cargo gh; do
        if ! command -v "$tool" &> /dev/null; then
            MISSING="$MISSING $tool"
        fi
    done
    
    if [ -n "$MISSING" ]; then
        echo "Installation de:$MISSING"
        apt-get update
        [ -z "$(command -v rustc)" ] && apt-get install -y rustc cargo
        [ -z "$(command -v gh)" ] && apt-get install -y gh
    else
        echo -e "${GREEN}✅ Tous les outils sont présents${NC}"
    fi
else
    echo -e "${YELLOW}⚠️ Pas root - skip installation outils (sudo ./setup-hermes.sh)${NC}"
fi

# 6. Créer lien symbolique si nécessaire
echo ""
echo "🔗 === LIENS SYMBOLIQUES ==="
if [ ! -L "$NOOK_DIR/.hermes" ] && [ -d "$HERMES_DIR" ]; then
    echo -e "${GREEN}✅ .hermes déjà présent dans le repo${NC}"
fi

# 7. Résumé
echo ""
echo "📋 === RÉSUMÉ ==="
echo "Répertoire Hermes: $HERMES_DIR"
echo "Session active: $(cat $HERMES_DIR/hermes/active-session.md 2>/dev/null | head -3 || echo 'Non définie')"
echo ""
echo -e "${GREEN}✅ Initialisation Hermes terminée${NC}"
echo ""
echo "📚 Prochaines étapes:"
echo "  1. Lire $HERMES_DIR/hermes/INDEX.md"
echo "  2. Consulter $HERMES_DIR/hermes/memory/core.md"
echo "  3. Vérifier $HERMES_DIR/hermes/active-session.md"
