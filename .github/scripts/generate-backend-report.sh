#!/usr/bin/env bash
# .github/scripts/generate-backend-report.sh
# Generates .claude/BACKEND-BUILD-REPORT-{ARCH}.md
# Called by Backend.yml (runs per matrix target)
#
# Env vars expected:
#   ARCH, RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, BUILD_STATUS,
#   BIN_SIZE, CHECK_EXIT, CLIPPY_EXIT, BUILD_DURATION,
#   WARNINGS_COUNT, ERRORS_COUNT, NEW_WARNINGS, RUSTC_VER

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/BACKEND-BUILD-REPORT-${ARCH}.md"
COMMIT_SHORT="${COMMIT_SHA:0:7}"

[ "$BUILD_STATUS" = "OK" ] && BUILD_ICON="✅" || BUILD_ICON="❌"
[ "$CHECK_EXIT"  = "0" ]   && CHECK_ICON="✅"  || CHECK_ICON="❌"
[ "$CLIPPY_EXIT" = "0" ]   && CLIPPY_ICON="✅" || CLIPPY_ICON="❌"

GLOBAL_STATUS="✅ OK"
if [ "$BUILD_STATUS" != "OK" ] || [ "$CHECK_EXIT" != "0" ] || [ "$CLIPPY_EXIT" != "0" ]; then
  GLOBAL_STATUS="❌ ÉCHEC"
fi

# Build Time
COMPILE_TIME=$(grep -oP 'Finished .*(optimized|release).*in \K[0-9.]+' /tmp/cargo-build.txt 2>/dev/null | tail -1 || echo "N/A")
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

# Warnings
CHECK_WARNINGS=$(grep -cE "^warning:" /tmp/cargo-check.txt 2>/dev/null || echo "?")
CLIPPY_WARN_COUNT=$(grep -cE "^warning:" /tmp/cargo-clippy.txt 2>/dev/null || echo "?")
CLIPPY_CONTEXT=$(grep -E "^warning:.*--> .*(chess|main|admin|chess_|mod.rs):" /tmp/cargo-clippy.txt 2>/dev/null | head -10 || echo "(aucun)")
CLIPPY_LINTS=$(grep -oP '^warning: \K[a-z_]+' /tmp/cargo-clippy.txt 2>/dev/null | sort | uniq -c | sort -rn | head -10 || echo "(non disponible)")
BUILD_WARNINGS_FILE=$(grep -oP 'warning.*-->.*(src/[^:]+\.(rs|toml))' /tmp/cargo-build.txt 2>/dev/null | awk '{print $3}' | sort -u || echo "N/A")
BUILD_ERRORS=$(grep -cE "^error" /tmp/cargo-build.txt 2>/dev/null || echo "?")
CHECK_ERRORS=$(grep -cE "^error" /tmp/cargo-check.txt 2>/dev/null || echo "?")
CLIPPY_WARNS=$(grep -cE "^warning:" /tmp/cargo-clippy.txt 2>/dev/null || echo "?")
DEAD_CODE_WARNS=$(grep -c "dead_code\|never_used\|unused_" /tmp/cargo-clippy.txt 2>/dev/null || echo "?")
DEPRECATED_WARNS=$(grep -c "deprecated" /tmp/cargo-clippy.txt 2>/dev/null || echo "?")
FINISHED_LINE=$(grep -E "^\s*Finished" /tmp/cargo-build.txt 2>/dev/null | tail -1 | sed 's/\x1b\[[0-9;]*m//g' || echo "N/A")
COMPILE_PROGRESSION=$(grep -E "^\s*Compiling|Downloading|Compiling" /tmp/cargo-build.txt 2>/dev/null | tail -5 || echo "(non disponible)")

cat > "$REPORT" << ENDOFMD
# 🏗️ Backend Build Report — ${ARCH} — Nook

> Généré automatiquement par \`Backend.yml\` · target \`${ARCH}\`
> **${RUN_DATE}**

---

## Statut global : ${GLOBAL_STATUS}

| Champ | Valeur |
|-------|--------|
| **Architecture** | \`${ARCH}\` |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
| **Rust** | \`${RUSTC_VER}\` |
| **Build** | ${BUILD_ICON} ${BUILD_STATUS:-INCONNU} |
| **cargo check** | ${CHECK_ICON} exit=${CHECK_EXIT:-?} |
| **cargo clippy** | ${CLIPPY_ICON} exit=${CLIPPY_EXIT:-?} |
| **Taille binaire** | ${BIN_SIZE:-N/A} |
| **Durée** | ${BUILD_DURATION} |
| **Warnings (check)** | ${WARNINGS_COUNT:-N/A} |
| **Errors (check)** | ${CHECK_ERRORS:-N/A} |
| **Nouveaux warnings** | ${NEW_WARNINGS:-N/A} |
| **Warnings (clippy)** | ${CLIPPY_WARN_COUNT:?} |
| **Build errors** | ${BUILD_ERRORS:?} |
| **Deprecated** | ${DEPRECATED_WARNS:-?} |
| **Dead code** | ${DEAD_CODE_WARNS:-?} |
| **Run CI** | [Voir le run](${RUN_URL}) |

---

## Build Status

\`\`\`
${FINISHED_LINE}
\`\`\`

---

## Clippy — Top Lints

\`\`\`
${CLIPPY_LINTS}
\`\`\`

---

## Clippy — Context (fichiers chess/main/admin)

\`\`\`
${CLIPPY_CONTEXT}
\`\`\`

---

## Fichiers avec warnings

\`\`\`
${BUILD_WARNINGS_FILE}
\`\`\`

---

## Compilation progression (dernier step)

\`\`\`
${COMPILE_PROGRESSION}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-backend-report.sh\`*
ENDOFMD

echo "✅ BACKEND-BUILD-REPORT-${ARCH}.md généré ($(du -h $REPORT | cut -f1))"
