#!/usr/bin/env bash
# =====================================================================
#  deploy-guard.sh — GARDE-FOU DE DÉPLOIEMENT
#
#  Vérifie que le bundle effectivement SERVI par une instance déployée
#  correspond bien au commit attendu.
#
#  Pourquoi ce script existe (incident réel du 2026-10-22) :
#    L'instance 192.168.1.192:6300 servait une image construite AVANT
#    HEAD. Le bundle servi était 0.CrOWh7b4.css (1 occurrence de
#    « alert-bg ») alors que le bundle du commit 8076eb2a est
#    0.DyZ9Wpm1.css (4 occurrences). Personne ne pouvait le détecter :
#    aucun workflow ne comparait le bundle servi au bundle attendu.
#
#  Le hash du bundle est un signal discriminant : il change dès qu'un
#  asset CSS change. On compare donc :
#    - bundle ATTENDU : provenant de l'artifact frontend du commit attendu
#                       (ou reconstruit depuis les sources du commit)
#    - bundle SERVI   : récupéré depuis l'URL de déploiement
#
#  Sortie : exit 0 si conforme, exit 1 (rouge) sinon, avec les DEUX
#  hashs dans le message d'erreur.
#
#  Usage :
#    DEPLOY_URL=http://192.168.1.192:6300 \
#    EXPECTED_SHA=8076eb2a... \
#    GITHUB_TOKEN=ghp_xxx \
#    ./scripts/deploy-guard.sh
#
#  Variables :
#    DEPLOY_URL       URL de l'instance déployée (défaut http://192.168.1.192:6300)
#    DEPLOY_IMAGE     image du conteneur (ex. ghcr.io/mx10-ac2n/nook:dev).
#                     Repli utilisé quand DEPLOY_URL est injoignable — cas
#                     d'un déploiement LAN vérifié depuis un runner CI
#                     GitHub (le runner public ne route pas vers le LAN).
#                     Vérifie alors le bundle contenu dans l'IMAGE.
#    EXPECTED_SHA     commit attendu (défaut : HEAD du dépôt cloné)
#    EXPECTED_DIR     répertoire contenant le build frontend attendu
#                     (index.html à la racine). Si absent, on télécharge
#                     l'artifact « nook-frontend » du run Frontend.yml
#                     correspondant à EXPECTED_SHA.
#    WORKFLOW_RUN_ID  optionnel : force le run Frontend.yml à utiliser
#    GITHUB_TOKEN     token pour l'API GitHub (artifacts)
#    GITHUB_REPOSITORY        défaut MX10-AC2N/Nook
#    FRONTEND_WORKFLOW        défaut « Frontend.yml »
#    SKIP_ARTIFACT    si "1", ne pas tenter le téléchargement d'artifact
#    FALLBACK_BUILD   si "1", rebuild le frontend si l'artifact a expiré
# =====================================================================
set -uo pipefail

DEPLOY_URL="${DEPLOY_URL:-http://192.168.1.192:6300}"
DEPLOY_IMAGE="${DEPLOY_IMAGE:-}"
EXPECTED_SHA="${EXPECTED_SHA:-}"
EXPECTED_DIR="${EXPECTED_DIR:-}"
WORKFLOW_RUN_ID="${WORKFLOW_RUN_ID:-}"
GITHUB_TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-MX10-AC2N/Nook}"
FRONTEND_WORKFLOW="${FRONTEND_WORKFLOW:-Frontend.yml}"
EXPECTED_BRANCH="${EXPECTED_BRANCH:-develop}"
SKIP_ARTIFACT="${SKIP_ARTIFACT:-0}"
FALLBACK_BUILD="${FALLBACK_BUILD:-0}"

RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; BOLD=$'\033[1m'; RESET=$'\033[0m'

