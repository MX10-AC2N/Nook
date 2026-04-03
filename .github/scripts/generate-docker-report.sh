#!/usr/bin/env bash
# generate-docker-report.sh
# Génère .claude/DOCKER-BUILD-REPORT.md
# Env vars passées par le workflow:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, VERSION, IMAGE, PUSH_DIGEST,
#   AMD64_SIZE, ARM64_SIZE, FRONT_FILES, BUILD_DURATION

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

# Safe defaults
COMMIT_SHA="${COMMIT_SHA:-unknown}"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="${BRANCH:-develop}"
RUN_DATE="${RUN_DATE:-unknown}"
RUN_URL="${RUN_URL:-#}"
VERSION="${VERSION:-unknown}"
IMAGE="${IMAGE:-unknown}"
PUSH_DIGEST="${PUSH_DIGEST:-N/A}"
BUILD_DURATION="${BUILD_DURATION:-N/A}"
FRONT_FILES="${FRONT_FILES:-N/A}"
AMD64_SIZE="${AMD64_SIZE:-N/A}"
ARM64_SIZE="${ARM64_SIZE:-N/A}"

REPORT=".claude/DOCKER-BUILD-REPORT.md"
[ -n "$PUSH_DIGEST" ] && PUSH_STATUS="✅ OK" || PUSH_STATUS="❌ FAIL"

# ── Per-Platform Build Status ──────────────────────────────────
AMD64_STATUS="✅"
ARM64_STATUS="✅"
[ ! -f "docker-context/backend/nook-backend-amd64" ] && AMD64_STATUS="❌ manquant"
[ ! -f "docker-context/backend/nook-backend-arm64" ] && ARM64_STATUS="❌ manquant"

# ── Build logs ─────────────────────────────────────────────────
BUILD_LOGS="(non capturé)"
if [ -f /tmp/docker-build.txt ]; then
  BUILD_LOGS=$(tail -30 /tmp/docker-build.txt | sed 's/\x1b\[[0-9;]*m//g') || true
fi
[ -z "$BUILD_LOGS" ] && BUILD_LOGS="(non capturé)"

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
| **Digest** | \`${PUSH_DIGEST}\` |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](RUN_URL/commit/${COMMIT_SHA}) |
| **Build Time** | ${BUILD_DURATION} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Images multi-plateforme

| Platform | Taille | Statut |
|----------|--------|--------|
| linux/amd64 | ${AMD64_SIZE} | ${AMD64_STATUS} |
| linux/arm64 | ${ARM64_SIZE} | ${ARM64_STATUS} |

---

## Frontend inclus

\`\`\`
front-end files: ${FRONT_FILES}
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
exit 0
