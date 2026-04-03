#!/usr/bin/env bash
# generate-backend-report.sh
# Génère .claude/BACKEND-BUILD-REPORT-{ARCH}.md

# DON'T USE set -euo pipefail — too strict, grep failures kill the script
set -uo pipefail

cd "${GITHUB_WORKSPACE:-.}"

# ── Safe defaults for ALL env vars ────────────────────────────────
ARCH="${ARCH:-unknown}"
RUN_DATE="${RUN_DATE:-unknown}"
COMMIT_SHA="${COMMIT_SHA:-unknown}"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="${BRANCH:-develop}"
RUN_URL="${RUN_URL:-#}"
RUSTC_VER="${RUSTC_VER:-unknown}"
BUILD_STATUS="${BUILD_STATUS:-INCONNU}"
BIN_SIZE="${BIN_SIZE:-N/A}"
CHECK_EXIT="${CHECK_EXIT:-?}"
CLIPPY_EXIT="${CLIPPY_EXIT:-?}"
BUILD_DURATION="${BUILD_DURATION:-N/A}"
WARNINGS_COUNT="${WARNINGS_COUNT:-N/A}"
ERRORS_COUNT="${ERRORS_COUNT:-N/A}"
NEW_WARNINGS="${NEW_WARNINGS:-N/A}"

REPORT=".claude/BACKEND-BUILD-REPORT-${ARCH}.md"

[ "$BUILD_STATUS" = "OK" ] && BUILD_ICON="✅" || BUILD_ICON="❌"
[ "$CHECK_EXIT"  = "0" ]   && CHECK_ICON="✅"  || CHECK_ICON="❌"
[ "$CLIPPY_EXIT" = "0" ]   && CLIPPY_ICON="✅" || CLIPPY_ICON="❌"

GLOBAL_STATUS="✅ OK"
if [ "$BUILD_STATUS" != "OK" ] || [ "$CHECK_EXIT" != "0" ] || [ "$CLIPPY_EXIT" != "0" ]; then
  GLOBAL_STATUS="❌ ÉCHEC"
fi

# ── Build Time Extraction ─────────────────────────────────────────
COMPILE_TIME="N/A"
[ -f /tmp/cargo-build.txt ] &&   COMPILE_TIME=$(grep -oP 'Finished .*(optimized|release).*profile.*in \K[0-9.]+' /tmp/cargo-build.txt 2>/dev/null | tail -1 || echo "N/A")
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

FINISHED_LINE="(non disponible)"
[ -f /tmp/cargo-build.txt ] &&   FINISHED_LINE=$(grep -iE "Finished " /tmp/cargo-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -3 || echo "non disponible")

COMPILE_PROGRESSION="(non disponible)"
[ -f /tmp/cargo-build.txt ] &&   COMPILE_PROGRESSION=$(grep -E "Compiling " /tmp/cargo-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -20 || echo "non disponible")

CHECK_WARNINGS="(aucun)"
[ -f /tmp/cargo-check.txt ] &&   CHECK_WARNINGS=$(grep -E "warning:" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -30 || echo "aucun")

CHECK_ERRORS="(aucune)"
[ -f /tmp/cargo-check.txt ] &&   CHECK_ERRORS=$(grep -E "error\[E" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -20 || echo "aucune")

CLIPPY_OUTPUT="(non disponible)"
[ -f /tmp/cargo-clippy.txt ] &&   CLIPPY_OUTPUT=$(cat /tmp/cargo-clippy.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -50 || echo "non disponible")

CLIPPY_LINTS="(non disponible)"
[ -f /tmp/cargo-clippy.txt ] &&   CLIPPY_LINTS=$(grep "warning: " /tmp/cargo-clippy.txt 2>/dev/null | sed 's/.*warning: //;s/:.*//' | sed 's/\x1b\[[0-9;]*m//g' | sort | uniq -c | sort -rn | head -10 || echo "non disponible")

DEPRECATED_WARNS="0"
DEAD_CODE_WARNS="0"
if [ -f /tmp/cargo-build.txt ]; then
  DEPRECATED_WARNS=$(grep -ci "deprecated" /tmp/cargo-build.txt 2>/dev/null || echo "0")
  DEAD_CODE_WARNS=$(grep -ciE "never used|dead code" /tmp/cargo-build.txt 2>/dev/null || echo "0")
fi

TEST_OUTPUT="(non disponible)"
[ -f /tmp/cargo-test.txt ] &&   TEST_OUTPUT=$(grep -E "test .* \.\.\. |running " /tmp/cargo-test.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -20 || echo "non disponible")

DEPS_INFO=""
[ -f Cargo.toml ] &&   DEPS_INFO=$(grep -A100 '\[dependencies\]' Cargo.toml 2>/dev/null | grep -B1 -A1 '=' | head -20 || true)

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
| **Commit** | ${COMMIT_SHORT} |
| **Rust** | \`${RUSTC_VER}\` |
| **Build** | ${BUILD_ICON} ${BUILD_STATUS} |
| **cargo check** | ${CHECK_ICON} exit=${CHECK_EXIT} |
| **cargo clippy** | ${CLIPPY_ICON} exit=${CLIPPY_EXIT} |
| **Binary Size** | ${BIN_SIZE} |
| **Compile Time** | ${COMPILE_TIME} |
| **Warnings** | ${WARNINGS_COUNT} |
| **Errors** | ${ERRORS_COUNT} |
| **New Warnings** | ${NEW_WARNINGS} |
| **Deprecated** | ${DEPRECATED_WARNS} |
| **Dead Code** | ${DEAD_CODE_WARNS} |
| **Run CI** | ${RUN_URL} |

---

## Compilation Progression (20 derniers crates)

\`\`\`
${COMPILE_PROGRESSION}

${FINISHED_LINE}
\`\`\`

---

## cargo check — Warnings

\`\`\`
${CHECK_WARNINGS}
\`\`\`

## cargo check — Errors

\`\`\`
${CHECK_ERRORS}
\`\`\`

---

## cargo clippy — Output

\`\`\`
${CLIPPY_OUTPUT}
\`\`\`

---

## cargo test — Résultats

\`\`\`
${TEST_OUTPUT}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-backend-report.sh\`*
ENDOFMD

echo "✅ BACKEND-BUILD-REPORT-${ARCH}.md généré"
exit 0
