#!/usr/bin/env bash
# generate-backend-report.sh
# Génère .claude/BACKEND-BUILD-REPORT-{ARCH}.md
# Env vars passées par le workflow:
#   ARCH, RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, RUSTC_VER, BUILD_STATUS,
#   BIN_SIZE, CHECK_EXIT, CLIPPY_EXIT, WARNINGS_COUNT, ERRORS_COUNT, NEW_WARNINGS

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

# Safe defaults for all env vars
ARCH="${ARCH:-unknown}"
COMMIT_SHA="${COMMIT_SHA:-unknown}"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="${BRANCH:-develop}"
RUN_DATE="${RUN_DATE:-unknown}"
RUN_URL="${RUN_URL:-#}"
RUSTC_VER="${RUSTC_VER:-unknown}"
BUILD_STATUS="${BUILD_STATUS:-INCONNU}"
BIN_SIZE="${BIN_SIZE:-N/A}"
CHECK_EXIT="${CHECK_EXIT:-?}"
CLIPPY_EXIT="${CLIPPY_EXIT:-?}"
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
if [ -f /tmp/cargo-build.txt ]; then
  COMPILE_TIME=$(grep -oP 'Finished .*(optimized|release).*profile.*in \K[0-9.]+' /tmp/cargo-build.txt 2>/dev/null | tail -1) || true
fi
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

FINISHED_LINE="(non disponible)"
if [ -f /tmp/cargo-build.txt ]; then
  FINISHED_LINE=$(grep -iE "^\s*Finished " /tmp/cargo-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -3) || true
fi
[ -z "$FINISHED_LINE" ] && FINISHED_LINE="(non disponible)"

COMPILE_PROGRESSION="(non disponible)"
if [ -f /tmp/cargo-build.txt ]; then
  COMPILE_PROGRESSION=$(grep -E "^\s*Compiling " /tmp/cargo-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | tail -20) || true
fi
[ -z "$COMPILE_PROGRESSION" ] && COMPILE_PROGRESSION="(non disponible)"

# ── Warnings from check ───────────────────────────────────────────
CHECK_WARNINGS="(aucun)"
if [ -f /tmp/cargo-check.txt ]; then
  CHECK_WARNINGS=$(grep -E "^warning:|^warning\[" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -30) || true
fi
[ -z "$CHECK_WARNINGS" ] && CHECK_WARNINGS="(aucun)"

CHECK_ERRORS="(aucune)"
if [ -f /tmp/cargo-check.txt ]; then
  CHECK_ERRORS=$(grep -E "^error\[E" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -20) || true
fi
[ -z "$CHECK_ERRORS" ] && CHECK_ERRORS="(aucune)"

# ── Clippy output ─────────────────────────────────────────────────
CLIPPY_OUTPUT="(non disponible)"
if [ -f /tmp/cargo-clippy.txt ]; then
  CLIPPY_OUTPUT=$(cat /tmp/cargo-clippy.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -50) || true
fi
[ -z "$CLIPPY_OUTPUT" ] && CLIPPY_OUTPUT="(non disponible)"

# ── Clippy lint frequency ─────────────────────────────────────────
CLIPPY_LINTS="(non disponible)"
if [ -f /tmp/cargo-clippy.txt ]; then
  CLIPPY_LINTS=$(grep -E "^warning: " /tmp/cargo-clippy.txt 2>/dev/null | sed 's/.*warning: //;s/:.*//' | sed 's/\x1b\[[0-9;]*m//g' | sort | uniq -c | sort -rn | head -10) || true
fi
[ -z "$CLIPPY_LINTS" ] && CLIPPY_LINTS="(non disponible)"

# ── Deprecated / dead code ────────────────────────────────────────
DEPRECATED_WARNS="?"
DEAD_CODE_WARNS="?"
if [ -f /tmp/cargo-build.txt ] || [ -f /tmp/cargo-clippy.txt ]; then
  DEPRECATED_WARNS=$(cat /tmp/cargo-build.txt /tmp/cargo-clippy.txt 2>/dev/null | grep -ciE "deprecated") || true
  DEAD_CODE_WARNS=$(cat /tmp/cargo-build.txt /tmp/cargo-clippy.txt 2>/dev/null | grep -ciE "never used|dead code") || true
fi
[ -z "$DEPRECATED_WARNS" ] && DEPRECATED_WARNS="?"
[ -z "$DEAD_CODE_WARNS" ] && DEAD_CODE_WARNS="?"

# ── Tests output ──────────────────────────────────────────────────
TEST_OUTPUT="(non disponible)"
if [ -f /tmp/cargo-test.txt ]; then
  TEST_OUTPUT=$(cat /tmp/cargo-test.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -E "test .* \.\.\. |running ") || true
fi
[ -z "$TEST_OUTPUT" ] && TEST_OUTPUT="(non disponible)"

# ── Dependency info ───────────────────────────────────────────────
DEPS_INFO=""
if [ -f Cargo.toml ]; then
  DEPS_INFO=$(grep -A100 '\[dependencies\]' Cargo.toml | grep -B1 -A1 '=' | head -20) || true
fi

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
| **Commit** | [\`${COMMIT_SHORT}\`](RUN_URL/commit/${COMMIT_SHA}) |
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
| **Run CI** | [Voir le run](${RUN_URL}) |

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

## cargo clippy — Top Lints

\`\`\`
${CLIPPY_LINTS}
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

## Dépendances principales

\`\`\`
${DEPS_INFO}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-backend-report.sh\`*
ENDOFMD

echo "✅ BACKEND-BUILD-REPORT-${ARCH}.md généré"
exit 0
