#!/usr/bin/env python3
"""
Génère l'instruction personnalisée Android Claude.ai
→ Analyse complète du repo + .claude/roles/ à chaque exécution
"""

import sys
import pathlib


def get_agents_list() -> str:
    """Liste dynamique des agents depuis .claude/roles/"""
    roles_dir = pathlib.Path(".claude/roles")
    if not roles_dir.exists():
        return "Aucun agent détecté"

    emoji_map = {
        "rust-backend": "🦀", "svelte-frontend": "🎨", "ci-devops": "🚀",
        "e2e-testing": "🧪", "security-crypto": "🔐", "chess-engine": "♟️",
        "data-analytics": "📊", "architect": "📐", "delegate": "🤖",
        "founder": "🏠", "reviewer": "🔎", "security-auditor": "🔐",
        "ui-optimizer": "🎨",
    }

    agents = []
    for md_file in sorted(roles_dir.glob("*.md")):
        stem = md_file.stem.replace("-", " ").title()
        emoji = emoji_map.get(md_file.stem, "📦")
        agents.append(f"{emoji}{stem}")

    return " | ".join(agents)


def get_repo_stats() -> str:
    """Statistiques live du codebase (analyse tout le repo)"""
    root = pathlib.Path(".")
    rust_count = len(list(root.rglob("*.rs")))
    svelte_count = len(list(root.rglob("**/*.svelte")))
    toml_count = len(list(root.rglob("Cargo.toml"))) + len(list(root.rglob("*.toml")))
    return f"({rust_count} fichiers Rust | {svelte_count} composants Svelte | {toml_count} fichiers TOML)"


def generate_instruction(version: str, session: str, date: str, active_bugs: str) -> tuple[str, int]:
    """Génère le texte de l'instruction (<1500 chars)"""
    agents_str = get_agents_list()
    stats = get_repo_stats()

    instruction = f"""Tu es l'assistant principal du projet Nook (v{version}, session {session}).
Messagerie familiale self-hosted — Rust/Axum 0.8 + SvelteKit 5 Runes + SQLite + Docker distroless.
Repo: https://github.com/MX10-AC2N/Nook | Branche: main
Raw: https://raw.githubusercontent.com/MX10-AC2N/Nook/main/
Codebase actuelle : {stats}

AVANT CHAQUE INTERVENTION:
1. Fetcher .claude/BUGS.md ({active_bugs} bugs actifs)
2. Fetcher .claude/rules/memory-sessions.md
3. Fetcher les fichiers sources concernés (jamais travailler de mémoire)

AGENTS DISPONIBLES (fichiers dans .claude/roles/ — {len(get_agents_list().split(" | "))} agents) :
{agents_str}

RÈGLES ABSOLUES:
• Fichier complet — jamais de diff partiel
• .svelte/.ts → livrer en .txt
• Chemin exact en tête de chaque bloc de code
• Signaler les effets de bord inter-agents
• Clôture: mettre à jour BUGS.md + SESSIONS.md

Pièges critiques: rand::rng() (pas thread_rng) | routes {{param}} axum 0.8 | $state Svelte 5 via Object.assign | CORS + credentials → origins explicites | sqlx sans macros si queries.json vide"""

    return instruction, len(instruction)


def generate_markdown(instruction: str, char_count: int, date: str, version: str, session: str, active_bugs: str) -> str:
    """Génère le fichier Markdown complet"""
    status = "OK" if char_count <= 1500 else f"TROP LONG ({char_count} chars)"
    return f"""# Instruction personnalisée Android — Nook

> Générée le : **{date}** | Version : **{version}** | Session : **{session}**
> Taille : **{char_count} / 1500 chars** {status}

---

## Instruction à copier dans Claude.ai Android

> Paramètres → Instructions personnalisées → coller le texte ci-dessous

{instruction}

---

## Mise à jour

Ce fichier est **auto-généré** après analyse complète du repo + `.claude/`.
Il se met à jour automatiquement quand des rôles, règles, bugs ou le code changent.

Pour forcer une régénération : lance le workflow manuellement.
"""


def main():
    if len(sys.argv) != 5:
        print(f"Usage: {sys.argv[0]} <version> <session> <date> <active_bugs>", file=sys.stderr)
        sys.exit(1)

    version = sys.argv[1]
    session = sys.argv[2]
    date = sys.argv[3]
    active_bugs = sys.argv[4]

    instruction, char_count = generate_instruction(version, session, date, active_bugs)

    markdown_content = generate_markdown(
        instruction, char_count, date, version, session, active_bugs
    )

    output_path = ".claude/ANDROID-INSTRUCTION.md"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(markdown_content)

    print(f"✅ Instruction générée : {char_count} chars | {len(get_agents_list().split(' | '))} agents détectés")
    if char_count > 1500:
        print("⚠️  Attention : l'instruction dépasse 1500 caractères !")
        sys.exit(1)


if __name__ == "__main__":
    main()