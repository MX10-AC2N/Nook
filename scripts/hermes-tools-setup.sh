#!/usr/bin/env bash
# scripts/hermes-tools-setup.sh
# Installe les outils nécessaires pour Hermes (curl, playwright, lightpanda)
# À exécuter au début de chaque session si les outils manquent

set -uo pipefail

echo "=== Hermes Tools Setup ==="

# 1. curl
if command -v curl &>/dev/null; then
    echo "✅ curl: $(curl --version | head -1)"
else
    echo "📦 Installing curl..."
    apt-get update -qq && apt-get install -y -qq curl
    echo "✅ curl: $(curl --version | head -1)"
fi

# 2. Playwright (Python)
if python3 -c "import playwright" 2>/dev/null; then
    echo "✅ playwright: installed"
else
    echo "📦 Installing playwright..."
    pip install --break-system-packages -q playwright 2>/dev/null || pip install --break-system-packages playwright
    echo "✅ playwright: installed"
fi

# 3. Chromium for Playwright
if [ -d "$HOME/.cache/ms-playwright/chromium"* ] 2>/dev/null; then
    echo "✅ chromium: installed"
else
    echo "📦 Installing chromium..."
    playwright install chromium 2>/dev/null
    echo "✅ chromium: installed"
fi

# 4. Lightpanda (optional, large binary)
LIGHTPANDA="/tmp/lightpanda"
if [ -f "$LIGHTPANDA" ] && [ -x "$LIGHTPANDA" ]; then
    echo "✅ lightpanda: installed"
else
    echo "⚠️  lightpanda: not installed (111MB, install manually if needed)"
    echo "   Run: curl -L -o $LIGHTPANDA https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux && chmod +x $LIGHTPANDA"
fi

echo ""
echo "=== Tools Ready ==="
