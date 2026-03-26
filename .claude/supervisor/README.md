# Superviseur Nook

## Installation

```bash
# 1. Migrer les rôles existants
python3 .claude/supervisor/migrate_roles.py

# 2. Installer les dépendances
pip install pyyaml anthropic

# 3. Configurer les clés API
export ANTHROPIC_API_KEY="your-key-here"

# 4. Tester le superviseur
python3 .claude/supervisor/cli.py status