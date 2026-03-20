#!/usr/bin/env python3
"""
scripts/fetch-gifs.py — Télécharge les GIFs curatés pour Nook
Usage  : python3 scripts/fetch-gifs.py --key YOUR_TENOR_KEY
Secrets: TENOR_API_KEY (GitHub Actions) ou --key en CLI
Output : frontend/static/gifs/*.gif + frontend/static/gifs/index.json

80 GIFs répartis en 4 catégories :
  - Réactions   (25 GIFs)
  - Fête        (20 GIFs)
  - Animaux     (20 GIFs)
  - Salutations (15 GIFs)
"""

import argparse
import json
import os
import sys
import time
import urllib.request
import urllib.parse
from pathlib import Path

# ─── Catalogue curé ───────────────────────────────────────────────────────────
# Format : (slug_fichier, terme_de_recherche, rang_dans_les_résultats)
# Le rang permet de piocher des GIFs variés plutôt que toujours le top 1.
CATALOG = {
    "reactions": [
        ("thumbs-up-1",        "thumbs up",          0),
        ("thumbs-up-2",        "thumbs up happy",    1),
        ("thumbs-down",        "thumbs down",        0),
        ("lol-1",              "laughing",           0),
        ("lol-2",              "hahaha",             1),
        ("lol-3",              "lmao",               0),
        ("love-1",             "love heart",         0),
        ("love-2",             "i love you",         1),
        ("love-3",             "heart eyes",         0),
        ("thinking",           "thinking",           0),
        ("mind-blown",         "mind blown",         0),
        ("wow",                "wow surprised",      0),
        ("no-way",             "no way",             0),
        ("yes",                "yes nodding",        0),
        ("no",                 "no shaking head",    0),
        ("angry",              "angry mad",          0),
        ("sad",                "sad crying",         0),
        ("crying-laughing",    "crying laughing",    0),
        ("clapping",           "clapping applause",  0),
        ("facepalm",           "facepalm",           0),
        ("ok",                 "ok okay",            0),
        ("perfect",            "perfect chef kiss",  0),
        ("fire",               "fire lit",           0),
        ("100",                "100 percent",        0),
        ("eyeroll",            "eye roll",           0),
    ],
    "fete": [
        ("happy-birthday-1",   "happy birthday cake",     0),
        ("happy-birthday-2",   "birthday celebration",    1),
        ("happy-birthday-3",   "birthday balloons",       0),
        ("congratulations-1",  "congratulations",         0),
        ("congratulations-2",  "congrats well done",      1),
        ("party-1",            "party celebration",       0),
        ("party-2",            "lets party",              1),
        ("champagne",          "champagne celebration",   0),
        ("fireworks",          "fireworks celebrate",     0),
        ("confetti",           "confetti party",          0),
        ("cheers",             "cheers toast",            0),
        ("woohoo",             "woohoo excited",          0),
        ("victory",            "victory winner",          0),
        ("success",            "success happy dance",     0),
        ("dancing-1",          "happy dance",             0),
        ("dancing-2",          "celebrate dance",         1),
        ("yay",                "yay excited happy",       0),
        ("great",              "great awesome",           0),
        ("amazing",            "amazing wow",             0),
        ("new-year",           "new year celebrate",      0),
    ],
    "animaux": [
        ("dog-happy-1",        "happy dog",          0),
        ("dog-happy-2",        "excited dog",        1),
        ("dog-wave",           "dog waving",         0),
        ("cat-happy-1",        "happy cat",          0),
        ("cat-happy-2",        "cute cat",           1),
        ("cat-no",             "cat no",             0),
        ("frog-cute",          "cute frog",          0),
        ("frog-wave",          "pepe wave",          0),
        ("bunny",              "cute bunny",         0),
        ("panda",              "cute panda",         0),
        ("hamster",            "cute hamster",       0),
        ("duck",               "cute duck",          0),
        ("bear-wave",          "bear waving",        0),
        ("dog-love",           "dog love heart",     0),
        ("cat-love",           "cat heart",          0),
        ("puppy-eyes",         "puppy eyes cute",    0),
        ("cat-vibing",         "cat vibing",         0),
        ("dog-yes",            "dog nodding yes",    0),
        ("penguin",            "cute penguin",       0),
        ("fox",                "cute fox",           0),
    ],
    "salutations": [
        ("hello-wave-1",       "hello wave",         0),
        ("hello-wave-2",       "hi hello",           1),
        ("bye-1",              "bye goodbye",        0),
        ("bye-2",              "see you later",      1),
        ("good-morning",       "good morning",       0),
        ("good-night",         "good night",         0),
        ("welcome",            "welcome",            0),
        ("handshake",          "handshake",          0),
        ("hug-1",              "hug",                0),
        ("hug-2",              "big hug",            1),
        ("high-five",          "high five",          0),
        ("fist-bump",          "fist bump",          0),
        ("thank-you",          "thank you thanks",   0),
        ("please",             "please begging",     0),
        ("good-luck",          "good luck",          0),
    ],
}

