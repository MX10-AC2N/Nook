#!/usr/bin/env bash
# generate-backend-report.sh
# Lit les fichiers /tmp/cargo-*.txt capturés pendant le build
# pour générer un rapport de debug exploitable.

set +e
cd "${GITHUB_WORKSPACE:-.}" 2>/dev/null || true

# ── Safe defaults ──────────────────────────────────────────────────
ARCH="${ARCH:-unknown}"
RUN_DATE="${RUN_DATE:-unknown}"
COMMIT_SHA="${COMMIT_SHA:-unknown}"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="${BRANCH:-develop}"
RUN_URL="${RUN_URL:-#}"
RUSTC_VER="${RUSTC_VER:-unknown}"
BUILD_STATUS="${BUILD_STATUS:-unknown}"
BIN_SIZE="${BIN_SIZE:-N/A}"
CHECK_EXIT="${CHECK_EXIT:-?}"
CLIPPY_EXIT="${CLIPPY_EXIT:-?}"
BUILD_DURATION="${BUILD_DURATION:-N/A}"
WARNINGS_COUNT="${WARNINGS_COUNT:-N/A}"
ERRORS_COUNT="${ERRORS_COUNT:-N/A}"
NEW_WARNINGS="${NEW_WARNINGS:-N/A}"

[ "$BUILD_STATUS" = "OK" ] && BUILD_ICON="✅" || BUILD_ICON="❌"
[ "$CHECK_EXIT"  = "0" ]   && CHECK_ICON="✅"  || CHECK_ICON="❌"
[ "$CLIPPY_EXIT" = "0" ]   && CLIPPY_ICON="✅" || CLIPPY_ICON="❌"

# ── Parse build output ────────────────────────────────────────────
# Finished line (duration)
FINISHED_LINE=""
[ -f /tmp/cargo-build.txt ] && FINISHED_LINE=$(grep -i "Finished" /tmp/cargo-build.txt 2>/dev/null | tail -1) || true

# Compile time
COMPILE_TIME=""
[ -f /tmp/cargo-build.txt ] && COMPILE_TIME=$(grep -oP '[0-9]+\.[0-9]+s' /tmp/cargo-build.txt 2>/dev/null | tail -1) || true
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="$BUILD_DURATION"
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

# Last 20 crates compiled (pour voir où ça a planté)
COMPILE_LAST=""
[ -f /tmp/cargo-build.txt ] && COMPILE_LAST=$(grep -E "^\s*(Compiling|Downloading|Building|Fresh)" /tmp/cargo-build.txt 2>/dev/null | tail -30) || true

# Warnings from check (with crate name)
CHECK_WARN_LIST=""
if [ -f /tmp/cargo-check.txt ]; then
  # Extract warning: lines with the file:line:col prefix
  CHECK_WARN_LIST=$(grep -E "warning:|warning\[" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -40) || true
fi
[ -z "$CHECK_WARN_LIST" ] && CHECK_WARN_LIST="(aucun warning)"

# Specific warning categories from check
DEPRECATED_WARNS="0"
DEAD_CODE_WARNS="0"
UNREACHABLE_WARNS="0"
UNUSED_VAR_WARNS="0"
if [ -f /tmp/cargo-check.txt ]; then
  DEPRECATED_WARNS=$(grep -ci "deprecated" /tmp/cargo-check.txt 2>/dev/null) || true
  DEAD_CODE_WARNS=$(grep -ciE "never used|dead code" /tmp/cargo-check.txt 2>/dev/null) || true
  UNREACHABLE_WARNS=$(grep -ci "unreachable" /tmp/cargo-check.txt 2>/dev/null) || true
  UNUSED_VAR_WARNS=$(grep -ciE "unused variable|unused " /tmp/cargo-check.txt 2>/dev/null) || true
fi

# Errors from check (critical — build may have failed here)
CHECK_ERROR_LIST=""
if [ -f /tmp/cargo-check.txt ]; then
  CHECK_ERROR_LIST=$(grep -E "^error|error\[" /tmp/cargo-check.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -20) || true
fi
[ -z "$CHECK_ERROR_LIST" ] && CHECK_ERROR_LIST="(aucune erreur)"

