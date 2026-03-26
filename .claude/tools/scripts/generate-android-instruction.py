#!/usr/bin/env python3
"""
Génère l'instruction personnalisée Android Claude.ai
→ Analyse complète du repo + .claude/roles/ + critical-pitfalls.md
"""

import sys
import pathlib


def get_agents_list() -> str:
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
    root = pathlib.Path(".")
    rust_count = len(list(root.rglob("*.rs")))
    svelte_count = len(list(root.rglob("**/*.svelte")))
    toml_count = len(list(root.rglob("Cargo.toml"))) + len(list(root.rglob("*.toml")))
    return f"({rust_count} fichiers Rust | {svelte_count} composants Svelte | {toml_count} fichiers TOML)"


def get_critical_pitfalls() -> str:
    """Lit dynamiquement .claude/rules/critical-pitfalls.md"""
    pitfalls_file = pathlib.Path(".claude/rules/critical-pitfalls.md")
    if not pitfalls_file.exists():
        return "Aucun piège critique défini pour le moment."

    content = pitfalls_file.read_text(encoding="utf-8")
    pitfalls = []
    for line in content.splitlines():
        line = line.strip()
        if line.startswith(("- ", "* ", "• ")):
            pitfalls.append(line[2:].strip())
        elif line.startswith("•"):
            pitfalls.append(line[1:].strip())

    return " | ".join(pitfalls[:10]) if pitfalls else "Aucun piège critique défini."


def generate_instruction(version: str, session: str, date: str, active_bugs: str) -> tuple[str, int]:
    agents_str = get_agents_list()
    stats = get_repo_stats()
    pitfalls = get_critical_pitfalls()

    instruction = f"""Tu es l'assistant principal du projet **Nook** (v{version} — session {session}).

📱 **Nook** : messagerie familiale self-hosted complète (chat, fichiers, calendrier, sondages, échecs IA, WebRTC, E2E X25519 + XChaCha20).
Rust/Axum 0.8 + SvelteKit 5 Runes + SQLite + Docker distroless.

Repo : https://github.com/MX10-AC2N/Nook
Raw  : https://raw.githubusercontent.com/MX10-AC2N/Nook/main/
Codebase : {stats}

RÈGLE N°1 — AVANT CHAQUE ACTION
1. Fetch .claude/BUGS.md ({active_bugs} bugs actifs)
2. Fetch .claude/rules/memory-sessions.md
3. Fetch .claude/rules/critical-pitfalls.md
4. Fetch le(s) fichier(s) source concerné(s)

AGENTS DISPONIBLES ({len(get_agents_list().split(" | "))} agents) :
{agents_str}

RÈGLES ABSOLUES :
• Toujours livrer le fichier **complet** (jamais de diff partiel)
• .svelte / .ts → toujours en .txt
• Chemin exact en tête de chaque bloc de code
• Signaler les effets de bord inter-agents
• Clôture : mettre à jour BUGS.md + memory-sessions.md

PIÈGES CRITIQUES (issus de .claude/rules/critical-pitfalls.md) :
{pitfalls}

Style attendu : pense étape par étape, sois concis mais complet, propose la solution la plus simple ET maintenable."""

    return instruction, len(instruction)


def generate_markdown(instruction: str, char_count: int, date: str, version: str, session: str, active_bugs: str) -> str:
    status = "OK" if char_count <= 1500 else f"TROP LONG ({char_count} chars)"
    return f"""# Instruction personnalisée Android — Nook

> Générée le : **{date}** | Version : **{version}** | Session : **{session}**
> Taille : **{char_count} / 1500 chars** {status}

---

## Instruction à copier dans Claude.ai Android

{instruction}

---

## Mise à jour
Fichier auto-généré après analyse complète du repo + `.claude/`.
Se met à jour dès qu’un rôle, une règle ou un piège critique change.
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

    print(f"✅ Instruction générée : {char_count} chars | {len(get_agents_list().split(' | '))} agents | pitfalls chargés")


if __name__ == "__main__":
    main()