CATEGORY_LABELS = {
    "reactions":   "Réactions",
    "fete":        "Fête 🎉",
    "animaux":     "Animaux 🐾",
    "salutations": "Salutations",
}

# ─── Fonctions ────────────────────────────────────────────────────────────────

def tenor_search(query: str, api_key: str, pos: int = 0) -> dict | None:
    """Cherche un GIF sur Tenor et retourne le résultat à l'index `pos`."""
    params = urllib.parse.urlencode({
        "q":            query,
        "key":          api_key,
        "client_key":   "nook_fetch",
        "limit":        max(pos + 1, 3),
        "media_filter": "gif,tinygif",
        "contentfilter":"medium",
    })
    url = f"https://tenor.googleapis.com/v2/search?{params}"
    try:
        with urllib.request.urlopen(url, timeout=10) as resp:
            data = json.loads(resp.read())
            results = data.get("results", [])
            if not results:
                return None
            idx = min(pos, len(results) - 1)
            return results[idx]
    except Exception as e:
        print(f"  ⚠️  Erreur Tenor pour '{query}': {e}", file=sys.stderr)
        return None


def best_gif_url(result: dict) -> tuple[str, str]:
    """Retourne (url_preview_tinygif, url_full_gif) depuis un résultat Tenor."""
    fmt = result.get("media_formats", {})
    preview = (
        fmt.get("tinygif", {}).get("url") or
        fmt.get("gif",     {}).get("url") or ""
    )
    full = (
        fmt.get("gif",     {}).get("url") or
        fmt.get("tinygif", {}).get("url") or ""
    )
    return preview, full


def download_file(url: str, dest: Path) -> bool:
    """Télécharge un fichier vers dest. Retourne True si succès."""
    try:
        with urllib.request.urlopen(url, timeout=15) as resp:
            dest.write_bytes(resp.read())
        return True
    except Exception as e:
        print(f"  ⚠️  Téléchargement échoué {url}: {e}", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(description="Fetch curated GIFs for Nook")
    parser.add_argument("--key", default=os.environ.get("TENOR_API_KEY", ""),
                        help="Clé API Tenor (ou var TENOR_API_KEY)")
    parser.add_argument("--output", default="frontend/static/gifs",
                        help="Dossier de sortie (défaut: frontend/static/gifs)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Afficher sans télécharger")
    args = parser.parse_args()

    if not args.key:
        print("❌ TENOR_API_KEY manquante — passer --key ou définir la variable d'env",
              file=sys.stderr)
        sys.exit(1)

    out_dir = Path(args.output)
    out_dir.mkdir(parents=True, exist_ok=True)

    index = []          # catalogue final → index.json
    total = ok = 0

    for cat_key, items in CATALOG.items():
        cat_label = CATEGORY_LABELS[cat_key]
        print(f"\n{'─'*50}")
        print(f"📂 {cat_label} ({len(items)} GIFs)")
        print('─'*50)

        for slug, query, pos in items:
            total += 1
            filename = f"{cat_key}-{slug}.gif"
            dest = out_dir / filename
            print(f"  [{total:02d}] {slug:<22} ← '{query}'", end=" ")

            if args.dry_run:
                print("(dry-run)")
                index.append({"id": slug, "category": cat_key,
                               "cat_label": cat_label, "file": filename})
                ok += 1
                continue

            # Chercher sur Tenor
            result = tenor_search(query, args.key, pos)
            if not result:
                print("✗ aucun résultat")
                continue

            preview_url, full_url = best_gif_url(result)
            if not full_url:
                print("✗ URL manquante")
                continue

            # Télécharger la version tinygif (légère) pour preview ET full
            # On prend tinygif pour garder les fichiers légers (~50-200 kB)
            dl_url = preview_url or full_url
            if download_file(dl_url, dest):
                size_kb = dest.stat().st_size // 1024
                print(f"✓ {size_kb} kB")
                index.append({
                    "id":        slug,
                    "category":  cat_key,
                    "cat_label": cat_label,
                    "file":      filename,
                    "title":     result.get("title", slug),
                    "size_kb":   size_kb,
                })
                ok += 1
            else:
                print("✗ téléchargement échoué")

            # Pause pour rester dans les limites de taux (300 req/min Tenor)
            time.sleep(0.25)

    # Écrire index.json
    index_path = out_dir / "index.json"
    index_data = {
        "version":    1,
        "generated":  time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total":      ok,
        "categories": list(CATEGORY_LABELS.items()),
        "gifs":       index,
    }
    index_path.write_text(json.dumps(index_data, ensure_ascii=False, indent=2))

    print(f"\n{'═'*50}")
    print(f"✅ {ok}/{total} GIFs téléchargés → {out_dir}/")
    print(f"📄 index.json écrit ({ok} entrées)")
    if ok < total:
        print(f"⚠️  {total - ok} GIFs manquants — relancer le script pour les récupérer")
    print('═'*50)


if __name__ == "__main__":
    main()
