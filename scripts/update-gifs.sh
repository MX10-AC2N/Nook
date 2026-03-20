#!/usr/bin/env bash
# scripts/update-gifs.sh — Mise à jour hebdomadaire des GIFs Nook
#
# Usage  : ./scripts/update-gifs.sh
# Config : GIPHY_API_KEY et GIFS_DIR dans le .env ou en variables d'env
#
# Prérequis sur la Zimaboard : curl, jq (sudo apt install jq)
#
# Crontab recommandé (lundi 3h du matin) :
#   0 3 * * 1 cd /chemin/vers/Nook && bash scripts/update-gifs.sh >> logs/gifs-update.log 2>&1
#
# Les GIFs sont stockés dans GIFS_DIR (/app/data/gifs/ par défaut dans Docker).
# Nook les sert directement depuis ce dossier sans rebuild de l'image.

set -euo pipefail

# ── Charger le .env si présent ────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
if [ -f "$ROOT_DIR/.env" ]; then
  set -a
  # shellcheck disable=SC1091
  source "$ROOT_DIR/.env"
  set +a
fi

# ── Variables ─────────────────────────────────────────────────────────────────
GIPHY_API_KEY="${GIPHY_API_KEY:-}"
GIFS_DIR="${GIFS_DIR:-/app/data/gifs}"
RATING="g"          # g = tous publics
LANG="fr"
LIMIT=12            # 12 GIFs par thème
TIMEOUT=15          # secondes par requête curl
DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# ── Vérifications ─────────────────────────────────────────────────────────────
if [ -z "$GIPHY_API_KEY" ]; then
  echo "❌ GIPHY_API_KEY manquante dans .env"
  echo "   → Créer une clé gratuite sur https://developers.giphy.com → Create App → SDK"
  exit 1
fi

if ! command -v jq &>/dev/null; then
  echo "❌ jq non installé — sudo apt install jq"
  exit 1
fi

if ! command -v curl &>/dev/null; then
  echo "❌ curl non installé"
  exit 1
fi

mkdir -p "$GIFS_DIR"
echo "═══════════════════════════════════════════════════"
echo "🎬 Mise à jour GIFs Nook — $DATE"
echo "   Dossier : $GIFS_DIR"
echo "   Limite  : $LIMIT GIFs × thème"
echo "═══════════════════════════════════════════════════"

# ── 12 thèmes Giphy les plus populaires ───────────────────────────────────────
declare -A THEMES=(
  ["reactions"]="thumbs up reaction"
  ["lol"]="laughing funny"
  ["love"]="love heart"
  ["celebration"]="celebration party"
  ["birthday"]="happy birthday"
  ["animals"]="cute animals"
  ["hello"]="hello wave"
  ["bye"]="goodbye wave"
  ["yes"]="yes nodding"
  ["no"]="no shaking head"
  ["wow"]="wow surprised"
  ["facepalm"]="facepalm"
)

THEME_LABELS=(
  "reactions:Réactions"
  "lol:😂 Humour"
  "love:❤️ Amour"
  "celebration:🎉 Fête"
  "birthday:🎂 Anniversaire"
  "animals:🐾 Animaux"
  "hello:👋 Bonjour"
  "bye:👋 Au revoir"
  "yes:✅ Oui"
  "no:❌ Non"
  "wow:😮 Wow"
  "facepalm:🤦 Facepalm"
)

# ── Téléchargement ─────────────────────────────────────────────────────────────
TOTAL=0
OK=0
INDEX_ENTRIES="[]"

