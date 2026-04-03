#!/usr/bin/env bash
# generate-frontend-report.sh
# Génère .claude/FRONTEND-BUILD-REPORT.md
# Appelé par Frontend.yml
# Variables d'env attendues : RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL,
#   BUILD_STATUS, NODE_VER, BUILD_DURATION, OUTPUT_SIZE, FILE_COUNT
# Fichiers lus : /tmp/frontend-build.txt

set -euo pipefail
cd "${GITHUB_WORKSPACE:-.}"

RUN_DATE=$(date -u '+%Y-%m-%d %H:%M UTC')
COMMIT_SHORT="${COMMIT_SHA:0:7}"
RUN_URL="https://github.com/$REPO/actions/runs/$RUN_ID"
NODE_VER=$(node --version 2>/dev/null || echo "?")

[ "$BUILD_STATUS" = "OK" ] && STATUS_ICON="✅" || STATUS_ICON="❌"

# ── Build Timing ────────────────────────────────────────────────
VITE_MS=$(grep -oP 'built in \K[0-9.]+' /tmp/frontend-build.txt 2>/dev/null | head -1 || echo "N/A")
[ -z "$VITE_MS" ] && VITE_MS="N/A"

# ── Warning & Error Counts (trend tracking) ─────────────────────
WARN_COUNT=$(grep -cE "WARNING|warning\b" /tmp/frontend-build.txt 2>/dev/null || echo "0")
ERR_COUNT=$(grep -cE "^Error|error TS[0-9]+|✘ \[ERROR\]" /tmp/frontend-build.txt 2>/dev/null || echo "0")
CHUNK_COUNT=$(grep -cE "kB|gzip" /tmp/frontend-build.txt 2>/dev/null || echo "0")
[ ! -f /tmp/frontend-build.txt ] && WARN_COUNT="N/A"
[ ! -f /tmp/frontend-build.txt ] && ERR_COUNT="N/A"

# ── Route & Asset Listing ───────────────────────────────────────
ASSET_LIST="(non disponible — build échoué ou absent)"
if [ -d "frontend/build" ]; then
  ASSET_LIST=$(find frontend/build -type f \( -name "*.html" -o -name "*.js" -o -name "*.css" \) \
    -printf "%8s %P\n" 2>/dev/null | sort -rn | head -50 || echo "Aucun fichier trouvé")
fi

# ── Top 5 Largest Chunks (hotspots) ─────────────────────────────
TOP_CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | grep -v "^$" \
  | sort -rn -k3 2>/dev/null | head -5 \
  || echo "(non disponible)")

# ── Warnings svelte-plugin-vite ─────────────────────────────────
WARNINGS=$(grep -E "WARNING|warning\b|\[WARNING\]" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' \
  | grep -v "^$" | head -30 || echo "(aucun)")

# ── Erreurs TypeScript / Vite ───────────────────────────────────
ERRORS=$(grep -E "^Error|error TS[0-9]+|✘ \[ERROR\]|SyntaxError|Cannot find|Module not found" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -30 || echo "(aucune)")

# ── Chunks produits (taille finale du bundle) ───────────────────
CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -20 || echo "(non disponible)")

# ── Résumé vite (modules transformés, timing) ───────────────────
VITE_SUMMARY=$(grep -E "modules transformed|✓|vite v|rendering chunks|built in" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non disponible)")

# ── npm audit (vulnérabilités) ──────────────────────────────────
NPM_AUDIT=$(grep -E "vulnerabilit|high|critical|moderate|npm audit" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non vérifié)")

REPORT=".claude/FRONTEND-BUILD-REPORT.md"
cat > "$REPORT" << ENDOFMD
nd Build Report — Nook

utomatiquement par `Frontend.yml`
DATE}**





Valeur |
-------|
* | ${STATUS_ICON} ${BUILD_STATUS:-INCONNU} |
e** | `${BRANCH}` |
** | [`${COMMIT_SHA:0:7}`](https://github.com/$REPO/commit/${COMMIT_SHA}) |
s** | `${NODE_VER}` |
 Files** | ${FILE_COUNT} |
| [Voir le run](${RUN_URL}) |



d Metrics (AI Trend Tracking)

 | Valeur |
-|--------|
Duration** | ${BUILD_DURATION} |
uild Time** | ${VITE_MS}ms |
Output Size** | ${OUTPUT_SIZE} |
g Count** | ${WARN_COUNT} |
Count** | ${ERR_COUNT} |
Count** | ${CHUNK_COUNT} |

 ces valeurs entre les runs pour détecter les régressions.*



argest Chunks (Hotspot Detection)


KS}




 TypeScript / Vite







s svelte-vite-plugin (a11y, imports)


}




— modules et tailles (gzip)


MARY}






Files & Routes


ST}




it


T}




énéré par `.github/workflows/Frontend.yml`*


echo "✅ FRONTEND-BUILD-REPORT.md généré"
cat "$REPORT"
