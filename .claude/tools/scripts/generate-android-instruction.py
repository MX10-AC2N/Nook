#!/usr/bin/env python3
"""
Génère l'instruction personnalisée pour l'application Android Claude.ai.
Ce script est appelé par le workflow GitHub Actions.

Usage:
    python3 generate-android-instruction.py <version> <session> <date> <active_bugs>
"""

import sys


def generate_instruction(version: str, session: str, date: str, active_bugs: str) -> tuple[str, int]:
    """
    Génère l'instruction Android et retourne le contenu avec le nombre de caractères.

    Args:
        version: Version du projet
        session: Numéro de session
        date: Date de génération
        active_bugs: Nombre de bugs actifs

    Returns:
        Tuple contenant l'instruction et le nombre de caractères
    """
    instruction = f"""Tu es l'assistant principal du projet Nook (v{version}, session {session}).
Messagerie familiale self-hosted — Rust/Axum 0.8 + SvelteKit 5 Runes + SQLite + Docker distroless.
Repo: https://github.com/MX10-AC2N/Nook | Branche: main
Raw: https://raw.githubusercontent.com/MX10-AC2N/Nook/main/

AVANT CHAQUE INTERVENTION:
1. Fetcher .claude/BUGS.md ({active_bugs} bugs actifs)
2. Fetcher .claude/rules/memory-sessions.md
3. Fetcher les fichiers sources concernés (jamais travailler de mémoire)

AGENTS DISPONIBLES (fichiers dans .claude/roles/):
🦀RUST | 🎨SVELTE | 🚀DEVOPS | 🧪E2E | 🔐CRYPTO | ♟CHESS | 📊DATA | 📐ARCHITECT | 🤖DELEGATE

RÈGLES ABSOLUES:
• Fichier complet — jamais de diff partiel
• .svelte/.ts → livrer en .txt
• Chemin exact en tête de chaque bloc de code
• Signaler les effets de bord inter-agents
• Clôture: mettre à jour BUGS.md + SESSIONS.md

Pièges critiques: rand::rng() (pas thread_rng) | routes {{param}} axum 0.8 | $state Svelte 5 via Object.assign | CORS + credentials → origins explicites | sqlx sans macros si queries.json vide"""

    char_count = len(instruction)
    return instruction, char_count


def generate_markdown(instruction: str, char_count: int, date: str, version: str, session: str, active_bugs: str) -> str:
    """
    Génère le contenu Markdown du fichier d'instruction.

    Args:
        instruction: L'instruction générée
        char_count: Nombre de caractères de l'instruction
        date: Date de génération
        version: Version du projet
        session: Numéro de session
        active_bugs: Nombre de bugs actifs

    Returns:
        Contenu Markdown complet
    """
    status = "✅ OK" if char_count <= 1500 else f"⚠️ TROP LONG ({char_count} chars)"

    return f"""# 📱 Instruction personnalisée Android — Nook

> Générée le : **{date}** | Version : **{version}** | Session : **{session}**
> Taille : **{char_count} / 1500 chars** {status}

---

## 📋 Instruction à copier dans Claude.ai Android

> Paramètres → Instructions personnalisées → coller le texte ci-dessous

{instruction}


---

## 🔄 Mise à jour

Ce fichier est **auto-généré** par le workflow `generate-android-instruction.yml`.
Il se met à jour automatiquement quand `VERSION`, `BUGS.md` ou `CLAUDE.md` changent.

Pour forcer une régénération : lancer le workflow manuellement depuis GitHub Actions.

---

## 📊 Statistiques

| | |
|---|---|
| Taille instruction | {char_count} chars / 1500 max |
| Bugs actifs | {active_bugs} |
| Version projet | {version} |
| Session | {session} |
"""


def main():
    """Point d'entrée principal du script."""
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

    print(f"Instruction générée: {char_count} chars")
    status = "✅ OK" if char_count <= 1500 else f"⚠️ TROP LONG ({char_count} chars)"
    print(status)

    if char_count > 1500:
        sys.exit(1)


if __name__ == "__main__":
    main()