for LABEL_PAIR in "${THEME_LABELS[@]}"; do
  CAT_KEY="${LABEL_PAIR%%:*}"
  CAT_LABEL="${LABEL_PAIR#*:}"
  QUERY="${THEMES[$CAT_KEY]}"

  echo ""
  echo "── $CAT_LABEL ($QUERY) ──"

  # Appel Giphy
  RESPONSE=$(curl -sf --max-time "$TIMEOUT" \
    "https://api.giphy.com/v1/gifs/search?api_key=${GIPHY_API_KEY}&q=$(python3 -c "import urllib.parse; print(urllib.parse.quote('${QUERY}'))" 2>/dev/null || echo "${QUERY// /+}")&limit=${LIMIT}&rating=${RATING}&lang=${LANG}" \
    || echo '{}')

  GIF_COUNT=$(echo "$RESPONSE" | jq '.data | length' 2>/dev/null || echo 0)

  if [ "$GIF_COUNT" -eq 0 ]; then
    echo "  ⚠️  Aucun résultat (vérifier GIPHY_API_KEY)"
    continue
  fi

  # Télécharger chaque GIF
  for i in $(seq 0 $((GIF_COUNT - 1))); do
    TOTAL=$((TOTAL + 1))

    GIF_ID=$(echo "$RESPONSE" | jq -r ".data[$i].id")
    GIF_TITLE=$(echo "$RESPONSE" | jq -r ".data[$i].title" | tr '/' '-' | tr ' ' '_' | head -c 40)
    # Préférer fixed_width (légère ~100-200kB), sinon original
    GIF_URL=$(echo "$RESPONSE" | jq -r \
      ".data[$i].images.fixed_width.url // .data[$i].images.original.url // empty")

    if [ -z "$GIF_URL" ]; then
      echo "  [$i] ✗ URL manquante"
      continue
    fi

    FILENAME="${CAT_KEY}-${i}-${GIF_ID:0:8}.gif"
    DEST="$GIFS_DIR/$FILENAME"

    # Télécharger (écrase si déjà présent — mise à jour)
    if curl -sf --max-time "$TIMEOUT" -o "$DEST" "$GIF_URL" 2>/dev/null; then
      SIZE_KB=$(( $(wc -c < "$DEST") / 1024 ))
      echo "  [$((i+1))/$GIF_COUNT] ✓ $FILENAME ($SIZE_KB kB)"
      OK=$((OK + 1))

      # Ajouter à l'index
      ENTRY=$(jq -n \
        --arg id "$GIF_ID" \
        --arg cat "$CAT_KEY" \
        --arg label "$CAT_LABEL" \
        --arg file "$FILENAME" \
        --arg title "$GIF_TITLE" \
        --argjson size "$SIZE_KB" \
        '{id:$id, category:$cat, cat_label:$label, file:$file, title:$title, size_kb:$size}')
      INDEX_ENTRIES=$(echo "$INDEX_ENTRIES" | jq ". + [$ENTRY]")
    else
      echo "  [$((i+1))/$GIF_COUNT] ✗ Téléchargement échoué"
    fi

    sleep 0.1  # rester dans les limites de taux Giphy
  done
done

# ── Générer index.json ─────────────────────────────────────────────────────────
INDEX_PATH="$GIFS_DIR/index.json"

# Construire les catégories à partir des labels
CATS_JSON="[]"
for LABEL_PAIR in "${THEME_LABELS[@]}"; do
  CAT_KEY="${LABEL_PAIR%%:*}"
  CAT_LABEL="${LABEL_PAIR#*:}"
  CATS_JSON=$(echo "$CATS_JSON" | jq ". + [[$CAT_KEY, $CAT_LABEL]]" \
    --arg k "$CAT_KEY" --arg l "$CAT_LABEL" 2>/dev/null \
    || echo "$CATS_JSON")
done

jq -n \
  --arg version "1" \
  --arg generated "$DATE" \
  --argjson total "$OK" \
  --argjson gifs "$INDEX_ENTRIES" \
  '{version:1, generated:$generated, total:$total, categories:[
    ["reactions","Réactions"],["lol","😂 Humour"],["love","❤️ Amour"],
    ["celebration","🎉 Fête"],["birthday","🎂 Anniversaire"],["animals","🐾 Animaux"],
    ["hello","👋 Bonjour"],["bye","👋 Au revoir"],["yes","✅ Oui"],
    ["no","❌ Non"],["wow","😮 Wow"],["facepalm","🤦 Facepalm"]
  ], gifs:$gifs}' > "$INDEX_PATH"

# ── Résumé ─────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════"
echo "✅ $OK/$TOTAL GIFs mis à jour → $GIFS_DIR"
echo "📄 index.json régénéré ($OK entrées)"
TOTAL_SIZE=$(du -sh "$GIFS_DIR"/*.gif 2>/dev/null | tail -1 | cut -f1 || echo "?")
echo "💾 Taille totale : $(du -sh "$GIFS_DIR" 2>/dev/null | cut -f1 || echo "?")"
echo "═══════════════════════════════════════════════════"
