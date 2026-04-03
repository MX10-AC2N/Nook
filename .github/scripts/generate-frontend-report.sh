#!/usr/bin/env bash
# generate-frontend-report.sh
# Génère .claude/FRONTEND-BUILD-REPORT.md
# Env vars passées par le workflow:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, BUILD_STATUS, NODE_VER,
#   BUILD_DURATION, OUTPUT_SIZE, FILE_COUNT

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

# Safe defaults
COMMIT_SHA="${COMMIT_SHA:-unknown}"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="${BRANCH:-develop}"
RUN_DATE="${RUN_DATE:-unknown}"
RUN_URL="${RUN_URL:-#}"
BUILD_STATUS="${BUILD_STATUS:-INCONNU}"
NODE_VER="${NODE_VER:-inconnu}"
BUILD_DURATION="${BUILD_DURATION:-N/A}"
OUTPUT_SIZE="${OUTPUT_SIZE:-N/A}"
FILE_COUNT="${FILE_COUNT:-N/A}"

REPORT=".claude/FRONTEND-BUILD-REPORT.md"
[ "$BUILD_STATUS" = "OK" ] && STATUS_ICON="✅" || STATUS_ICON="❌"

# ── Build Timing ──────────────────────────────────────────────────
VITE_MS="N/A"
if [ -f /tmp/frontend-build.txt ]; then
  VITE_MS=$(grep -oP 'built in \K[0-9.]+' /tmp/frontend-build.txt 2>/dev/null | head -1) || true
fi
[ -z "$VITE_MS" ] && VITE_MS="N/A"

# ── Warning & Error Counts ────────────────────────────────────────
WARN_COUNT="N/A"
ERR_COUNT="N/A"
CHUNK_COUNT="N/A"
if [ -f /tmp/frontend-build.txt ]; then
  WARN_COUNT=$(grep -cE "WARNING|warning\b" /tmp/frontend-build.txt 2>/dev/null) || true
  ERR_COUNT=$(grep -cE "^Error|error TS[0-9]+|✘ \[ERROR\]" /tmp/frontend-build.txt 2>/dev/null) || true
  CHUNK_COUNT=$(grep -cE "kB|gzip" /tmp/frontend-build.txt 2>/dev/null) || true
fi
[ -z "$WARN_COUNT" ] && WARN_COUNT="N/A"
[ -z "$ERR_COUNT" ] && ERR_COUNT="N/A"
[ -z "$CHUNK_COUNT" ] && CHUNK_COUNT="N/A"

# ── Warnings ──────────────────────────────────────────────────────
WARNINGS="(aucun)"
if [ -f /tmp/frontend-build.txt ]; then
  WARNINGS=$(grep -E "WARNING|warning\b|\[WARNING\]" /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -v "^$" | head -30) || true
fi
[ -z "$WARNINGS" ] && WARNINGS="(aucun)"

# ── Erreurs ───────────────────────────────────────────────────────
ERRORS="(aucune)"
if [ -f /tmp/frontend-build.txt ]; then
  ERRORS=$(grep -E "^Error|error TS[0-9]+|✘ \[ERROR\]|SyntaxError|Cannot find|Module not found" /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -30) || true
fi
[ -z "$ERRORS" ] && ERRORS="(aucune)"

# ── Chunks ────────────────────────────────────────────────────────
CHUNKS="(non disponible)"
if [ -f /tmp/frontend-build.txt ]; then
  CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -30) || true
fi
[ -z "$CHUNKS" ] && CHUNKS="(non disponible)"

# ── Résumé vite ───────────────────────────────────────────────────
VITE_SUMMARY="(non disponible)"
if [ -f /tmp/frontend-build.txt ]; then
  VITE_SUMMARY=$(grep -E "modules transformed|✓|vite v|rendering chunks|built in" /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -10) || true
fi
[ -z "$VITE_SUMMARY" ] && VITE_SUMMARY="(non disponible)"

# ── npm audit ─────────────────────────────────────────────────────
NPM_AUDIT="(non vérifié)"
if [ -f /tmp/frontend-build.txt ]; then
  NPM_AUDIT=$(grep -E "vulnerabilit|high|critical|moderate|npm audit" /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -10) || true
fi
[ -z "$NPM_AUDIT" ] && NPM_AUDIT="(non vérifié)"

# ── Top 5 plus gros chunks ────────────────────────────────────────
TOP_CHUNKS="(non disponible)"
if [ -n "$CHUNKS" ] && [ "$CHUNKS" != "(non disponible)" ]; then
  TOP_CHUNKS=$(echo "$CHUNKS" | grep -E "[0-9]+\." | sort -t',' -k1 -rn | head -5) || true
fi
[ -z "$TOP_CHUNKS" ] && TOP_CHUNKS="(non disponible)"

# ── Fichiers de build ─────────────────────────────────────────────
BUILD_FILES="build/ introuvable"
if [ -d build/ ]; then
  BUILD_FILES=$(find build -type f | head -30)
fi

cat > "$REPORT" << ENDOFMD
# 🎨 Frontend Build Report — Nook

> Généré automatiquement par \`Frontend.yml\`
> **${RUN_DATE}**

---

## Statut

| Champ | Valeur |
|-------|--------|
| **Build** | ${STATUS_ICON} ${BUILD_STATUS} |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](RUN_URL/commit/${COMMIT_SHA}) |
| **Node.js** | \`${NODE_VER}\` |
| **Vite time** | ${VITE_MS}ms |
| **Build Duration** | ${BUILD_DURATION} |
| **Output Size** | ${OUTPUT_SIZE} |
| **File Count** | ${FILE_COUNT} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Build Metrics (trend tracking)

| Metric | Valeur |
|--------|--------|
| **Warnings** | ${WARN_COUNT} |
| **Errors** | ${ERR_COUNT} |
| **Chunks** | ${CHUNK_COUNT} |

---

## Erreurs TypeScript / Vite

\`\`\`
${ERRORS}
\`\`\`

---

## Warnings svelte-vite-plugin (a11y, imports)

\`\`\`
${WARNINGS}
\`\`\`

---

## Bundle — modules et tailles (gzip)

\`\`\`
${VITE_SUMMARY}

${CHUNKS}
\`\`\`

---

## npm audit

\`\`\`
${NPM_AUDIT}
\`\`\`

---

## Fichiers de build

\`\`\`
${BUILD_FILES}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-frontend-report.sh\`*
ENDOFMD

echo "✅ FRONTEND-BUILD-REPORT.md généré"
exit 0
