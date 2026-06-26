---
name: hermes-skills-hub
description: Manage Hermes Agent skills from the Nous Research hub and community registries — browse, search, install, update, audit. Covers official (builtin) skills from Nous Research and community skills from skills.sh, lobehub, clawhub, and other sources.
category: devops
tags:
  - hermes
  - skills
  - cli
  - hub
  - nous-research
  - package-management
---

# Hermes Skills Hub Management

Skill for operating the `hermes skills` CLI to discover, install, and maintain skills from the Nous Research hub and community registries.

## When to Use

- Discovering available skills for a task (`hermes skills browse`, `hermes skills search`)
- Installing official Nous Research skills (`hermes skills install official/...`)
- Installing community skills (`hermes skills install skills-sh/...`, `lobehub/...`)
- Checking for skill updates (`hermes skills check`)
- Auditing installed skills (`hermes skills audit`)
- Understanding security scan verdicts (SAFE vs DANGEROUS)
- Managing skill sources/taps (`hermes skills tap`)

## Key Commands

| Command | Purpose |
|---------|---------|
| `hermes skills browse [--page N]` | Paginated list of all 1297+ skills |
| `hermes skills search <query> [--source <src>]` | Search by keyword, optionally filter source |
| `hermes skills inspect <identifier>` | Preview skill before installing |
| `hermes skills install <identifier> [--yes] [--force]` | Install skill (quarantine + security scan) |
| `hermes skills check` | Check installed hub skills for updates |
| `hermes skills update` | Update installed hub skills |
| `hermes skills list` | List locally installed skills |
| `hermes skills audit` | Re-scan installed hub skills |
| `hermes skills tap list/add/remove` | Manage skill registries |

## Skill Sources & Trust Levels

| Source | Trust | Description |
|--------|-------|-------------|
| `official` | ★ official | Nous Research maintained, builtin to hermes-agent |
| `skills-sh` | trusted/community | skills.sh index (Anthropic, Vercel, etc.) |
| `lobehub` | community | LobeHub skill registry |
| `clawhub` | community | ClawHub registry |
| `github` | community | Direct from GitHub repos |

**Official skills** are bundled with hermes-agent but not activated by default — installing copies them to your skills directory.

## Security Scan Verdicts

Every install runs a security scan. Verdicts:

- **SAFE** — No critical/high findings. Install proceeds.
- **DANGEROUS** — Critical/high findings flagged. **Official skills are ALLOWED despite DANGEROUS**; community skills may be blocked.

Common patterns that trigger DANGEROUS in official skills:
- Reading environment variables for tokens (watchers, rest-graphql-debug)
- Container runtime patterns (docker-management, hermes-s6-container-supervision)
- Package installation commands in references
- Path traversal references in forensic/security skills

**Decision**: Official = ALLOWED (builtin source). Community = case-by-case.

## Installation Path

Skills install to: `/opt/data/home/.hermes/skills/<category>/<name>/`

**Not** the repo's `.hermes/skills/` — the runtime loads from `/opt/data/home/.hermes/skills/`.

After repo changes, sync: `cp -r .hermes/skills/* /opt/data/home/.hermes/skills/` (or reinstall via hub).

## Common Patterns

### Install Official Skill (non-interactive)
```bash
hermes skills install official/devops/docker-management --yes
```

### Install Community Skill (full identifier from search)
```bash
hermes skills install skills-sh/rust-best-practices --yes
hermes skills install lobehub/rust-expert --yes
```

### Search Tips
- Official skills use path: `official/<category>/<name>` (e.g., `official/security/sherlock`)
- Community skills use full identifier from search results (e.g., `skills-sh/affaan-m/everything-claude-code/rust-testing`)
- Some community identifiers fail to resolve — try `hermes skills inspect <id>` first

### Update Workflow
```bash
hermes skills check      # See what's outdated
hermes skills update     # Update all
# Or update specific: hermes skills install <id> --force
```

## Pitfalls & Gotchas

1. **Gateway blocks restart from inside** — `hermes gateway restart` fails if run from within the gateway process. Run from external shell.

2. **Config changes need gateway restart** — `hermes config set rich_messages true` takes effect after gateway restart.

3. **Skill identifiers from browse/search can be truncated** — use `hermes skills inspect` to verify before install.

4. **Community skills may fail to fetch** — skills.sh/lobehub identifiers sometimes 404. Try alternative naming or different source.

5. **--yes flag required for automation** — Interactive prompts block CI/non-TTY runs.

6. **DANGEROUS verdict ≠ blocked** — Official skills pass anyway. Read the scan output to understand what the skill does.

7. **Rich messages config** — Set via `hermes config set rich_messages true` for formatted CLI output.

## Related Skills

- `hermes-agent` (bundled) — General Hermes CLI usage, config, gateway, profiles
- `dev-practices` — Software development practices including tooling workflows
- `nook-github-workflows` — CI/CD orchestration (uses skills for automation)