# Errors from build itself
BUILD_ERROR_LIST=""
if [ -f /tmp/cargo-build.txt ]; then
  BUILD_ERROR_LIST=$(grep -E "^error|error\[" /tmp/cargo-build.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -20) || true
fi
[ -z "$BUILD_ERROR_LIST" ] && BUILD_ERROR_LIST="(aucune erreur)"

# Clippy warnings
CLIPPY_WARN=""
if [ -f /tmp/cargo-clippy.txt ]; then
  CLIPPY_WARN=$(grep -E "warning:|warning\[" /tmp/cargo-clippy.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -40) || true
  # Top 10 clippy lint types
  CLIPPY_LINTS=$(grep -oP "warning: \[\w+" /tmp/cargo-clippy.txt 2>/dev/null | sort | uniq -c | sort -rn | head -10) || true
fi
[ -z "$CLIPPY_WARN" ] && CLIPPY_WARN="(non disponible ou aucun)"
[ -z "$CLIPPY_LINTS" ] && CLIPPY_LINTS="(non disponible)"

# Test results
TEST_RESULT=""
if [ -f /tmp/cargo-test.txt ]; then
  TEST_RESULT=$(grep -E "^test |running [0-9]+ test" /tmp/cargo-test.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g') || true
  TEST_SUMMARY=$(grep -E "test result:" /tmp/cargo-test.txt 2>/dev/null | tail -3 | sed 's/\x1b\[[0-9;]*m//g') || true
fi
[ -z "$TEST_RESULT" ] && TEST_RESULT="(non disponible)"
[ -z "$TEST_SUMMARY" ] && TEST_SUMMARY="(non disponible)"

# Dependency changes (if Cargo.lock changed)
DEP_DIFF=""
if [ -f Cargo.toml ]; then
  DEP_COUNT=$(grep -c "^[a-z_-]* " Cargo.toml 2>/dev/null) || true
  DEP_COUNT=$(grep -cE "^[a-z]" Cargo.toml 2>/dev/null) || true
fi

REPORT=".claude/BACKEND-BUILD-REPORT-${ARCH}.md"
mkdir -p .claude 2>/dev/null || true

cat > "$REPORT" << ENDOFMD
# 🏗️ Backend Build Report — ${ARCH} — Nook

> **${RUN_DATE}** | commit ${COMMIT_SHORT} | [run](${RUN_URL})

## Récapitulatif statuts

| Check | Résultat |
|-------|----------|
| **cargo build** | ${BUILD_ICON} ${BUILD_STATUS} |
| **cargo check** | ${CHECK_ICON} exit=${CHECK_EXIT} |
| **cargo clippy** | ${CLIPPY_ICON} exit=${CLIPPY_EXIT} |

| Métrique | Valeur |
|----------|--------|
| **Bin Size** | ${BIN_SIZE} |
| **Compile Time** | ${COMPILE_TIME} |
| **Warnings (check)** | ${WARNINGS_COUNT} |
| **Errors (check)** | ${ERRORS_COUNT} |
| **New Warnings** | ${NEW_WARNINGS} |
| **Deprecated refs** | ${DEPRECATED_WARNS} |
| **Dead code** | ${DEAD_CODE_WARNS} |
| **Unused vars** | ${UNUSED_VAR_WARNS} |
| **Unreachable** | ${UNREACHABLE_WARNS} |

---

## ⚠️ Warnings cargo check (top)

\`\`\`
${CHECK_WARN_LIST}
\`\`\`

## ❌ Erreurs cargo check

\`\`\`
${CHECK_ERROR_LIST}
\`\`\`

## ❌ Erreurs cargo build

\`\`\`
${BUILD_ERROR_LIST}
\`\`\`

## 🔧 Clippy warnings

\`\`\`
${CLIPPY_WARN}
\`\`\`

### Clippy lint types (top 10)

\`\`\`
${CLIPPY_LINTS}
\`\`\`

---

## 🧪 Tests

\`\`\`
${TEST_SUMMARY}

${TEST_RESULT}
\`\`\`

---

## 📦 Compilation (30 derniers crates)

\`\`\`
${COMPILE_LAST}

${FINISHED_LINE}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-backend-report.sh\`*
ENDOFMD

echo "✅ Backend report generated (${ARCH})"
exit 0
