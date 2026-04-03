#!/usr/bin/env bash
# generate-frontend-report.sh
# Génère .claude/FRONTEND-BUILD-REPORT.md
# Appelé par Frontend.yml via: bash .github/scripts/generate-frontend-report.sh
# Env vars passées par le workflow:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, BUILD_STATUS, NODE_VER,
#   BUILD_DURATION, OUTPUT_SIZE, FILE_COUNT

set -euo pipefail
cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/FRONTEND-BUILD-REPORT.md"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
[ "$BUILD_STATUS" = "OK" ] && STATUS_ICON="✅" || STATUS_ICON="❌"

# ── Build Timing ──────────────────────────────────────────────────────
VITE_MS=$(grep -oP 'built in \K[0-9.]+' /tmp/frontend-build.txt 2>/dev/null | head -1 || echo "N/A")
[ -z "$VITE_MS" ] && VITE_MS="N/A"

# ── Warning & Error Counts (trend tracking) ──────────────────────────
WARN_COUNT=$(grep -cE "WARNING|warning\b" /tmp/frontend-build.txt 2>/dev/null || echo "0")
ERR_COUNT=$(grep -cE "^Error|error TS[0-9]+|✘ \[ERROR\]" /tmp/frontend-build.txt 2>/dev/null || echo "0")
CHUNK_COUNT=$(grep -cE "kB|gzip" /tmp/frontend-build.txt 2>/dev/null || echo "0")
[ ! -f /tmp/frontend-build.txt ] && WARN_COUNT="N/A"
[ ! -f /tmp/frontend-build.txt ] && ERR_COUNT="N/A"

# ── Warnings ──────────────────────────────────────────────────────────
WARNINGS=$(grep -E "WARNING|warning\b|\[WARNING\]" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | grep -v "^$" | head -30 || echo "(aucun)")

# ── Erreurs ──────────────────────────────────────────────────────────
ERRORS=$(grep -E "^Error|error TS[0-9]+|✘ \[ERROR\]|SyntaxError|Cannot find|Module not found" \
  /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -30 || echo "(aucune)")

# ── Chunks (taille) ──────────────────────────────────────────────────
CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -30 || echo "(non disponible)")

# ── Résumé vite ──────────────────────────────────────────────────────
VITE_SUMMARY=$(grep -E "modules transformed|✓|vite v|rendering chunks|built in" \
  /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non disponible)")

# ── npm audit ────────────────────────────────────────────────────────
NPM_AUDIT=$(grep -E "vulnerabilit|high|critical|moderate|npm audit" \
  /tmp/frontend-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non vérifié)")

# ── Top 5 plus gros chunks ───────────────────────────────────────────
TOP_CHUNKS=$(echo "$CHUNKS" | grep -E "[0-9]+\." | sort -t',' -k1 -rn | head -5 || echo "(non disponible)")

# ── Fichiers de build ────────────────────────────────────────────────
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
| **Build** | ${STATUS_ICON} ${BUILD_STATUS:-INCONNU} |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
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

*Rapport généré par \`.github/scripts/generate-frontend-report.sh\`*
ENDOFMD

echo "✅ FRONTEND-BUILD-REPORT.md généré"
