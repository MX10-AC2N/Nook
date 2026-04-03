#!/usr/bin/env bash
# generate-backend-report.sh
# Génère .claude/BACKEND-BUILD-REPORT-{ARCH}.md
# Appelé par Backend.yml (matrix job)
# Variables d'env attendues : ARCH, RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL,
#   BUILD_STATUS, BIN_SIZE, CHECK_EXIT, CLIPPY_EXIT, BUILD_DURATION,
#   WARNINGS_COUNT, ERRORS_COUNT, NEW_WARNINGS
# Fichiers lus : /tmp/cargo-build.txt, /tmp/cargo-check.txt, /tmp/cargo-clippy.txt, /tmp/cargo-test.txt

set -euo pipefail
cd "${GITHUB_WORKSPACE:-.}"

# ARCH is set via workflow env
RUN_DATE=$(date -u '+%Y-%m-%d %H:%M UTC')
COMMIT_SHORT="${COMMIT_SHA:0:7}"
RUN_URL="https://github.com/$REPO/actions/runs/$RUN_ID"
RUSTC_VER=$(rustc --version 2>/dev/null || echo "?")

[ "$BUILD_STATUS" = "OK" ] && BUILD_ICON="✅" || BUILD_ICON="❌"
[ "$CHECK_EXIT"  = "0" ]   && CHECK_ICON="✅"  || CHECK_ICON="❌"
[ "$CLIPPY_EXIT" = "0" ]   && CLIPPY_ICON="✅" || CLIPPY_ICON="❌"

if [ "$BUILD_STATUS" = "OK" ] && [ "$CHECK_EXIT" = "0" ] && [ "$CLIPPY_EXIT" = "0" ]; then
  GLOBAL_STATUS="✅ OK"
else
  GLOBAL_STATUS="❌ FAIL"
fi

# ── Build Time Extraction ─────────────────────────────────────
COMPILE_TIME=$(grep -oP 'Finished .*(optimized|release).*profile.*in \K[0-9.]+' /tmp/cargo-build.txt 2>/dev/null | tail -1 || echo "N/A")
[ -z "$COMPILE_TIME" ] && COMPILE_TIME="N/A"

FINISHED_LINE=$(grep -iE "^\s*Finished " /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -3)
[ -z "$FINISHED_LINE" ] && FINISHED_LINE="(non disponible)"

COMPILE_PROGRESSION=$(grep -E "^\s*Compiling " /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -20)
[ -z "$COMPILE_PROGRESSION" ] && COMPILE_PROGRESSION="(non disponible)"

# ── Warning Counts (AI Trend Tracking) ────────────────────────
CHECK_WARNING_COUNT=$(grep -cE "^warning" /tmp/cargo-check.txt 2>/dev/null || echo "0")
CLIPPY_WARNING_COUNT=$(grep -cE "^warning" /tmp/cargo-clippy.txt 2>/dev/null || echo "0")
BUILD_WARNING_COUNT=$(grep -cE "^warning" /tmp/cargo-build.txt 2>/dev/null || echo "0")
DEPRECATED_COUNT=$(grep -ciE "deprecated" /tmp/cargo-build.txt 2>/dev/null || echo "0")
DEAD_CODE_COUNT=$(grep -cE "never used|dead code" /tmp/cargo-build.txt 2>/dev/null || echo "0")

# ── Erreurs cargo check ───────────────────────────────────────
CHECK_ERRORS=$(grep -E "^error\[|^error:" /tmp/cargo-check.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -30 || true)
[ -z "$CHECK_ERRORS" ] && CHECK_ERRORS="(aucune)"

# ── Compilation Warnings file-by-file (build) ─────────────────
BUILD_WARNINGS_FILE=$(grep -E "^warning| -->" /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -50 || true)
[ -z "$BUILD_WARNINGS_FILE" ] && BUILD_WARNINGS_FILE="(aucun)"

# ── Deprecated Warnings Detection ─────────────────────────────
DEPRECATED_WARNS=$(grep -iE "warning.*deprecated|^.*is deprecated" /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -20 || true)
[ -z "$DEPRECATED_WARNS" ] && DEPRECATED_WARNS="(aucune déprécation détectée)"

