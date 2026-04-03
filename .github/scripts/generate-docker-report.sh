#!/usr/bin/env bash
# generate-docker-report.sh
# Génère .claude/DOCKER-BUILD-REPORT.md
# Env vars passées par le workflow:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, VERSION, IMAGE, PUSH_DIGEST,
#   AMD64_SIZE, ARM64_SIZE, FRONT_FILES, BUILD_DURATION

set -euo pipefail
cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/DOCKER-BUILD-REPORT.md"
COMMIT_SHORT="${COMMIT_SHA:0:7}"

[ -n "$PUSH_DIGEST" ] && PUSH_STATUS="✅ OK" || PUSH_STATUS="❌ FAIL"

# ── Per-Platform Build Status ──────────────────────────────────
AMD64_STATUS="✅"
ARM64_STATUS="✅"
[ ! -f "docker-context/backend/nook-backend-amd64" ] && AMD64_STATUS="❌ manquant"
[ ! -f "docker-context/backend/nook-backend-arm64" ] && ARM64_STATUS="❌ manquant"

# ── Build logs ─────────────────────────────────────────────────
BUILD_LOGS="(non capturé)"
if [ -f /tmp/docker-build.txt ]; then
  BUILD_LOGS=$(tail -30 /tmp/docker-build.txt | sed 's/\x1b\[[0-9;]*m//g')
fi

cat > "$REPORT" << ENDOFMD
# 🐳 Docker Build Report — Nook

> Généré automatiquement par \`Docker.yml\`
> **${RUN_DATE}**

---

## Statut

| Champ | Valeur |
|-------|--------|
| **Push GHCR** | ${PUSH_STATUS} |
| **Version** | \`${VERSION}\` |
| **Image** | \`${IMAGE}\` |
| **Digest** | \`${PUSH_DIGEST:-N/A}\` |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
| **Build Time** | ${BUILD_DURATION:-N/A} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Images multi-plateforme

| Platform | Taille | Statut |
|----------|--------|--------|
| linux/amd64 | ${AMD64_SIZE:-N/A} | ${AMD64_STATUS} |
| linux/arm64 | ${ARM64_SIZE:-N/A} | ${ARM64_STATUS} |

---

## Frontend inclus

\`\`\`
front-end files: ${FRONT_FILES:-N/A}
\`\`\`

---

## Build logs (fin)

\`\`\`
${BUILD_LOGS}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-docker-report.sh\`*
ENDOFMD

echo "✅ DOCKER-BUILD-REPORT.md généré"
