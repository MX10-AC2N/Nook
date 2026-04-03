#!/usr/bin/env bash
# generate-docker-report.sh
# Génère .claude/DOCKER-BUILD-REPORT.md
# Appelé par Docker.yml
# Variables d'env attendues : RUN_DATE, COMMIT_SHA, BRANCH, RUN_URL, VERSION,
#   AMD64_SIZE, ARM64_SIZE, FRONT_FILES, IMAGE, PUSH_DIGEST, BUILD_DURATION
# Fichiers lus : /tmp/docker-build.txt (optionnel)

set -euo pipefail
cd "${GITHUB_WORKSPACE:-.}"

RUN_DATE=$(date -u '+%Y-%m-%d %H:%M UTC')
COMMIT_SHA="$COMMIT_SHA"
COMMIT_SHORT="${COMMIT_SHA:0:7}"
BRANCH="$BRANCH"
RUN_URL="https://github.com/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
VERSION="$VERSION"
AMD64_SIZE="$AMD64_SIZE"
ARM64_SIZE="$ARM64_SIZE"
FRONT_FILES="$FRONT_FILES"
IMAGE="ghcr.io/$GITHUB_REPOSITORY"
PUSH_DIGEST="$PUSH_DIGEST"

[ -n "$PUSH_DIGEST" ] && PUSH_STATUS="✅ OK" || PUSH_STATUS="❌ FAIL"

# Per-Platform Build Status
AMD64_STATUS="✅"
ARM64_STATUS="✅"
[ ! -f "docker-context/backend/nook-backend-amd64" ] && AMD64_STATUS="❌ manquant"
[ ! -f "docker-context/backend/nook-backend-arm64" ] && ARM64_STATUS="❌ manquant"

# Docker Image Info
LAYER_INFO="(image non disponible — push échoué)"
if [ -n "$PUSH_DIGEST" ]; then
  LAYER_INFO="Digest: ${PUSH_DIGEST}"
  LAYER_INFO="${LAYER_INFO}\nAMD64 binary: ${AMD64_SIZE}"
  LAYER_INFO="${LAYER_INFO}\nARM64 binary: ${ARM64_SIZE}"
  LAYER_INFO="${LAYER_INFO}\nFrontend: ${FRONT_FILES} fichiers"
fi

REPORT=".claude/DOCKER-BUILD-REPORT.md"
cat > "$REPORT" << ENDOFMD
 Build Report — Nook

utomatiquement par `Docker.yml`
DATE}**





Valeur |
-------|
HCR** | ${PUSH_STATUS} |
n** | `${VERSION}` |
* | `${IMAGE}` |
** | `${PUSH_DIGEST:-N/A}` |
e** | `${BRANCH}` |
** | [`${COMMIT_SHA:0:7}`](https://github.com/$GITHUB_REPOSITORY/commit/${COMMIT_SHA}) |
| [Voir le run](${RUN_URL}) |



ts intégrés

t | Taille | Statut |
--|--------|--------|
d amd64** | ${AMD64_SIZE} | ${AMD64_STATUS} |
d arm64** | ${ARM64_SIZE} | ${ARM64_STATUS} |
nd** | ${FRONT_FILES} fichiers | ✅ |



nfo & Platform Status


FO}




bliés


meta.outputs.tags }}




rmes

 | Statut | Cible |
-|--------|-------|
md64` | ${AMD64_STATUS} | Zimaboard x86 + CI GitHub |
rm64` | ${ARM64_STATUS} | Raspberry Pi, Apple M1/M2 |



tion homeserver


 jour le homeserver Zimaboard
l ${IMAGE}:${VERSION}

l ${IMAGE}:latest




énéré par `.github/workflows/Docker.yml`*


echo "✅ DOCKER-BUILD-REPORT.md généré"
cat "$REPORT"
