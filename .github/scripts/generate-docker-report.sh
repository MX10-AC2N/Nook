#!/usr/bin/env bash
# .github/scripts/generate-docker-report.sh
# Generates .claude/DOCKER-BUILD-REPORT.md
# Called by Docker.yml
#
# Env vars expected:
#   RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, VERSION, AMD64_SIZE,
#   ARM64_SIZE, FRONT_FILES, IMAGE, PUSH_DIGEST, BUILD_DURATION

set -uo pipefail
cd "${GITHUB_WORKSPACE:-.}"

REPORT=".claude/DOCKER-BUILD-REPORT.md"
COMMIT_SHORT="${COMMIT_SHA:0:7}"

[ -n "$PUSH_DIGEST" ] && PUSH_STATUS="✅ OK" || PUSH_STATUS="❌ FAIL"

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
| **Arch amd64** | ${AMD64_SIZE:-N/A} |
| **Arch arm64** | ${ARM64_SIZE:-N/A} |
| **Frontend** | ${FRONT_FILES:-N/A} fichiers |
| **Branche** | \`${BRANCH}\` |
| **Commit** | [\`${COMMIT_SHORT}\`](https://github.com/${GITHUB_REPOSITORY:-}/commit/${COMMIT_SHA}) |
| **Durée** | ${BUILD_DURATION:-N/A} |
| **Run** | [Voir le run](${RUN_URL}) |

---

## Registries

\`\`\`
Pushed: ghcr.io/${GITHUB_REPOSITORY:-}:${VERSION}
\`\`\`

---

*Rapport généré par \`.github/scripts/generate-docker-report.sh\`*
ENDOFMD

echo "✅ DOCKER-BUILD-REPORT.md généré ($(du -h $REPORT | cut -f1))"