# ── Dead Code Warnings ────────────────────────────────────────
DEAD_CODE_WARNS=$(grep -E "^warning:.*never used|^warning:.*dead code" /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -20 || true)
[ -z "$DEAD_CODE_WARNS" ] && DEAD_CODE_WARNS="(aucun dead code détecté)"

# ── Warnings clippy + contexte fichier:ligne ──────────────────
CLIPPY_WARNS=$(grep -E "^warning\[|^warning:" /tmp/cargo-clippy.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -40 || true)
[ -z "$CLIPPY_WARNS" ] && CLIPPY_WARNS="(aucun)"

CLIPPY_CONTEXT=$(grep -E "^warning|[^-]-->" /tmp/cargo-clippy.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -60 || true)
[ -z "$CLIPPY_CONTEXT" ] && CLIPPY_CONTEXT="(aucun)"

# ── Clippy Lint Breakdown by type ─────────────────────────────
CLIPPY_LINTS=$(grep -oP "warning\[clippy::\K[^\]]+" /tmp/cargo-clippy.txt 2>/dev/null \
  | sort | uniq -c | sort -rn | head -15 || true)
[ -z "$CLIPPY_LINTS" ] && CLIPPY_LINTS="(aucune violation clippy)"

# ── Erreurs build release ─────────────────────────────────────
BUILD_ERRORS=$(grep -E "^error\[|^error:" /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -30 || true)
[ -z "$BUILD_ERRORS" ] && BUILD_ERRORS="(aucune)"

# ── Dependency Version Info ───────────────────────────────────
DEP_INFO="(non disponible)"
if [ -f "backend/Cargo.toml" ]; then
  DEP_INFO=$(grep -E "^\[dependencies\]|^\[dev-dependencies\]|^[a-z_-]+ = " backend/Cargo.toml 2>/dev/null \
    | head -30 || true)
fi

REPORT=".claude/BACKEND-BUILD-REPORT-${ARCH}.md"

cat > "$REPORT" << ENDOFMD
nd Build Report — ${ARCH} — Nook

utomatiquement par `Backend.yml` · target `$ARCH_TARGET`
DATE}**



global : ${GLOBAL_STATUS}

Valeur |
-------|
ecture** | `${ARCH}` (`$ARCH_TARGET`) |
e** | `${BRANCH}` |
** | [`${COMMIT_SHA:0:7}`](https://github.com/$REPO/commit/${COMMIT_SHA}) |
 | `${RUSTC_VER}` |
** | [Voir le run complet](${RUN_URL}) |





Statut | Détail |
-------|--------|
check** | ${CHECK_ICON} | exit ${CHECK_EXIT:-?} |
clippy** | ${CLIPPY_ICON} | exit ${CLIPPY_EXIT:-?} (-D warnings) |
build --release** | ${BUILD_ICON} | binaire ${BIN_SIZE:-N/A} stripped |



d Metrics (AI Trend Tracking)

 | Valeur |
-|--------|
e Time** | ${COMPILE_TIME}s |
 Size** | ${BIN_SIZE:-N/A} |
Warnings** | ${CHECK_WARNING_COUNT} |
Warnings** | ${BUILD_WARNING_COUNT} |
 Warnings** | ${CLIPPY_WARNING_COUNT} |
ated Warnings** | ${DEPRECATED_COUNT} |
ode Warnings** | ${DEAD_CODE_COUNT} |

 ces valeurs entre les runs pour détecter les régressions.*



tion Progression (derniers crates compilés)


PROGRESSION}



_LINE}




ppy Lints (par fréquence)


INTS}




 cargo check


RORS}




s clippy (-D warnings = fail si présents)


ARNS}


 avec fichiers et lignes


ONTEXT}




tion Warnings (par fichier)


RNINGS_FILE}




ings de déprécation


ED_WARNS}




de Warnings


E_WARNS}




 cargo build --release


RORS}




nces (Cargo.toml)


}




énéré par `.github/workflows/Backend.yml` · job `$ARCH_TARGET`*


echo "✅ ${REPORT} généré"

