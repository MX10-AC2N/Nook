#!/usr/bin/env bash
# generate-backend-report.sh
# Génère .claude/BACKEND-BUILD-REPORT-{ARCH}.md
# Appelé par Backend.yml (job matrix pour amd64/arm64)
# Variables d'environnement attendues :
#   ARCH, RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, RUSTC_VER
#   BUILD_STATUS, BIN_SIZE, CHECK_EXIT, CLIPPY_EXIT, BUILD_EXIT
#   BUILD_DURATION, WARNINGS_COUNT, ERRORS_COUNT, NEW_WARNINGS
# Fichiers lus : /tmp/cargo-build.txt, /tmp/cargo-check.txt, /tmp/cargo-clippy.txt, /tmp/cargo-test.txt

set -euo pipefail

cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/BACKEND-BUILD-REPORT-${ARCH}.md"
mkdir -p .claude

[ "$BUILD_STATUS" = "OK" ] && BUILD_ICON="✅" || BUILD_ICON="❌"
[ "$CHECK_EXIT"  = "0" ]   && CHECK_ICON="✅"  || CHECK_ICON="❌"
[ "$CLIPPY_EXIT" = "0" ]   && CLIPPY_ICON="✅" || CLIPPY_ICON="❌"

# Global status
GLOBAL_STATUS="✅ OK"
if [ "$BUILD_STATUS" != "OK" ] || [ "$CHECK_EXIT" != "0" ] || [ "$CLIPPY_EXIT" != "0" ]; then
  GLOBAL_STATUS="❌ ÉCHEC"
fi

# ── Ligne Finished / Erreur ────────────────────────────────────────────
FINISHED=$(grep -E "Finished|Compiling nook-backend" /tmp/cargo-build.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | tail -5 || true)
[ -z "$FINISHED" ] && FINISHED="(non disponible)"

# ── Warnings check ─────────────────────────────────────────────────────
CHECK_WARNINGS=$(grep -E "^warning:|^warning\[" /tmp/cargo-check.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -30 || echo "(aucun)")

# ── Warnings+Erreurs clippy ─────────────────────────────────────────────
CLIPPY_OUTPUT=$(cat /tmp/cargo-clippy.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | head -50 || echo "(non disponible)")

# ── Tests output ────────────────────────────────────────────────────────
TEST_OUTPUT=$(cat /tmp/cargo-test.txt 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' | grep -E "test .* \.\.\. |running " | tail -20 || echo "(non disponible)")

cat > "$REPORT" << ENDOFMD
# 🏗️ Backend Build Report — ${ARCH} — Nook

> Généré automatiquement par \`Backend.yml\` · target \`${{ matrix.target }}\`
> **${RUN_DATE}**

---

## Statut global : ${GLOBAL_STATUS}

| Champ | Valeur |
|-------|--------|
| **Architecture** | \`${ARCH}\` (\`${{ matrix.target }}\`) |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHA:0:7}\`](https://github.com/${{ github.repository }}/commit/${COMMIT_SHA}) |
| **Rust** | \`${RUSTC_VER}\` |
| **Build** | ${BUILD_ICON} ${BUILD_STATUS:-INCONNU} |
| **cargo check** | ${CHECK_ICON} exit=${CHECK_EXIT:-?} |
| **cargo clippy** | ${CLIPPY_ICON} exit=${CLIPPY_EXIT:-?} |
| **Binary Size** | ${BIN_SIZE:-N/A} |
| **Build Time** | ${BUILD_DURATION:-N/A} |
| **Warnings** | ${WARNINGS_COUNT:-N/A} |
| **Errors** | ${ERRORS_COUNT:-N/A} |
| **New Warnings** | ${NEW_WARNINGS:-N/A} |
| **Run CI** | [Voir le run complet](${RUN_URL}) |

---

## Compilation (fin)

\`\`\`
${FINISHED}
\`\`\`

---

## cargo check — Warnings

\`\`\`
${CHECK_WARNINGS}
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
