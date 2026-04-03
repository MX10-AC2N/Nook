#!/usr/bin/env bash
# generate-frontend-report.sh
# Génère .claude/FRONTEND-BUILD-REPORT.md
# Appelé par Frontend.yml
# Variables d'environnement attendues :
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, BUILD_STATUS, NODE_VER
#   BUILD_DURATION, OUTPUT_SIZE, FILE_COUNT
# Fichiers lus : /tmp/frontend-build.txt

set -euo pipefail

cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/FRONTEND-BUILD-REPORT.md"
mkdir -p .claude

[ "$BUILD_STATUS" = "OK" ] && STATUS_ICON="✅" || STATUS_ICON="❌"

# ── Build Timing ──────────────────────────────────────────────────────
VITE_MS=$(grep -oP 'built in \K[0-9.]+' /tmp/frontend-build.txt 2>/dev/null \| head -1 || echo "N/A")
[ -z "$VITE_MS" ] && VITE_MS="N/A"

# ── Warnings ──────────────────────────────────────────────────────────
WARNINGS=$(grep -E "WARNING|warning|\[WARNING\]" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' \
  | grep -v "^$" | head -50 || echo "(aucun)")

# ── Erreurs ───────────────────────────────────────────────────────────
ERRORS=$(grep -E "^Error|error TS[0-9]+|✘ \[ERROR\]|SyntaxError|Cannot find|Module not found" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -50 || echo "(aucune)")

# ── Chunks (taille) ───────────────────────────────────────────────────
CHUNKS=$(grep -E "kB|gzip" /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -30 || echo "(non disponible)")

# ── Résumé vite ───────────────────────────────────────────────────────
VITE_SUMMARY=$(grep -E "modules transformed|✓|vite v|rendering chunks|built in" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non disponible)")

# ── npm audit ─────────────────────────────────────────────────────────
NPM_AUDIT=$(grep -E "vulnerabilit|high|critical|moderate|npm audit" \
  /tmp/frontend-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -10 || echo "(non vérifié)")

# ── Top 5 plus gros chunks ────────────────────────────────────────────
TOP_CHUNKS=$(echo "$CHUNKS" | grep -E "[0-9]+\." | sort -t',' -k1 -rn | head -5 || echo "(non disponible)")

# ── Liste fichiers de build ───────────────────────────────────────────
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
| **Commit** | [\`${COMMIT_SHA:0:7}\`](https://github.com/${{ github.repository }}/commit/${COMMIT_SHA}) |
| **Node.js** | \`${NODE_VER}\` |
| **Build Time** | ${BUILD_DURATION} |
| **Vite time** | ${VITE_MS} |
| **Output Size** | ${OUTPUT_SIZE} |
| **File Count** | ${FILE_COUNT} |
| **Run** | [Voir le run](${RUN_URL}) |

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

## Top 5 plus gros chunks

\`\`\`
${TOP_CHUNKS}
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
