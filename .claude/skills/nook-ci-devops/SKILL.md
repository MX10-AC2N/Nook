---
name: nook-ci-devops
description: Troubleshoot and fix Nook CI pipelines (test-nook.yml, Backend.yml, Frontend.yml). Use when CI runs fail, workflows refuse to dispatch, or test reports show 0/0 tests.
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [CI, GitHub Actions, Playwright, Rust, Troubleshooting, API, Docker, Alpine]
    related_skills: [github-pr-workflow]
---

# Nook CI/CI DevOps — Troubleshooting Guide

## Quick Diagnostic Flow

```
1. Check latest run: GitHub API → /actions/runs?branch=develop&per_page=3
2. Identify failed job → failed step number
3. Check YAML for: escape bytes, indentation, continue-on-error misuse
4. Fix via GitHub Contents API (PUT to /contents/.github/workflows/{file})
```

## Stack Technique Actuelle (2026-04-05)

**Toutes les images sont Alpine 3.21 — ZERO dependance Google.**

| Composant | Runtime | Cible build | Size estimée |
|-----------|---------|-------------|-------------|
| Nook backend | `alpine:3.21` | `x86_64-unknown-linux-musl` | ~15-25MB |
| Turn-rs | `alpine:3.21` | `x86_64-unknown-linux-musl` | ~10-15MB |

**Cross-compilation aarch64**: `zig cc 0.13.0` comme linker musl universel.

## Common Issues Found

### Issue: "No jobs were run"
**Cause**: Raw ANSI escape byte (0x1b) embedded in YAML string → GitHub parser rejects entire file.
**Fix**: Search for 0x1b bytes and replace with escaped text `\x1b`:
```bash
python3: content.count(b'\x1b')
content.replace(b'\x1b\[[0-9;]*m//g', b'\x1b\[[0-9;]*m//g')
```

### Issue: Playwright tests run but show 0/0 in TEST_REPORT.md
**Cause**: `--reporter=list --reporter=json` on CLI **overrides** the config's reporter array.
**Fix**: DO NOT add `--reporter` flags. Let `playwright.config.ts` handle reporters.

### Issue: `generate-test-report.py` produces 0/0 even when Playwright ran
**Root causes (in order)**:
1. **`--reporter` CLI flags override config** → remove ALL `--reporter` flags from CLI
2. **`PIPESTATUS` capture error** → `npx playwright test 2>&1 | tee ...; EXIT_CODE=${PIPESTATUS[0]}`

### Issue: workflow_dispatch returns 422
**Cause**: GitHub Actions caches broken workflow definition.
**Workaround**: Push to trigger auto-run instead of manual dispatch.

## API Pattern — Push workflow/file without git clone

```python
import json, base64, urllib.request

token = "YOUR_GITHUB_TOKEN"

# 1. Get SHA
req = urllib.request.Request(
    f"https://api.github.com/repos/MX10-AC2N/Nook/contents/{path}?ref=develop",
    headers={"Authorization": f"Bearer {token}"})
with urllib.request.urlopen(req) as resp:
    sha = json.loads(resp.read().decode())['sha']

# 2. Push
urllib.request.Request(f"https://api.github.com/repos/MX10-AC2N/Nook/contents/{path}",
    data=json.dumps({"message": msg, "content": base64.b64encode(content.encode()).decode(),
                     "sha": sha, "branch": "develop"}).encode(),
    headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    method="PUT")
```

## Playwright Test Architecture & Common Failure Patterns

### Test File Organization
- `api-sanity.spec.ts` — API-level security/CRUD tests using `{ request }` fixture
- `user.spec.ts` — Full user flow: login, chat, chess, polls, calls, calendar
- `admin.spec.ts` — Admin flows: user management, analytics, approvals
- `helpers.ts` — Shared: `loginAs()`, `loginAsAdmin()`, `clearSession()`, constants

### ⚠️ CRITICAL: Playwright `{ page }` Fixture Isolation
Each `describe` block receives its **own** `{ page }` fixture. A page from one describe
has **NO session/cookies** connection to another describe's page.

**Fix**: Use module-scoped `adminPage` with `test.beforeAll`/`test.afterAll`:
```typescript
let adminPage: Page;
test.beforeAll(async ({ browser }) => {
    adminPage = await browser.newPage();
    await loginAsAdmin(adminPage);
});
test.afterAll(async () => { if (adminPage) await adminPage.close(); });
```

