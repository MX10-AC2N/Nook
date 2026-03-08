#!/usr/bin/env python3
"""
Génère l'instruction personnalisée pour l'application Android Claude.ai.
Appelé par le workflow GitHub Actions.
"""

import os
import sys
import logging
from pathlib import Path
from typing import List, Tuple

# ------------------------------------------------------------
# Configuration du logger (facultatif mais très utile)
# ------------------------------------------------------------
logging.basicConfig(
    level=logging.INFO,
    format="%(levelname)s – %(message)s",
)

# ------------------------------------------------------------
# Mapping ordonné des agents (fichier → emoji)
# ------------------------------------------------------------
AGENT_MAPPINGS: List[Tuple[str, str]] = [
    ("rust-backend.md", "🦀RUST"),
    ("svelte-frontend.md", "🎨SVELTE"),
    ("ci-devops.md", "🚀DEVOPS"),
    ("e2e-testing.md", "🧪E2E"),
    ("security-crypto.md", "🔐CRYPTO"),
    ("chess-engine.md", "♟CHESS"),
    ("data-analytics.md", "📊DATA"),
    ("architect.md", "📐ARCHITECT"),
    ("delegate.md", "🤖DELEGATE"),
]


def _detect_roles_dir() -> Path:
    """
    Retourne le chemin absolu du répertoire ``.claude/roles``.
    Le calcul est résilient même si le script est lancé depuis un sous‑répertoire.
    """
    script_path = Path(__file__).resolve()
    project_root = script_path.parent.parent.parent.parent  # ../../../..
    roles_dir = project_root / ".claude" / "roles"
    if not roles_dir.is_dir():
        raise FileNotFoundError(f"Répertoire rôles introuvable : {roles_dir}")
    logging.info(f"Roles directory detected: {roles_dir}")
    return roles_dir


def _list_md_files(roles_dir: Path) -> set[str]:
    """Renvoie l’ensemble des noms de fichiers *.md* présents dans ``roles_dir``."""
    md_files = {p.name for p in roles_dir.iterdir() if p.suffix.lower() == ".md"}
    logging.debug(f"Fichiers .md détectés : {sorted(md_files)}")
    return md_files


def _build_available_agents(md_files: set[str]) -> str:
    """
    Construit la chaîne affichée dans l’instruction.
    Seuls les agents dont le fichier existe sont conservés, dans l’ordre du mapping.
    """
    agents = [
        emoji
        for filename, emoji in AGENT_MAPPINGS
        if filename in md_files
    ]
    if not agents:
        logging.warning("Aucun agent trouvé ! Vérifiez les fichiers .md dans .claude/roles.")
    return " | ".join(agents)


def generate_instruction(
    version: str, session: str, date: str, active_bugs: str
) -> Tuple[str, int]:
    """
    Génère le texte d’instruction et renvoie (instruction, nombre_de_caractères).
    """
    roles_dir = _detect_roles_dir()
    md_files = _list_md_files(roles_dir)
    agents_str = _build_available_agents(md_files)

    instruction = f"""Tu es l'assistant principal du projet Nook (v{version}, session {session}).

Messagerie familiale self‑hosted — Rust/Axum 0.8 + SvelteKit 5 Runes + SQLite + Docker distroless.

Repo: https://github.com/MX10-AC2N/Nook | Branche: main
Raw: https://raw.githubusercontent.com/MX10-AC2N/Nook/main/

AVANT CHAQUE INTERVENTION:
1. Fetcher .claude/BUGS.md ({active_bugs} bugs actifs)
2. Fetcher .claude/rules/memory‑sessions.md
3. Fetcher les fichiers sources concernés (jamais travailler de mémoire)

AGENTS DISPONIBLES (fichiers dans .claude/roles/):
{agents_str}

RÈGLES ABSOLUES:
• Fichier complet — jamais de diff partiel
• .svelte/.ts → livrer en .txt
• Chemin exact en tête de chaque bloc de code
• Signaler les effets de bord inter‑agents
• Clôture: mettre à jour BUGS.md + SESSIONS.md

Pièges critiques: rand::rng() (pas thread_rng) | routes {{param}} axum 0.8 |
$state Svelte 5 via Object.assign | CORS + credentials → origins explicites |
sqlx sans macros si queries.json vide"""
    return instruction, len(instruction)


def generate_markdown(
    instruction: str,
    char_count: int,
    date: str,
    version: str,
    session: str,
    active_bugs: str,
) -> str:
    """Construit le fichier Markdown final."""
    status = (
        "✅ OK"
        if char_count <= 1500
        else f"⚠️ TROP LONG ({char_count} chars)"
    )
    return f"""# 📱 Instruction personnalisée Android — Nook

> Générée le : **{date}** | Version : **{version}** | Session : **{session}**

> Taille : **{char_count} / 1500 chars** {status}
---
## 📋 Instruction à copier dans Claude.ai Android

> Paramètres → Instructions personnalisées → coller le texte ci‑dessous

{instruction}
---
## 🔄 Mise à jour

Ce fichier est **auto‑généré** par le workflow `generate-android-instruction.yml`.
Il se met à jour automatiquement quand `VERSION`, `BUGS.md` ou `CLAUDE.md` changent.

Pour forcer une régénération : lancer le workflow manuellement depuis GitHub Actions.
---
## 📊 Statistiques

| | |
|---|---|
| Taille instruction | {char_count} chars / 1500 max |
| Bugs actifs | {active_bugs} |
| Version projet | {version} |
| Session | {session} |
"""


def main() -> None:
    """Entrée du script."""
    if len(sys.argv) != 5:
        sys.stderr.write(
            f"Usage: {sys.argv[0]} <version> <session> <date> <active_bugs>\n"
        )
        sys.exit(1)

    version, session, date, active_bugs = sys.argv[1:5]

    instruction, char_count = generate_instruction(
        version, session, date, active_bugs
    )
    markdown_content = generate_markdown(
        instruction, char_count, date, version, session, active_bugs
    )

    output_path = Path(".claude") / "ANDROID-INSTRUCTION.md"
    output_path.parent.mkdir(parents=True, exist_ok=True)

    output_path.write_text(markdown_content, encoding="utf-8")
    logging.info(f"Instruction générée : {char_count} chars → {output_path}")

    if char_count > 1500:
        logging.error("Taille dépasse la limite de 1500 chars.")
        sys.exit(1)


if __name__ == "__main__":
    main()