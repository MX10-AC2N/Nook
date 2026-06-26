---
name: ai-coding-agents
description: Complete guide to delegating coding tasks to autonomous AI agents — Claude Code, Codex, OpenCode, and Hermes Agent spawning.
version: 2.0.0
tags: [claude-code, codex, opencode, hermes-agent, autonomous, delegation, coding-agent, cli, pr, feature]
related_skills: []
---

# AI Coding Agents

Unified interface for delegating coding tasks to autonomous AI coding agents. Covers Claude Code CLI, OpenAI Codex CLI, OpenCode CLI, and Hermes Agent self-spawning.

## Quick Navigation

| Agent | CLI Command | Best For | Auth |
|-------|-------------|----------|------|
| [Claude Code](#1-claude-code) | `claude` | Features, PRs, complex refactors | Anthropic API / Console |
| [Codex](#2-codex) | `codex` | Features, PRs, repo exploration | OpenAI API |
| [OpenCode](#3-opencode) | `opencode` | PR review, features, codebase queries | Anthropic/OpenAI/other |
| [Hermes Agent](#4-hermes-agent-self-spawn) | `hermes` + subagents | Parallel workflows, skill-based tasks | Local/configured providers |

---

## 1. Claude Code

### Install & Auth

```bash
# Install
npm install -g @anthropic-ai/claude-code

# Auth (one-time)
claude auth login
# Opens browser for Anthropic Console authentication
```

### Delegate Coding Tasks

```bash
# Feature development
claude-code --prompt "Add user authentication with JWT. Create login/register endpoints, User model, auth middleware, and tests."

# PR creation
claude-code --prompt "Create PR for the auth feature. Run tests, ensure CI passes."

# Code review
claude-code --prompt "Review PR #42. Check for security issues, performance, and code quality."

# Bug fix
claude-code --prompt "Fix the login redirect bug. The ?next= parameter is ignored after login."
```

### Key Capabilities

- **Full repo context** — Reads codebase, understands architecture
- **Git operations** — Creates branches, commits, pushes, opens PRs
- **Test execution** — Runs test suites, fixes failures
- **Multi-file edits** — Coordinates changes across files
- **CI integration** — Monitors and fixes CI failures

### Usage Pattern

```bash
# Long-running feature work (background)
claude-code --prompt "Build the notifications feature per SPEC.md" --background

# Quick fix
claude-code --prompt "Fix typo in README.md"

# With specific files
claude-code --prompt "Refactor auth module" --files "src/auth/*.py"
```

---

## 2. Codex

### Install & Auth

```bash
# Install
npm install -g @openai/codex

# Auth
codex auth login
# Uses OpenAI API key
```

### Delegate Coding Tasks

```bash
# Feature development
codex --prompt "Implement websocket support for real-time chat. Add connection handling, message routing, and reconnection logic."

# PR workflow
codex --prompt "Create PR with the websocket implementation. Include tests and documentation."

# Code exploration
codex --prompt "Explain how the authentication flow works in this codebase."

# Refactoring
codex --prompt "Extract the database connection logic into a separate module."
```

### Key Capabilities

- **Sandboxed execution** — Runs in isolated environment
- **Git integration** — Commits, branches, PRs
- **Language agnostic** — Works with any language
- **Parallel execution** — Multiple agents on different tasks

---

## 3. OpenCode

### Install & Auth

```bash
# Install
npm install -g opencode-ai

# Auth
opencode auth login
# Supports Anthropic, OpenAI, and other providers
```

### Delegate Coding Tasks

```bash
# PR review (specialty)
opencode review --pr 42 --depth thorough

# Feature development
opencode --prompt "Add rate limiting to API endpoints. Use token bucket algorithm."

# Codebase query
opencode --prompt "Find all places where user input is used in SQL queries."

# Debugging
opencode --prompt "Investigate the memory leak in the websocket handler."
```

### Key Capabilities

- **PR review focus** — Specialized for code review workflows
- **Multi-provider** — Anthropic, OpenAI, local models
- **Codebase indexing** — Fast semantic search
- **Interactive mode** — Chat-style exploration

---

## 4. Hermes Agent Self-Spawn

### Subagent Delegation (via delegate_task tool)

```python
# In Hermes agent context
delegate_task(
    goal="Implement user authentication with JWT",
    context="Repo: /opt/data/Nook. Backend is Rust/Axum. Frontend is SvelteKit. Use existing patterns in src/auth.rs",
    toolsets=["terminal", "file", "web"]
)

# Parallel batch delegation
delegate_task(
    tasks=[
        {"goal": "Backend: Add login/register endpoints", "toolsets": ["terminal", "file"]},
        {"goal": "Frontend: Create login/register pages", "toolsets": ["terminal", "file"]},
        {"goal": "Tests: Add auth integration tests", "toolsets": ["terminal", "file"]}
    ]
)
```

### Cron Job Agents

```bash
# Scheduled autonomous runs
cronjob create \
  --name "nightly-code-review" \
  --schedule "0 2 * * *" \
  --prompt "Review all open PRs in MX10-AC2N/Nook. Post security and quality comments." \
  --skills ["github-code-review", "github-pr-workflow"]
```

### ACP Transport (GitHub Copilot)

```bash
# If Copilot CLI is available
delegate_task(
    goal="Refactor the user service",
    acp_command="copilot",
    acp_args=["--acp", "--stdio"],
    toolsets=["terminal", "file"]
)
```

---

## Comparison Matrix

| Feature | Claude Code | Codex | OpenCode | Hermes Subagent |
|---------|-------------|-------|----------|-----------------|
| **Provider** | Anthropic | OpenAI | Multi | Your config |
| **PR Creation** | ✅ | ✅ | ✅ | Via git |
| **PR Review** | ✅ | ✅ | ⭐ Specialty | Via skills |
| **Parallel Tasks** | Limited | ✅ | ✅ | ✅ (3 concurrent) |
| **Repo Context** | Full | Full | Indexed | Via toolsets |
| **Background Runs** | ✅ | ✅ | ✅ | Via cronjob |
| **Cost Model** | Subscription | API usage | API usage | Your providers |
| **Offline/Local** | ❌ | ❌ | With local LLM | ✅ |

---

## When to Use Each

| Scenario | Recommended Agent |
|----------|-------------------|
| Complex feature, need deep reasoning | Claude Code |
| Quick exploration, OpenAI ecosystem | Codex |
| PR review, security audit | OpenCode |
| Parallel sub-tasks, skill-based workflows | Hermes Subagent |
| Scheduled/recurring autonomous work | Hermes Cronjob |
| Local-only, privacy-sensitive | Hermes Subagent + local provider |

---

## Common Patterns

### Feature Development (All Agents)

```bash
# 1. Create spec/plan first
cat > SPEC.md << 'EOF'
## Feature: User Notifications
- Real-time via WebSocket
- Email fallback
- Preferences per user
EOF

# 2. Delegate with spec
claude-code --prompt "Implement per SPEC.md"
codex --prompt "Implement per SPEC.md"
opencode --prompt "Implement per SPEC.md"
```

### PR Review Workflow

```bash
# OpenCode (specialized)
opencode review --pr 123 --depth thorough --post-comments

# Claude Code
claude-code --prompt "Review PR #123. Focus on security and performance. Post inline comments."

# Hermes (with skills)
delegate_task(
    goal="Review PR #123 for security and quality",
    context="Repo: MX10-AC2N/Nook. PR adds websocket support.",
    toolsets=["terminal", "file", "web"],
    skills=["github-code-review"]
)
```

### Bug Fix Delegation

```bash
# Provide context: error, logs, reproduction steps
claude-code --prompt "
Fix the login redirect bug.
Error: After login, user lands on /dashboard instead of /settings?next=...
Reproduction: 1) Go to /settings logged out 2) Redirected to /login?next=/settings 3) Login 4) Actual: /dashboard
Root cause hypothesis: auth middleware drops ?next= parameter.
Files to check: src/auth/middleware.rs, src/auth/handlers.rs
"
```

---

## Hermes-Specific: Skill-Based Delegation

Hermes subagents can load skills for specialized knowledge:

```python
delegate_task(
    goal="Fix Nook backend build failures",
    context="Repo: /opt/data/Nook. Clippy warnings, Axum 0.8 migration, musl targets.",
    toolsets=["terminal", "file"],
    skills=["nook-build-ci"]  # Loads the consolidated build skill
)

delegate_task(
    goal="Review PR for GitHub best practices",
    context="PR #45 in MX10-AC2N/Nook",
    toolsets=["terminal", "file", "web"],
    skills=["github-cli-workflow"]
)
```

---

## Reference Files

| File | Purpose |
|------|---------|
| `references/claude-code-setup.md` | Detailed Claude Code installation & config |
| `references/codex-setup.md` | Codex installation & API key management |
| `references/opencode-setup.md` | OpenCode multi-provider configuration |
| `references/hermes-delegation-patterns.md` | Advanced delegate_task patterns |

---

## When to Use This Skill

- Delegating feature development to autonomous agents
- Automating PR creation and review workflows
- Parallelizing independent coding tasks
- Running scheduled code audits and reviews
- Bug investigation and fix delegation
- Codebase exploration and documentation