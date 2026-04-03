#!/usr/bin/env bash
# .github/scripts/generate-frontend-report.sh
# Generates .claude/FRONTEND-BUILD-REPORT.md from build artifacts
# Called by Frontend.yml
#
# Env vars expected from workflow:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, BUILD_STATUS,
#   NODE_VER, BUILD_DURATION, OUTPUT_SIZE, FILE_COUNT

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/FRONTEND-BUILD-REPORT.md"
COMMIT_SHORT="${COMMIT_SHA:0:7}"

[ "$BUILD_STATUS" = "OK" ] && STATUS_ICON="✅" || STATUS_ICON="❌"

# Build Timing
VITE_MS=$(grep -oP 'built in \K[0-9.]+' /tmp/frontend-build.txt 2>/dev/null | head -1 || echo "N/A")
[ -z "$VITE_MS" ] && VITE_MS="N/A"

# Warnings
WARNINGS=$(grep -E "WARNING|warning|\[WARNING\]" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -20 || echo "(aucun)")

# Errors
ERRORS=$(grep -E "^Error|error TS|✘ \[ERROR\]|SyntaxError|Cannot find|Module not found" \
  /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -20 || echo "(aucune)")

# Chunks
CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' || echo "(non disponible)")

# Vite summary
VITE_SUMMARY=$(grep -E "modules transformed|✓|vite v|rendering chunks|built in" \
  /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' || echo "(non disponible)")

cat > "$REPORT" << ENDOFMD
# 🎨 Frontend Build Report — Nook

> Généré automatiquement par \`Frontend.yml\`
> **${RUN_DATE}**

---

## Statut

| Champ | Valeur |
|-------|--------|
| **Build** | ${STATUS_ICON} ${BUILD_STATUS:-INCONNU} |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
| **Node.js** | \`${NODE_VER}\` |
| **Vite** | ${VITE_MS} |
| **Durée** | ${BUILD_DURATION} |
| **Taille** | ${OUTPUT_SIZE} |
| **Fichiers** | ${FILE_COUNT} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Build Timing

\`\`\`
${VITE_SUMMARY}
\`\`\`

---

## Warnings svelte-vite-plugin (a11y, imports)

\`\`\`
${WARNINGS}
\`\`\`

---

## Erreurs TypeScript / Vite

\`\`\`
${ERRORS}
\`\`\`

---

## Bundle — modules et tailles (gzip)

\`\`\`
${CHUNKS}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-frontend-report.sh\`*
ENDOFMD

echo "✅ FRONTEND-BUILD-REPORT.md généré en $(du -h $REPORT | cut -f1)"
