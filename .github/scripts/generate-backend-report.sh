#!/usr/bin/env bash
# generate-backend-report.sh
# Génère .claude/BACKEND-BUILD-REPORT-{ARCH}.md
# Env vars passées par le workflow:
#   ARCH, RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, RUSTC_VER, BUILD_STATUS,
#   BIN_SIZE, CHECK_EXIT, CLIPPY_EXIT, BUILD_DURATION, WARNINGS_COUNT,
#   ERRORS_COUNT, NEW_WARNINGS

set -euo pipefail
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

# ── Build Time Extraction ─────────────────────────────────────────
COMPILE_TIME=$(grep -oP 'Finished .*(optimized|release).*profile.*in \K[0-9.]+' /tmp/cargo-build.txt 2>/dev/null | tail -1 || echo "N/A")
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

FINISHED_LINE=$(grep -iE "^\s*Finished " /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -3)
[ -z "$FINISHED_LINE" ] && FINISHED_LINE="(non disponible)"

COMPILE_PROGRESSION=$(grep -E "^\s*Compiling " /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -20)
[ -z "$COMPILE_PROGRESSION" ] && COMPILE_PROGRESSION="(non disponible)"

# ── Warnings from check ─────────────────────────────────────────────────────
CHECK_WARNINGS=$(grep -E "^warning:|^warning\[" /tmp/cargo-check.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -30 || echo "(aucun)")

CHECK_ERRORS=$(grep -E "^error\[E" /tmp/cargo-check.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -20 || echo "(aucune)")

# ── Clippy output ──────────────────────────────────────────────────
CLIPPY_OUTPUT=$(cat /tmp/cargo-clippy.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | head -50 || echo "(non disponible)")

# ── Clippy lint frequency ──────────────────────────────────────────
CLIPPY_LINTS=$(grep -E "^warning: " /tmp/cargo-clippy.txt 2>/dev/null \
  | sed 's/.*warning: //;s/:.*//;s/\\x1b\[[0-9;]*m//g' \
  | sort | uniq -c | sort -rn | head -10 || echo "(non disponible)")

# ── Deprecated / dead code ─────────────────────────────────────────
DEPRECATED_WARNS=$(grep -ciE "deprecated" /tmp/cargo-build.txt /tmp/cargo-clippy.txt 2>/dev/null | tail -1 | cut -d: -f2 || echo "0")
DEAD_CODE_WARNS=$(grep -ciE "never used|dead code" /tmp/cargo-build.txt /tmp/cargo-clippy.txt 2>/dev/null | tail -1 | cut -d: -f2 || echo "0")

# ── Tests output ───────────────────────────────────────────────────
TEST_OUTPUT=$(cat /tmp/cargo-test.txt 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -E "test .* \.\.\. |running " | tail -20 || echo "(non disponible)")

# ── Dependency info ────────────────────────────────────────────────
DEPS_INFO=""
if [ -f Cargo.toml ]; then
  DEPS_INFO=$(grep -A100 '\[dependencies\]' Cargo.toml | grep -B1 -A1 '=' | head -20)
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
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
| **Rust** | \`${RUSTC_VER}\` |
| **Build** | ${BUILD_ICON} ${BUILD_STATUS:-INCONNU} |
| **cargo check** | ${CHECK_ICON} exit=${CHECK_EXIT:-?} |
| **cargo clippy** | ${CLIPPY_ICON} exit=${CLIPPY_EXIT:-?} |
| **Binary Size** | ${BIN_SIZE:-N/A} |
| **Compile Time** | ${COMPILE_TIME} |
| **Warnings** | ${WARNINGS_COUNT:-N/A} |
| **Errors** | ${ERRORS_COUNT:-N/A} |
| **New Warnings** | ${NEW_WARNINGS:-N/A} |
| **Deprecated** | ${DEPRECATED_WARNS:-?} |
| **Dead Code** | ${DEAD_CODE_WARNS:-?} |
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
