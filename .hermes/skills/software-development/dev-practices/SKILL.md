---
name: dev-practices
description: Unified software development practices — planning, debugging, testing, code review, TDD, and spike experiments.
version: 2.0.0
tags: [planning, debugging, testing, tdd, code-review, spike, systematic-debugging, python-debug, node-debug, hermes-tui]
related_skills: []
---

# Development Practices

Unified toolkit for software development workflows: planning, debugging (Python, Node.js, Hermes TUI), testing (TDD), code review, and experimental spikes.

## Quick Navigation

| Practice | Purpose | When to Use |
|----------|---------|-------------|
| [Planning](#1-planning) | Markdown plans in .hermes/plans/, bite-sized tasks | Before any multi-step implementation |
| [Systematic Debugging](#2-systematic-debugging) | 4-phase root cause analysis | Any bug where cause is not immediately obvious |
| [Python Debugging](#3-python-debugging) | pdb REPL + debugpy remote (DAP) | Python bugs needing interactive inspection |
| [Node.js Debugging](#4-nodejs-debugging) | --inspect + Chrome DevTools Protocol | Node.js/TypeScript bugs |
| [Hermes TUI Debugging](#5-hermes-tui-debugging) | Slash commands, Python, gateway, Ink UI | Debugging Hermes Agent itself |
| [Test-Driven Development](#6-test-driven-development) | RED-GREEN-REFACTOR, tests before code | New features, bug fixes, refactoring |
| [Code Review](#7-pre-commit-code-review) | Security scan, quality gates, auto-fix | Before committing/pushing |
| [Spike Experiments](#8-spike-experiments) | Throwaway experiments to validate ideas | Before committing to implementation approach |

---

## 1. Planning

### Plan Mode

```bash
# Write plan to .hermes/plans/ (no execution)
plan "Add user authentication with JWT

## Tasks
1. Create User model with password hashing
2. Add login/register endpoints
3. Implement JWT token generation/validation
4. Add auth middleware for protected routes
5. Write unit and integration tests"

# View plans
ls .hermes/plans/
cat .hermes/plans/<plan-name>.md
```

### Writing Effective Plans

```python
# Use writing-plans patterns
# - Bite-sized tasks (1 task = 1 commit ideally)
# - Explicit file paths
# - Code snippets for key interfaces
# - Verification steps per task
# - Dependencies between tasks
```

### Plan Structure

```markdown
# Plan: Feature Name

## Context
- Repo: /path/to/repo
- Stack: Language, framework, key libs
- Related: Issue #123, PR #456

## Tasks
1. **Task name** (`path/to/file.ext`)
   - Details
   - Code sketch
   - Verification: `command to verify`

2. **Next task** ...

## Verification
- All tests pass: `pytest`
- CI green: `gh pr checks --watch`
- Manual test: steps
```

---

## 2. Systematic Debugging

### 4-Phase Process

| Phase | Goal | Output |
|-------|------|--------|
| **1. Understand** | Reproduce, gather facts, define expected vs actual | Minimal reproduction, fact list |
| **2. Locate** | Bisect, trace, isolate to function/module | Exact location (file:line) |
| **3. Fix** | Minimal change, verify fix works | Patch + test |
| **4. Prevent** | Add test, improve observability, document | Regression test + monitoring |

### Debugging Checklist

```bash
# Phase 1: Understand
# - Can I reproduce consistently?
# - What changed recently? (git log --oneline -20)
# - What are the exact error messages?
# - What is the expected behavior?

# Phase 2: Locate
# - Binary search: git bisect
# - Add logging/tracing at boundaries
# - Check: input -> function -> output at each layer
# - Use debugger breakpoints (see below)

# Phase 3: Fix
# - Minimal change principle
# - Verify fix doesn't break existing tests
# - Add regression test

# Phase 4: Prevent
# - Could type system catch this?
# - Could linting catch this?
# - Is there a pattern to generalize?
```

---

## 3. Python Debugging

### pdb (Built-in REPL)

```bash
# Insert breakpoint in code
breakpoint()
# or: import pdb; pdb.set_trace()

# Run with pdb
python -m pdb script.py

# Common commands
# n/next, s/step, c/continue, q/quit
# p expr / pp expr (print)
# l/list (show context), w/where (stack)
# u/up, d/down (navigate frames)
```

### debugpy (Remote/DAP - VS Code, etc.)

```bash
# Install
pip install debugpy

# Attach mode (script waits for debugger)
python -m debugpy --listen 5678 --wait-for-client script.py

# Connect from VS Code (launch.json):
# {
#   "type": "debugpy",
#   "request": "attach",
#   "connect": { "host": "localhost", "port": 5678 }
# }

# Programmatic
import debugpy
debugpy.listen(5678)
debugpy.wait_for_client()  # Blocks until attached
```

### Quick Debug Patterns

```python
# Conditional breakpoint
breakpoint() if condition else None

# Inspect variable without stopping
import sys; print(f"DEBUG: var={var}", file=sys.stderr)

# Post-mortem on exception
python -m pdb -c continue script.py  # Drops to pdb on unhandled exception
```

---

## 4. Node.js Debugging

### --inspect + Chrome DevTools

```bash
# Start with inspector
node --inspect=0.0.0.0:9229 script.js
node --inspect-brk=0.0.0.0:9229 script.js  # Break on first line

# Nodemon with inspect
nodemon --inspect script.js

# TypeScript (ts-node)
node --inspect -r ts-node/register script.ts
```

### Chrome DevTools Protocol CLI

```bash
# Install CDP CLI
npm install -g @devtools-protocol/cli

# List targets
devtools list

# Connect and run commands
devtools connect ws://localhost:9229/...
devtools evaluate "console.log('hello')"
```

### VS Code Launch Config

```json
{
  "type": "node",
  "request": "launch",
  "name": "Debug TypeScript",
  "runtimeArgs": ["-r", "ts-node/register"],
  "args": ["${workspaceFolder}/script.ts"],
  "sourceMaps": true
}
```

---

## 5. Hermes TUI Debugging

### Slash Commands

```bash
# In Hermes TUI
/python <code>        # Execute Python in agent context
/gateway <command>    # Gateway operations
/inspect <component>  # Ink UI component tree
```

### Debugging TUI Commands

```python
# Python REPL in TUI context
/python import sys; print(sys.path)
/python from hermes_tools import terminal; print(terminal("pwd"))

# Gateway debugging
/gateway status
/gateway logs --tail 50
/gateway restart

# Ink UI inspection
/inspect App
/inspect ChatView --props
```

---

## 6. Test-Driven Development

### RED-GREEN-REFACTOR Cycle

```bash
# 1. RED - Write failing test
cat > test_auth.py << 'EOF'
def test_login_returns_token():
    response = client.post("/login", json={"username": "test", "password": "pass"})
    assert response.status_code == 200
    assert "token" in response.json()
EOF
pytest test_auth.py  # FAILS

# 2. GREEN - Minimal implementation
# Edit src/auth.py to make test pass
pytest test_auth.py  # PASSES

# 3. REFACTOR - Improve without changing behavior
# Extract common code, improve names, etc.
pytest  # All tests still pass
```

### TDD Rules (Enforced)

1. **No production code without failing test**
2. **Write only enough test to fail**
3. **Write only enough code to pass**
4. **Refactor only on green**
5. **Run full suite after each cycle**

### Test Organization

```
tests/
├── unit/           # Fast, isolated, mocked
│   ├── test_auth.py
│   └── test_models.py
├── integration/    # Real DB, services
│   └── test_api.py
└── e2e/            # Full stack (Playwright, etc.)
    └── test_flow.py
```

### Coverage Targets

```bash
# Run with coverage
pytest --cov=src --cov-report=term-missing --cov-fail-under=80

# Per-module
pytest --cov=src/auth --cov-report=html
```

---

## 7. Pre-Commit Code Review

### Automated Gates

```bash
# Security scan
bandit -r src/          # Python
cargo audit             # Rust
npm audit               # Node.js

# Quality gates
ruff check src/         # Python lint
mypy src/               # Type check
cargo clippy            # Rust lint
eslint src/             # JS/TS lint

# Auto-fix
ruff check --fix src/
cargo fix --allow-dirty
eslint --fix src/
```

### Review Checklist

| Category | Checks |
|----------|--------|
| **Security** | No secrets, input validation, authz, no SQLi/XSS |
| **Correctness** | Edge cases, error handling, concurrency safety |
| **Quality** | Clear names, DRY, single responsibility, no premature abstraction |
| **Testing** | New paths covered, happy+error cases, readable tests |
| **Performance** | No N+1, appropriate caching, async-friendly |
| **Docs** | Public APIs documented, non-obvious logic explained |

### Pre-Commit Hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.4.0
    hooks:
      - id: ruff
        args: [--fix]
      - id: ruff-format
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.10.0
    hooks:
      - id: mypy
```

---

## 8. Spike Experiments

### Purpose

Throwaway code to validate an idea before committing to implementation. Time-boxed (typically 30-60 min).

### Spike Template

```bash
# Create spike directory
mkdir -p spikes/jwt-auth-experiment
cd spikes/jwt-auth-experiment

# Write minimal test
cat > spike.py << 'EOF'
import jwt
token = jwt.encode({"user_id": 1}, "secret", algorithm="HS256")
decoded = jwt.decode(token, "secret", algorithms=["HS256"])
print(decoded)
EOF

# Run
python spike.py

# Clean up (or keep for reference)
cd ../.. && rm -rf spikes/jwt-auth-experiment
```

### Spike Rules

1. **Time-box**: Set timer, stop when it rings
2. **No production code**: Spikes are disposable
3. **Document learning**: Write 3-bullet summary in plan/notes
4. **Decide**: Proceed / pivot / abandon
5. **Clean up**: Delete spike directory or move to archive

### When to Spike

- New library/framework evaluation
- Algorithm feasibility
- Performance characteristics
- Integration complexity
- "Will this even work?" questions

---

## Reference Files

| File | Purpose |
|------|---------|
| `references/debugging-cheatsheet.md` | Quick debugger command reference |
| `references/tdd-checklist.md` | TDD cycle enforcement checklist |
| `references/pre-commit-config.yaml` | Recommended pre-commit configuration |
| `references/spike-template.md` | Spike experiment structure |

---

## When to Use This Skill

- Planning multi-step features or refactors
- Debugging Python, Node.js, or Hermes TUI issues
- Enforcing TDD discipline
- Pre-commit quality and security gates
- Validating technical approaches with spikes
- Systematic root cause analysis