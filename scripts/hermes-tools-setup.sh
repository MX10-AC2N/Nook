#!/usr/bin/env bash
# scripts/hermes-tools-setup.sh
# Outils Hermes: curl, playwright, lightpanda

set -uo pipefail
echo "=== Hermes Tools Setup ==="

# curl
command -v curl &>/dev/null && echo "✅ curl OK" || (apt-get update -qq && apt-get install -y -qq curl && echo "✅ curl installed")

# Playwright
python3 -c "import playwright" 2>/dev/null && echo "✅ playwright OK" || (pip install --break-system-packages -q playwright && echo "✅ playwright installed")

# Chromium
[ -d "$HOME/.cache/ms-playwright/chromium"* ] 2>/dev/null && echo "✅ chromium OK" || (playwright install chromium 2>/dev/null && echo "✅ chromium installed")

# Lightpanda
LIGHTPANDA="/tmp/lightpanda"
if [ -f "$LIGHTPANDA" ] && [ -x "$LIGHTPANDA" ]; then
    echo "✅ lightpanda OK"
else
    echo "📦 Installing lightpanda (111MB)..."
    curl -L -o "$LIGHTPANDA" "https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux" --progress-bar
    chmod +x "$LIGHTPANDA"
    echo "✅ lightpanda installed"
fi

echo "=== Ready ==="