fail() { echo "${RED}${BOLD}❌ DEPLOY GUARD — ÉCHEC${RESET}"; shift 0; for l in "$@"; do echo "   $l"; done; exit 1; }
info() { echo "   $1"; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/deploy-guard.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------------
# 1. Déterminer le SHA attendu
# ---------------------------------------------------------------------
if [ -z "$EXPECTED_SHA" ]; then
  if git rev-parse --short HEAD >/dev/null 2>&1; then
    EXPECTED_SHA="$(git rev-parse HEAD)"
  else
    fail "EXPECTED_SHA non fourni et aucun dépôt git ici — impossible de déterminer le commit attendu."
  fi
fi
EXPECTED_SHA_FULL="$EXPECTED_SHA"
info "commit attendu : $EXPECTED_SHA_FULL"

# ---------------------------------------------------------------------
# 2. Déterminer le bundle ATTENDU
# ---------------------------------------------------------------------
EXPECTED_INDEX=""

if [ -n "$EXPECTED_DIR" ] && [ -f "$EXPECTED_DIR/index.html" ]; then
  info "bundle attendu : répertoire fourni ($EXPECTED_DIR)"
  EXPECTED_INDEX="$EXPECTED_DIR/index.html"
else
  # 2a. Trouver le run Frontend.yml du SHA attendu
  if [ -n "$GITHUB_TOKEN" ]; then
    AUTH=(-H "Authorization: Bearer ${GITHUB_TOKEN}" -H "Accept: application/vnd.github+json")
    if [ -n "$WORKFLOW_RUN_ID" ]; then
      RUN_ID="$WORKFLOW_RUN_ID"
    else
      info "recherche du run ${FRONTEND_WORKFLOW} pour ${EXPECTED_SHA_FULL} ..."
      RUN_ID="$(curl -fsS "${AUTH[@]}" \
        "https://api.github.com/repos/${GITHUB_REPOSITORY}/actions/workflows/${FRONTEND_WORKFLOW}/runs?per_page=100&branch=${EXPECTED_BRANCH}" 2>/dev/null \
        | python3 -c '
import json,sys
sha=sys.argv[1]
try:
    d=json.load(sys.stdin)
except Exception:
    sys.exit(0)
for r in d.get("workflow_runs",[]):
    if (r.get("head_sha") or "").startswith(sha):
        print(r["id"]); break
' "$EXPECTED_SHA_FULL" 2>/dev/null || true)"
    fi
  else
    info "GITHUB_TOKEN absent — saut du téléchargement d'artifact."
    RUN_ID=""
  fi

  # 2b. Télécharger l'artifact « nook-frontend » de ce run
  ARTIFACT_DIR=""
  if [ -n "${RUN_ID:-}" ]; then
    info "run Frontend.yml : $RUN_ID — téléchargement de l'artifact nook-frontend ..."
    ART_ID="$(curl -fsS "${AUTH[@]}" \
      "https://api.github.com/repos/${GITHUB_REPOSITORY}/actions/runs/${RUN_ID}/artifacts" 2>/dev/null \
      | python3 -c '
import json,sys
d=json.load(sys.stdin)
for a in d.get("artifacts",[]):
    if a.get("name")=="nook-frontend" and not a.get("expired"):
        print(a["id"]); break
' 2>/dev/null || true)"
    if [ -n "$ART_ID" ]; then
      if curl -fsSL "${AUTH[@]}" -o "$WORK/fe.zip" \
        "https://api.github.com/repos/${GITHUB_REPOSITORY}/actions/artifacts/${ART_ID}/zip" 2>/dev/null; then
        if python3 -c "
import zipfile,sys
z=zipfile.ZipFile('$WORK/fe.zip')
z.extractall('$WORK/expected')
" 2>/dev/null; then
          ARTIFACT_DIR="$WORK/expected"
          [ -f "$ARTIFACT_DIR/index.html" ] && EXPECTED_INDEX="$ARTIFACT_DIR/index.html"
        fi
      fi
    fi
    [ -n "$EXPECTED_INDEX" ] || info "artifact introuvable/expiré (rétention 7 j) pour le run $RUN_ID."
  fi
fi

# ---------------------------------------------------------------------
# 2c. Repli : reconstruire le frontend depuis le commit attendu
#     (l'artifact n'est conservé que 7 jours — au-delà, on rebuild)
# ---------------------------------------------------------------------
if [ -z "$EXPECTED_INDEX" ] && [ "$FALLBACK_BUILD" = "1" ]; then
  info "repli : reconstruction du frontend depuis $EXPECTED_SHA_FULL ..."
  if command -v npm >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    if git clone --quiet --no-checkout . "$WORK/src" 2>/dev/null \
       && git -C "$WORK/src" checkout --quiet "$EXPECTED_SHA_FULL" 2>/dev/null; then
      if ( cd "$WORK/src/frontend" \
             && npm ci --no-audit --no-fund --legacy-peer-deps >/dev/null 2>&1 \
             && npm run build >/dev/null 2>&1 ) \
         && [ -f "$WORK/src/frontend/build/index.html" ]; then
        EXPECTED_DIR="$WORK/src/frontend/build"
        EXPECTED_INDEX="$EXPECTED_DIR/index.html"
        info "repli OK : bundle reconstruit depuis $EXPECTED_SHA_FULL"
      else
        info "repli KO : le build frontend a échoué."
      fi
    else
      info "repli KO : commit $EXPECTED_SHA_FULL introuvable dans ce dépôt."
    fi
  else
    info "repli KO : npm absent — installez Node pour activer le repli."
  fi
fi

# ---------------------------------------------------------------------
# 3. Extraire le bundle attendu
# ---------------------------------------------------------------------
extract_css() {
  # $1 = fichier html → liste triée des noms de fichiers CSS
  grep -oE '(/_app/immutable/)?assets/[A-Za-z0-9._-]+\.css' "$1" 2>/dev/null \
    | sed -E 's|.*/||' | sort -u
}

if [ -z "$EXPECTED_INDEX" ]; then
  fail "Aucun bundle ATTENDU disponible (artifact introuvable, repli désactivé). Impossible de vérifier — le garde-fou ne passe PAS par silence."
fi

mapfile -t EXPECTED_CSS < <(extract_css "$EXPECTED_INDEX")
[ ${#EXPECTED_CSS[@]} -gt 0 ] || fail "Aucune référence CSS trouvée dans le bundle attendu ($EXPECTED_INDEX)."

# sha256 du (premier) CSS attendu
EXPECTED_CSS_MAIN="${EXPECTED_CSS[0]}"
EXPECTED_CSS_PATH="$([ -n "$EXPECTED_DIR" ] && echo "$EXPECTED_DIR" || echo "$ARTIFACT_DIR")/_app/immutable/assets/$EXPECTED_CSS_MAIN"
EXPECTED_CSS_SHA=""
if [ -f "$EXPECTED_CSS_PATH" ]; then
  EXPECTED_CSS_SHA="$(sha256sum "$EXPECTED_CSS_PATH" | cut -d' ' -f1)"
fi
info "bundle attendu : ${EXPECTED_CSS[*]}"
[ -n "$EXPECTED_CSS_SHA" ] && info "sha256 attendu : $EXPECTED_CSS_SHA"

# ---------------------------------------------------------------------
# 4. Récupérer le bundle SERVI
#    1) l'instance (DEPLOY_URL)   — mesure réelle du conteneur en cours
#    2) repli : l'image (DEPLOY_IMAGE) — quand l'instance est injoignable
#       (déploiement LAN vérifié depuis un runner CI public)
# ---------------------------------------------------------------------
SERVED_SOURCE=""
SERVED_INDEX_RAW=""

if [ -n "$DEPLOY_URL" ]; then
  info "instance : $DEPLOY_URL"
  SERVED_INDEX_RAW="$WORK/served-index.html"
  if curl -fsS -m 30 "$DEPLOY_URL/" -o "$SERVED_INDEX_RAW" 2>/dev/null \
     && grep -q "/_app/immutable/" "$SERVED_INDEX_RAW"; then
    SERVED_SOURCE="instance"
  else
    info "instance injoignable ou réponse non Nook ($DEPLOY_URL)."
  fi
fi

# --- Repli image (GHCR) -------------------------------------------------
fetch_image_static() {
  # $1 = image (ex. ghcr.io/mx10-ac2n/nook:dev), $2 = outdir
  # stdout : chemin de l'index.html extrait (exit 0), sinon IMGERR… (exit 3)
  python3 - "$1" "$2" <<'PY'
import io, json, os, sys, tarfile, urllib.request, warnings
warnings.filterwarnings("ignore")
ref, outdir = sys.argv[1], sys.argv[2]
reg, _, rest = ref.partition("/")
repo, _, tag = rest.rpartition(":")
if not tag or tag == rest:
    repo, tag = rest, "latest"

def get(url, hdr=None, raw=False):
    r = urllib.request.urlopen(urllib.request.Request(url, headers=hdr or {}), timeout=180)
    return r.read() if raw else r

try:
    tok = json.load(get(f"https://{reg}/token?scope=repository:{repo}:pull&service={reg}"))["token"]
except Exception as e:
    print("IMGERR token:", e); sys.exit(3)
AH = {"Authorization": f"Bearer {tok}"}
ACC = ("application/vnd.oci.image.index.v1+json,"
       "application/vnd.docker.distribution.manifest.list.v2+json,"
       "application/vnd.docker.distribution.manifest.v2+json,"
       "application/vnd.oci.image.manifest.v1+json")
try:
    ml = json.load(get(f"https://{reg}/v2/{repo}/manifests/{tag}", {**AH, "Accept": ACC}))
except Exception as e:
    print("IMGERR manifest:", e); sys.exit(3)

man = ml
if "manifests" in ml:
    amd = next((m["digest"] for m in ml["manifests"]
                if m.get("platform", {}).get("architecture") == "amd64"
                and m.get("platform", {}).get("os") == "linux"), None)
    if amd is None and ml["manifests"]:
        amd = ml["manifests"][0]["digest"]
    if amd:
        man = json.load(get(f"https://{reg}/v2/{repo}/manifests/{amd}", {**AH, "Accept": ACC}))

os.makedirs(outdir, exist_ok=True)
for layer in man.get("layers", []):
    try:
        data = get(f"https://{reg}/v2/{repo}/blobs/{layer['digest']}", AH, raw=True)
        tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:*")
    except Exception:
        continue
    names = tf.getnames()
    if any(n == "app/static/index.html" or n.endswith("/app/static/index.html") for n in names):
        for n in names:
            if n.startswith("app/static/"):
                tf.extract(n, outdir)
        tf.close()
        idx = os.path.join(outdir, "app/static/index.html")
        if os.path.exists(idx):
            print(idx); sys.exit(0)
    tf.close()
print("IMGERR: app/static/index.html absent de toutes les couches"); sys.exit(3)
PY
}

if [ -z "$SERVED_SOURCE" ] && [ -n "$DEPLOY_IMAGE" ]; then
  info "repli image : $DEPLOY_IMAGE"
  IMG_OUT="$(PYTHONWARNINGS=ignore fetch_image_static "$DEPLOY_IMAGE" "$WORK/image" 2>/dev/null | tail -n 1)" || true
  case "$IMG_OUT" in
    IMGERR*)
      info "image illisible : $IMG_OUT"
      ;;
    */index.html)
      SERVED_INDEX_RAW="$IMG_OUT"
      SERVED_SOURCE="image"
      info "image : bundle extrait depuis $DEPLOY_IMAGE"
      ;;
  esac
fi

if [ -z "$SERVED_SOURCE" ]; then
  fail "Ni l'instance ($DEPLOY_URL) ni l'image (${DEPLOY_IMAGE:-non définie}) n'a pu être mesurée — le garde-fou ne passe PAS par silence."
fi

mapfile -t SERVED_CSS < <(extract_css "$SERVED_INDEX_RAW")
[ ${#SERVED_CSS[@]} -gt 0 ] || fail "Aucune référence CSS trouvée dans le bundle servi ($SERVED_SOURCE)."
info "bundle servi   : ${SERVED_CSS[*]}"

SERVED_CSS_MAIN="${SERVED_CSS[0]}"
if [ "$SERVED_SOURCE" = "instance" ]; then
  SERVED_CSS_FILE="$WORK/served.css"
  if ! curl -fsS -m 30 "$DEPLOY_URL/_app/immutable/assets/$SERVED_CSS_MAIN" -o "$SERVED_CSS_FILE" 2>/dev/null; then
    fail "Le CSS servi référencé est introuvable : /_app/immutable/assets/$SERVED_CSS_MAIN (404 sur $DEPLOY_URL)."
  fi
else
  SERVED_CSS_FILE="$WORK/image/app/static/_app/immutable/assets/$SERVED_CSS_MAIN"
  [ -f "$SERVED_CSS_FILE" ] || fail "Le CSS est absent de l'image : _app/immutable/assets/$SERVED_CSS_MAIN."
fi
SERVED_CSS_SHA="$(sha256sum "$SERVED_CSS_FILE" | cut -d' ' -f1)"
info "sha256 servi   : $SERVED_CSS_SHA"

# ---------------------------------------------------------------------
# 5. Comparaison
# ---------------------------------------------------------------------
if [ "${EXPECTED_CSS[*]}" = "${SERVED_CSS[*]}" ] && [ -n "$EXPECTED_CSS_SHA" ] && [ "$EXPECTED_CSS_SHA" = "$SERVED_CSS_SHA" ]; then
  echo ""
  echo "${GREEN}${BOLD}✅ DEPLOY GUARD — OK${RESET}"
  echo "   commit attendu : $EXPECTED_SHA_FULL"
  echo "   bundle servi   : ${SERVED_CSS[*]} (sha256 $SERVED_CSS_SHA)"
  echo "   source         : $SERVED_SOURCE"
  echo "   conforme au bundle du commit attendu."
  exit 0
fi

echo ""
echo "${RED}${BOLD}❌ DEPLOY GUARD — LE BUNDLE SERVI NE CORRESPOND PAS AU COMMIT ATTENDU${RESET}"
echo ""
echo "   commit attendu : $EXPECTED_SHA_FULL"
echo "   bundle attendu : ${EXPECTED_CSS[*]}   (sha256 ${EXPECTED_CSS_SHA:-incalculable})"
echo "   bundle servi   : ${SERVED_CSS[*]}   (sha256 $SERVED_CSS_SHA)"
echo "   source         : $SERVED_SOURCE"
[ "$SERVED_SOURCE" = "instance" ] && echo "   instance       : $DEPLOY_URL"
[ "$SERVED_SOURCE" = "image" ] && echo "   image          : $DEPLOY_IMAGE"
echo ""
if [ "$SERVED_SOURCE" = "image" ]; then
  echo "   → L'image déployée ne contient pas le bundle du commit attendu."
else
  echo "   → Le conteneur servi par l'instance n'a pas été recréé avec l'image du commit attendu."
fi
echo "   → Redéploiement nécessaire : reconstruire l'image puis recréer le conteneur."
exit 1
