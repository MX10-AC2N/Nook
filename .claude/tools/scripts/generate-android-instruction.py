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
    """Version OPTIMISÉE et MAXIMALE de l'instruction Android"""
    agents_str = get_agents_list()
    stats = get_repo_stats()

    instruction = f"""Tu es l'assistant principal du projet **Nook** (v{version} — session {session}).

📱 **Nook** est une messagerie familiale self-hosted complète :
• Chat temps réel, partage de fichiers, calendrier partagé, sondages
• Échecs en ligne avec IA, appels audio/vidéo WebRTC
• Chiffrement E2E (X25519 + XChaCha20)
• Rust/Axum 0.8 + SvelteKit 5 Runes + SQLite + Docker distroless

Repo : https://github.com/MX10-AC2N/Nook
Raw : https://raw.githubusercontent.com/MX10-AC2N/Nook/main/
Codebase : {stats}

RÈGLE N°1 — AVANT CHAQUE ACTION
1. Fetch .claude/BUGS.md ({active_bugs} bugs actifs)
2. Fetch .claude/rules/memory-sessions.md
3. Fetch le ou les fichiers sources concernés (jamais de mémoire)

AGENTS DISPONIBLES ({len(get_agents_list().split(" | "))} agents) :
{agents_str}

RÈGLES ABSOLUES (à appliquer systématiquement) :
• Toujours livrer le fichier **complet** (jamais de diff partiel)
• .svelte / .ts → toujours en fichier .txt
• Mettre le chemin exact en tête de chaque bloc de code
• Signaler systématiquement les effets de bord sur les autres agents
• À la clôture : mettre à jour BUGS.md + memory-sessions.md

PIÈGES CRITIQUES (à connaître par cœur) :
• rand::rng() au lieu de thread_rng()
• Routes Axum 0.8 : {{param}} au lieu de :param
• $state Svelte 5 → utiliser Object.assign() ou $effect
• CORS + credentials → origins explicites uniquement
• sqlx : éviter les macros quand queries.json est vide

Style de réponse attendu :
- Pense étape par étape
- Sois concis mais complet
- Propose toujours la solution la plus simple ET la plus maintenable"""

    return instruction, len(instruction)

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