### loginAsAdmin() — First-Run + Subsequent-Run
Tries `ADMIN_NEW_PASSWORD` ('AdminCI2026!') first, falls back to `changeme2026`.
After initial login, changes password and re-logins.

### ⚠️ MANDATORY: Validate Before Pushing Test Files

```bash
cd frontend && npx playwright test --list
```
Exit 0 = structurally sound. Exit 1 = fix before pushing.

### Duplicate Test Titles Crash Playwright
If two tests share the same title, Playwright crashes with **zero tests run**.
**Scan**: `python3 -c "from collections import Counter; import re; titles=re.findall(r\"test\\('([^']+)'\", open(f).read()); [print(t,c) for t,c in Counter(titles).items() if c>1]"`

## test-nook.yml — Current Structure (after consolidation)

| Step | Blocking? | Notes |
|------|-----------|-------|
| Frontend Build | ✅ yes | Must pass |
| Backend: cargo fmt/check/test/clippy | ❌ `|| true` | Non-blocking |
| **Shell Integration Tests** | ❌ `continue-on-error: true` | ALL 7 sections in ONE `run:` block |
| **E2E Playwright** | ✅ **yes** | THE critical test, 157 tests |

### ⚠️ $ADMIN_COOKIE Persists ONLY Within Single `run:` Block
GitHub Actions spawns a **fresh shell** for every `- name:` step. Variables defined
in one step are **lost** in the next. That's why the shell integration tests are
all in one consolidated `run:` block — the cookie extracted during admin login
must persist through Security, Chat, WS, Polls, and Events tests.

### Admin Auth Flow (MUST match helpers.ts)
```
1. Login: changeme2026 → needs_password_change=true
2. Change password → AdminCI2026!
3. Re-login → AdminCI2026! → fresh session
4. Verify: GET /auth/me → role=admin
5. Register e2e_ci, approve, run other tests
```
If step 1 fails (password already changed), try AdminCI2026! directly.

### Common Shell Gotchas
- **Nested `$(...)` in curl `-d`**: Extract IDs first, use in next line
- **Heredoc in YAML `run: |`**: `python3 - "$VAR" << 'EOF'` sends bash echo to python stdin. Write file first: `cat > file << 'EOF'` then `python3 file`
- **YAML heredoc at col 0**: Heredoc content inside YAML `run: |` MUST be indented past the block indent, or YAML parser errors

## Musl/Alpine Cross-Compilation — Known Pitfalls

### `rustup target add x86_64-unknown-linux-musl` needs musl-tools FIRST
**Error**: `linker 'cc' not found` when building musl target.
**Fix**: `sudo apt-get install -y musl-tools musl-dev` THEN `rustup target add x86_64-unknown-linux-musl`

### aarch64 musl cross-compilation needs zig
**Error**: No usable linker for `aarch64-unknown-linux-musl` on x86_64 host.
**Fix**: Install zig as universal musl linker:
```bash
curl -sSfL "https://ziglang.org/download/0.13.0/zig-linux-$(uname -m)-0.13.0.tar.xz" \
  | sudo tar -xJ -C /usr/local --strip-components=1
rustup target add aarch64-unknown-linux-musl
# Set RUSTFLAGS: -C linker=/usr/local/bin/zig cc -C link-args=-target aarch64-linux-musl
```

### `rustup target add x86_64-unknown-linux-gnu` corrupts proc-macro
**Error**: `cannot produce proc-macro for asn1-rs-derive`
**Cause**: On `ubuntu-latest`, x86_64-gnu is the native host target. Adding it via rustup
corrupts the proc-macro toolchain. **NEVER** add the host target via rustup.

### .cargo/config.toml for musl targets
For Alpine-compatible builds, use musl targets in `.cargo/config.toml`:
```toml
[target.aarch64-unknown-linux-musl]
linker = "zig cc"
rustflags = ["-C", "link-args=-target aarch64-linux-musl"]
```
x86_64 musl uses musl-gcc natively (no linker override needed in config).

## Backend.yml — Current Configuration

