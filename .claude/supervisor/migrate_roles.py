"""
Migration automatique des rôles .claude/roles vers le système d'agents
"""
import json
from pathlib import Path

def migrate_roles():
    roles_dir = Path(".claude/roles")
    agents_dir = Path(".claude/agents")
    
    if not roles_dir.exists():
        print("❌ Dossier .claude/roles non trouvé")
        return
    
    agents_dir.mkdir(parents=True, exist_ok=True)
    
    for role_file in roles_dir.glob("*.md"):
        agent_id = role_file.stem
        agent_dir = agents_dir / agent_id
        agent_dir.mkdir(exist_ok=True)
        
        # Lire le rôle existant
        role_content = role_file.read_text()
        
        # Créer config.json pour l'agent
        config = {
            "agent_id": agent_id,
            "name": role_file.stem.replace("_", " ").title(),
            "role": agent_id,
            "source": f".claude/roles/{role_file.name}",
            "model": "sonnet",  # Par défaut
            "fallback_models": ["haiku"],
            "version": "1.0.0",
            "migrated_from": "roles"
        }
        
        config_file = agent_dir / "config.json"
        config_file.write_text(json.dumps(config, indent=2))
        
        # Copier le contenu comme système prompt
        system_file = agent_dir / "system.md"
        system_file.write_text(role_content)
        
        print(f"✅ Migré: {agent_id}")
    
    print(f"\n🎉 Migration terminée! {len(list(roles_dir.glob('*.md')))} rôles convertis")

if __name__ == "__main__":
    migrate_roles()