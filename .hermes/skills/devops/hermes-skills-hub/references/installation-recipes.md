# Installation Recipes for Common Nook Stack Skills

## Official Skills (Nous Research)

### DevOps / Infrastructure
```bash
# Docker lifecycle, debugging, Dockerfile optimization
hermes skills install official/devops/docker-management --yes

# Zero-install localhost tunnels via SSH
hermes skills install official/devops/pinggy-tunnel --yes

# Poll RSS, JSON APIs, GitHub with watermark dedup
hermes skills install official/devops/watchers --yes

# s6-overlay supervision inside Hermes Docker image
hermes skills install official/devops/hermes-s6-container-supervision --yes
```

### Security / Forensics
```bash
# Supply chain investigation, commit recovery, GitHub forensics
hermes skills install official/security/oss-forensics --yes

# OSINT username search across 400+ networks
hermes skills install official/security/sherlock --yes

# Authorized web app pentesting (recon, vuln analysis, reporting)
hermes skills install official/security/web-pentest --yes
```

### ML Ops / Inference
```bash
# Structured JSON/regex/Pydantic LLM generation
hermes skills install official/mlops/outlines --yes
```

### Research
```bash
# Free web search via DuckDuckGo (ddgs CLI)
hermes skills install official/research/duckduckgo-search --yes
```

### Software Development
```bash
# Debug REST/GraphQL APIs: status codes, auth, schemas
hermes skills install official/software-development/rest-graphql-debug --yes
```

## Community Skills (skills.sh / lobehub)

### Rust Development
```bash
# Idiomatic patterns, clippy config, performance, error handling
hermes skills install skills-sh/rust-best-practices --yes

# Rust teaching, comparisons, learning plans, exercises
hermes skills install lobehub/rust-expert --yes

# Rust programming support, learning assistance
hermes skills install lobehub/rust-assistant --yes
```

## Search & Install Workflow

```bash
# 1. Search for relevant skills
hermes skills search rust
hermes skills search docker
hermes skills search svelte

# 2. Inspect before installing (verify identifier, read content)
hermes skills inspect official/devops/docker-management
hermes skills inspect skills-sh/rust-best-practices

# 3. Install with --yes for non-interactive
hermes skills install <identifier> --yes

# 4. Check for updates periodically
hermes skills check
hermes skills update
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Community skill 404 on install | Try `hermes skills inspect <id>` first; identifier may be truncated in search results |
| "Already installed" warning | Use `--force` to reinstall: `hermes skills install <id> --force` |
| Skill not loading after install | Verify installed to `/opt/data/home/.hermes/skills/` not repo `.hermes/skills/` |
| Scanner blocks agent-created skill | The skill content contains scanner trigger words; this is expected for docs about security |