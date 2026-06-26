---
name: nook-build-ci
description: Complete Nook build and CI orchestration — backend/frontend/TURN builds, multi-arch Docker, GitHub Actions workflow management, Clippy/audit fixes, and deployment coordination.
version: 2.0.0
tags: [nook, rust, sveltekit, github-actions, docker, ci-cd, musl, alpine, cross-compile, axum, clippy]
related_skills: []
---

# Nook Build & CI Orchestration

Complete build and CI/CD workflow for the Nook project (MX10-AC2N/Nook). Covers backend (Rust/Axum), frontend (SvelteKit), TURN server (turn-rs), multi-arch Docker builds, GitHub Actions orchestration, and deployment coordination.

## Quick Navigation

| Section | Purpose |
|---------|---------|
| [Backend Build](#1-backend-build) | musl targets, Clippy fixes, Axum 0.8 migration, native runners |
| [Frontend Build](#2-frontend-build) | SvelteKit, npm ci, package-lock sync, Playwright |
| [TURN Server Build](#3-turn-server-build) | protoc, musl-tools, ARM64 native |
| [Docker Multi-Arch](#4-docker-multi-arch-build) | Artifact naming, branch tagging, workflow order |
| [GitHub Actions Orchestration](#5-github-actions-orchestration) | Workflow IDs, headSha filtering, anti-repeat-loop |
| [Deployment](#6-deployment-coordination) | Version verification, Zimaboard, GHCR |

---

## 1. Backend Build

### Architecture (May 2026 - PROVEN)

**Two separate jobs on native runners** — NO matrix, NO cross-compilation, NO cargo-zigbuild.

```yaml
jobs:
  backend-amd64:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-musl
          components: rustfmt, clippy
      - run: sudo apt-get update && sudo apt-get install -y musl-tools
      - run: cargo check --target x86_64-unknown-linux-musl
      - run: cargo clippy --target x86_64-unknown-linux-musl
      - run: cargo test  # native, no target
      - run: cargo build --release --target x86_64-unknown-linux-musl

  backend-arm64:
    runs-on: ubuntu-24.04-arm
    steps:
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-unknown-linux-musl
          components: rustfmt, clippy
      - run: sudo apt-get update && sudo apt-get install -y musl-tools
      - run: cargo check --target aarch64-unknown-linux-musl
      - run: cargo clippy --target aarch64-unknown-linux-musl
      - run: cargo test
      - run: cargo build --release --target aarch64-unknown-linux-musl
```

**Key Principles:**
- Native runners only: `ubuntu-latest` (amd64), `ubuntu-24.04-arm` (arm64)
- `musl-tools` via apt on each runner
- `cargo test` runs native (no `--target`)
- All check/clippy steps use `--target <musl>` to validate target-specific cfg gates
- Build uses `LTO`, `opt-level=z`, `strip=true` for small static binaries
- Binary path: `target/${{ matrix.target }}/release/nook-backend`
- ~8 minutes total (parallel)

### Critical Rules

| Rule | Detail |
|------|--------|
| ✅ Use `aarch64-unknown-linux-musl` | NOT `aarch64-unknown-linux-gnu-musl` (invalid) |
| ✅ Install `musl-tools` | `sudo apt-get install -y musl-tools` on each runner |
| ❌ Never use `cargo-zigbuild` | Fails with `--print=file-names` error on GH Actions |
| ❌ No cross-compilation | arm64 from amd64 runner fails target detection |
| ❌ No `cargo zigcheck`/`zigclippy` | These subcommands don't exist |

### Clippy Fixes (Systematic)

**Audit every `#![allow()]` before keeping — they rot silently.**

```bash
# 1. List all module-level allows
grep -rn '#!\[allow(' backend/src/

# 2. For each, verify the suppressed pattern EXISTS:
# for_kv_map:   grep -rn 'for\s*(\s*_\s*,' file.rs  (expect unused key)
# single_char_add_str: grep -rn 'push_str("\\n")' file.rs
# deprecated chrono: grep -rn 'from_utc\|from_timestamp_opt' file.rs
# unnecessary_map_or: grep -rn '\.map_or(' file.rs
# too_many_lines: max function lines < 100

# 3. Remove if pattern not found. Keep only legitimate allows.
```

**May 2026 Audit Result**: 44 lines deleted across 8 files. Common dead allows:
- `for_kv_map` in 6 files — ZERO had `(_, v)` with unused key
- `events.rs`: blanket `#![allow(clippy)]`, 5 duplicate `too_many_lines`, `missing_errors_doc`
- `chess.rs`: duplicate `#![allow(deprecated)]` + 44 lines dead code (duplicated header)

### Axum 0.8 Migration

```rust
// OLD (0.6/0.7)
Router::new().route("/events/:id", get(handler))

// NEW (0.8+)
Router::new().route("/events/{id}", get(handler))
```

Files: `backend/src/events.rs` (line 316), check all routers.

### Chrono Deprecations

```rust
// OLD
DateTime::from_utc()
NaiveDateTime::from_timestamp_opt()

// NEW
TimeZone::from_utc_datetime()
DateTime::from_timestamp()
```

### Verification

```bash
cd /tmp/Nook/backend
cargo check --all-targets
cargo clippy --all-targets -- -A clippy::too_many_lines
cargo test --all-targets
cargo build --release --target x86_64-unknown-linux-musl
file target/x86_64-unknown-linux-musl/release/nook-backend
# Should show: ELF 64-bit LSB executable, x86-64, statically linked, stripped
```

---

## 2. Frontend Build

### Common Failures

| Error | Fix |
|-------|-----|
| `npm ci: package-lock.json out of sync` | `cd frontend && npm install --include-optional --no-audit --no-fund && git add package-lock.json` |
| `esbuild` version drift | Lock to `0.27.4` in package.json |
| Missing optional deps (`@esbuild/*`, `@rollup/*`, `fsevents`) | Regenerate lockfile with `--include-optional` |
| Missing `workbox-window` | `npm install workbox-window` |
| Svelte parse errors | Run `mcp_svelte_mcp_svelte_autofixer` on modified `.svelte` files |

### Workflow (Frontend.yml)

```yaml
# Use exact workflow ID to avoid emoji/name issues
gh workflow run 220018364 --ref develop --repo MX10-AC2N/Nook
```

---

## 3. TURN Server Build

### Fixed Pattern (turn.yml)

```yaml
- uses: actions/checkout@v4
  with:
    persist-credentials: true
    repository: MX10-AC2N/Nook

- name: Ensure cargo is on PATH
  run: echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"

- name: Install build deps
  run: |
    sudo apt-get update
    sudo apt-get install -y protobuf-compiler libprotobuf-dev musl-tools

- uses: dtolnay/rust-toolchain@stable
  with:
    targets: x86_64-unknown-linux-musl aarch64-unknown-linux-musl
    components: rustfmt, clippy

# Build both architectures
```

### Artifact Names
- `nook-turn-server-amd64`
- `nook-turn-server-arm64`

Trigger via ID: `gh workflow run 257238341 --ref develop --repo MX10-AC2N/Nook`

---

## 4. Docker Multi-Arch Build

### Workflow Order (Non-Negotiable)

```
Frontend.yml  →  Backend.yml  →  Turn.yml  →  Docker.yml
```

**Docker.yml is ALWAYS manual** — `workflow_run` trigger is NOT configured. The `if:` condition checking `github.event.workflow_run.conclusion` is dead code.

```bash
# Trigger Docker manually AFTER all 3 succeed
gh workflow run 220018363 --ref develop --repo MX10-AC2N/Nook
```

### Artifact Naming (Critical)

Backend.yml uploads with Rust target triple as GitHub artifact name:
- `nook-backend-x86_64-unknown-linux-musl` → contains binary `nook-backend-amd64`
- `nook-backend-aarch64-unknown-linux-musl` → contains binary `nook-backend-arm64`

Docker.yml MUST download using GitHub artifact names, verify binary names match.

### Branch Tagging

```yaml
- name: Set branch tag
  id: branch_tag
  run: |
    BRANCH="${{ inputs.branch_name }}"
    if [ "$BRANCH" = "main" ]; then
      echo "tag=latest" >> $GITHUB_OUTPUT
    elif [ "$BRANCH" = "develop" ]; then
      echo "tag=dev" >> $GITHUB_OUTPUT
    else
      echo "tag=$BRANCH" >> $GITHUB_OUTPUT
    fi
```

**Never use `github.ref_name`** for tagging in manual workflows — it fails. Use `inputs.branch_name` with default `develop`.

### Dockerfile.release (Alpine)

```dockerfile
# Use musl binaries from artifacts
COPY nook-backend-amd64 /usr/local/bin/nook-backend
# Alpine packages:
# sqlite-libs (NOT libsqlite3 - Debian name)
# ca-certificates libsodium
RUN apk add --no-cache sqlite-libs ca-certificates libsodium
```

---

## 5. GitHub Actions Orchestration

### Anti-Repeat-Loop Discipline (User Mandate)

> \"Tu as tendance à refaire en boucle les choses\"

**MANDATORY pre-trigger checklist:**

```bash
# 0. FETCH FIRST — origin/develop may be ahead
git fetch origin --quiet
NEW_COMMITS=$(git log HEAD..origin/develop --oneline)
if [ -n "$NEW_COMMITS" ]; then
  echo "origin/develop ahead — resetting"
  git reset --hard origin/develop
fi

# 1. Verify branch is develop (NOT main — user called this out 2026-06-05)
git branch --show-current  # Must output: develop

# 2. Check recent runs - filter by YOUR commit SHA
MY_SHA=$(git rev-parse --short HEAD)
gh run list --repo MX10-AC2N/Nook --limit 20 --json number,headSha,status,workflowName,conclusion \
  | python3 -c "
import sys,json
runs=json.load(sys.stdin)
[print(f'{r[\"number\"]} | {r[\"headSha\"][:7]} | {r[\"status\"]} | {r[\"workflowName\"]}') for r in runs if r['headSha'][:7]=='$MY_SHA']
"

# 3. Check which files changed since last Docker build
gh run list --workflow 220018363 --repo MX10-AC2N/Nook --limit 1 --json headSha --jq '.[0].headSha' | read DOCKER_SHA
git diff --name-only "$DOCKER_SHA"..HEAD -- frontend/ backend/ services/turn-rs/

# 4. Map to workflows (only trigger those with actual changes)
# frontend/    → Frontend.yml (ID: 220018364)
# backend/     → Backend.yml  (ID: 220018362)
# services/turn-rs/ → turn.yml (ID: 257238341)

# 5. Verify not already running/queued
gh run list --status in_progress,queued --limit 3
```

### Free-Tier HeadSha Filtering (Primary Anti-Repeat Mechanism)

On free tier, `gh run list` returns runs in arrival order, not commit order. The "top" run may belong to an older commit.

**Only reliable check**: filter by `headSha[:7]` matching YOUR `git rev-parse --short HEAD`.

### Python Polling Helper

```bash
# Wait for all workflows matching commit SHA
python3 references/poll-workflows.py --sha $(git rev-parse --short HEAD) --timeout 900
# Exits 0 on all success, 1 on any failure/timeout
```

### Prerequisites for CI Orchestration (CRITICAL - 2026-06-13)

The container running Hermes **must have**:
- **`gh` CLI installed** (`apt install gh` on Debian, `apk add github-cli` on Alpine)
- **`gh auth login` completed** with PAT having scopes `repo` + `workflow`
- Without these, **no workflow triggering or monitoring possible**

### Optimization: Skip Unchanged Upstream Workflows

If only `frontend/` changed (validated 2026-06-06):
- Trigger Frontend only
- Docker downloads existing Backend/Turn artifacts (latest by name)
- Docker builds in ~49s
- **No redundant Backend/Turn runs needed**

---

## 6. Deployment Coordination

### Workflow IDs (develop branch)

| Workflow | ID |
|----------|-----|
| Frontend | 220018364 |
| Backend | 220018362 |
| Turn | 257238341 |
| Docker | 220018363 |

### Artifact Names (Exact Match Required)

| Artifact | GitHub Name | Binary Inside |
|----------|-------------|---------------|
| Backend amd64 | `nook-backend-x86_64-unknown-linux-musl` | `nook-backend-amd64` |
| Backend arm64 | `nook-backend-aarch64-unknown-linux-musl` | `nook-backend-arm64` |
| Turn amd64 | `nook-turn-server-amd64` | `turn-server` |
| Turn arm64 | `nook-turn-server-arm64` | `turn-server` |
| Frontend | `frontend` | `index.html` + assets |

### Post-Deployment Verification (CRITICAL)

```bash
# 1. Get commit from successful Docker run
gh run view <Docker-run-id> --json headSha --jq '.headSha'

# 2. Verify deployed version
ssh root@192.168.1.192 'docker inspect nook --format "{{.Config.Image}} {{.Id}}"'
# Or use provided script: references/verify-deployed-version.sh

# 3. If mismatch, pull and restart
ssh root@192.168.1.192 'cd /opt/nook && docker pull ghcr.io/mx10-ac2n/nook:develop && docker restart nook-backend nook-frontend'

# 4. Test basic functionality (login, chat, navigation)
# 5. Check browser console for 401/MIME errors (backend down)
# 6. Check docker compose logs for runtime crashes
```

**Never test a bug fix on an outdated server image** — always confirm version alignment first.

---

## Common Pitfalls

1. **Triggering Docker too early** — wait for all 3 predecessor workflows
2. **Not retrying ECONNRESET** — transient network, rerun workflow, don't change code
3. **Using workflow names instead of IDs** — emojis/special chars break `gh workflow run`
4. **Forgetting `--repo` flag** — all gh commands need `--repo MX10-AC2N/Nook`
5. **Going in circles** — user: "On arrête de refaire les mêmes actions en boucle, on avance"
6. **Blind `git add -A` on .hermes** — adds build artifacts, use path-specific adds
7. **Dockerfile.release naming** — uses `Dockerfile.release` not `Dockerfile`
8. **Not testing after changes** — verify BEFORE declaring success
9. **Branch mismatch** — canonical branch is `develop`, never `develop-temp` or `main`
10. **Artifact pollution** — `git status --short` before push, clean `node_modules.bak`, `target/`, `dist/`
11. **cargo-zigbuild on GH Actions** — DOES NOT WORK despite successful install
12. **cargo zigcheck/zigclippy don't exist** — only `cargo zigbuild` exists
13. **Stale local repo** — always `git fetch origin` then check `HEAD..origin/develop`

---

## Reference Files

| File | Purpose |
|------|---------|
| `references/clippy-allow-status.md` | Clippy allow directive audit status |
| `references/ci-artifact-reuse-pattern.md` | Artifact reuse across workflows |
| `references/ci-trigger-discipline.md` | Anti-repeat-loop discipline details |
| `references/changed-dirs-workflow-mapping-2026-06-06.md` | Changed dirs → workflow mapping |
| `references/cargo-zigbuild-failure.md` | Why cargo-zigbuild fails on GH Actions |
| `references/gh-cli-prerequisite.md` | gh CLI install + auth prerequisite for CI |
| `references/ci-chain-2026-06-13.md` | Full CI chain execution log |
| `references/poll-workflows.py` | Multi-workflow polling script |
| `references/filter-runs-by-commit.py` | Filter gh runs by commit SHA |
| `scripts/wait_for_commit_runs.py` | Wait for CI completion |

---

## When to Use This Skill

- Backend build fails (Clippy, Axum, musl, chrono)
- Frontend CI fails (npm ci, package-lock, Svelte)
- TURN server CI fails (protoc, protobuf, arm64)
- Multi-arch Docker build issues (artifact naming, tagging)
- GitHub Actions orchestration (workflow order, monitoring)
- Deployment coordination (version verification, Zimaboard)
- Pre-push checklist for Nook repo changes