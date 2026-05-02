---
name: nook-agent-manager
category: "devops"
description: "Manage the .hermes/ directory — audit, create, update, and delete roles/skills/rules. Ensure optimal agent ecosystem for Nook."
---

# 🧑‍💼 Agent Manager Skill

## Trigger
Run when:
- "Audit .hermes/" or "optimize agents"
- New feature needs a role/skill
- Agents seem outdated
- After major refactoring

## Steps

### 1. Audit
```bash
# List all files in .hermes/
find .hermes/ -name "*.md" | wc -l
find .hermes/ -name "SKILL.md" | wc -l
```

### 2. Check relevance
For each file:
- When was it last modified?
- Is it referenced in CLAUDE.md?
- Does it match current project state?

### 3. Create missing
- Roles for unhandled domains
- Skills for repeated procedures
- Rules for new conventions

### 4. Update existing
- Add new patterns discovered
- Remove outdated information
- Fix broken references

### 5. Delete obsolete
- Reports > 30 days
- Roles never used
- Skills with wrong information

## Output
```markdown
## Agent Audit — [Date]
- Total files: [N]
- Roles: [N] (active/inactive)
- Skills: [N] (active/inactive)
- Created: [list]
- Updated: [list]
- Deleted: [list]
```
