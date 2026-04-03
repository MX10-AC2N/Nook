#!/usr/bin/env bash
# generate-docker-report.sh
# Génère .claude/DOCKER-BUILD-REPORT.md
# Appelé par Docker.yml
# Variables d'environnement attendues :
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, VERSION
#   AMD64_SIZE, ARM64_SIZE, FRONT_FILES, IMAGE
#   PUSH_DIGEST, BUILD_DURATION
# Fichiers lus : /tmp/docker-build.txt (optionnel)

set -euo pipefail

cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/DOCKER-BUILD-REPORT.md"
mkdir -p .claude

# Statut global du push
if [ -n "$PUSH_DIGEST" ]; then
  PUSH_STATUS="✅ OK"
else
  PUSH_STATUS="❌ FAIL"
fi

# ── Build logs (si dispo) ──────────────────────────────────────────────
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
| **Commit** | [\`${COMMIT_SHA:0:7}\`](https://github.com/${{ github.repository }}/commit/${COMMIT_SHA}) |
| **Build Time** | ${BUILD_DURATION:-N/A} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Images multi-plateforme

| Platform | Taille | Statut |
|----------|--------|--------|
| linux/amd64 | ${AMD64_SIZE:-N/A} | $([ -n "$AMD64_SIZE" ] && echo "✅" || echo "❌") |
| linux/arm64 | ${ARM64_SIZE:-N/A} | $([ -n "$ARM64_SIZE" ] && echo "✅" || echo "❌") |

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