```yaml
strategy:
  matrix:
    target:
      - x86_64-unknown-linux-musl
      - aarch64-unknown-linux-musl

steps:
  # x86_64: musl-tools native, aarch64: zig + musl-tools
  - name: Install cross-compilation tools
    run: |
      sudo apt-get install -y libsodium-dev pkg-config musl-tools xz-utils
      if [ "${{ matrix.target }}" = "aarch64-unknown-linux-musl" ]; then
        curl -sSfL "https://ziglang.org/download/..." | sudo tar -xJ -C /usr/local
      fi
      rustup target add ${{ matrix.target }}
```

## Backend Compilation — Common Rust Errors

### `E0425/E0432/E0433`: Unresolved import
**Fix**: Add the type to the existing import block, don't create standalone imports.

### `E0308/E0382`: Borrow of moved value in `async move`
**Fix**: Clone before each `tokio::spawn(async move { ... })`.

### `sqlx::query!` vs `sqlx::query_as`
`query!` does compile-time table verification. Use `query_as` if tables don't exist
in the build environment.

### Rust CI Error Triage Flow
1. Read from bottom up — first `error:` is the root cause
2. Check `help:` suggestions — Rust compiler often gives exact fix
3. Cancel old CI runs — `POST /runs/{id}/cancel` on stale commits
4. Only the run on HEAD tells the truth

### rustrtc edition = "2024" → ALL rust workflows use @nightly
**NEVER revert to `@stable`** — it will fail on rustrtc.

## Docker Build Architecture — Alpine (Zero Google)

### Nook Backend (Dockerfile)
```
Stage 1: rust:1.88-bookworm + musl-tools
  → cargo build --target x86_64-unknown-linux-musl
Stage 2: alpine:3.21
  → apk add libsqlite3 libsodium ca-certificates
  → COPY binary from stage 1
```

### Turn-rs (services/turn-rs/Dockerfile)
```
Stage 1: rustlang/rust:nightly-bookworm + musl + protobuf
  → git clone + cargo build --target x86_64-unknown-linux-musl
Stage 2: alpine:3.21
  → COPY binary from stage 1
```

### Key Alpine Rules
- Use `musl-tools` for static musl linking (no `+crt-static` needed, musl is inherently static)
- `apk add --no-cache libsqlite3 libsodium ca-certificates` as runtime deps
- Health check: `wget -qO- http://localhost:3000/api/health` (no curl in Alpine by default)
- User: `addgroup -S nook && adduser -S nook -G nook` (alpine nobody = 65534)
- **NEVER reference `gcr.io`, `distroless`, or any Google service**

### turn-rs protobuf build deps
- `protobuf-compiler` + `libprotobuf-dev` for protoc
- `zig` for cross-compilation musl
- `git` for cloning turn-rs sources

## GitHub API Push Patterns

### Always fetch fresh SHA before each push
When pushing multiple files, call GET immediately before PUT. Never reuse SHA.

### CI runs often show errors from BEFORE the latest fix
Push triggers runs on each commit. Check `head_sha` vs current HEAD.
Cancel stale runs: `POST /runs/{id}/cancel`.

### Cancel-in-Progress
The workflow has `cancel-in-progress: true`. Rapid pushes = no complete runs.
Check `GET /runs` → wait if status=in_progress before pushing next fix.

## test-nook.yml Shell Integration — Correct Pattern

All integration tests in ONE `run:` block so variables persist:
```yaml
- name: Shell Integration Tests
  continue-on-error: true
  run: |
    # 1. AUTH (extracts ADMIN_COOKIE)
    curl ... login → ADMIN_COOKIE=...
    # 2. SECURITY (uses ADMIN_COOKIE from step 1)
    curl -H "Cookie: $ADMIN_COOKIE" ...
    # 3. CHAT (uses ADMIN_COOKIE from step 1)
    curl -H "Cookie: $ADMIN_COOKIE" ...
    # 4. WS, 5. POLLS, 6. EVENTS ...
```

## Report Generation — External Scripts Only

All CI reports live in `.github/scripts/`:
- `generate-frontend-report.sh`
- `generate-backend-report.sh`
- `generate-docker-report.sh`
- `generate-test-report.py`

**NEVER** keep report generation as inline `run: |` blocks > 30 lines.
Scripts must NOT contain `${{ }}` — pass vars via `env:` in workflow.

## Svelte 5 Template Rules

- `<tr>` must be in `<tbody>`, `<thead>`, or `<tfoot>` — never direct child of `<table>`
- NEVER inject imports between `<script` and its attributes — collapse tag first
- Remove duplicate imports before adding new ones
- Verify: `{}` balance = 0, no orphaned blocks