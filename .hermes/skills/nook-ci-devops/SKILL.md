---
name: nook-ci-devops
description: Troubleshoot and fix Nook CI pipelines (test-nook.yml, Backend.yml, Frontend.yml). Use when CI runs fail, workflows refuse to dispatch, or test reports show 0/0 tests.
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [CI, GitHub Actions, Playwright, Rust, Troubleshooting, API, Docker]
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

## Stack Technique Actuelle (2026-04-05 — APRES rollback Alpine)

**Toutes les images sont `debian:bookworm-slim` — ZERO dependance Google.**

| Composant | Runtime | Cible build | Base |
|-----------|---------|-------------|------|
| Nook backend | `debian:bookworm-slim` | `x86_64-unknown-linux-gnu` | Debian |
| Turn-rs | `debian:bookworm-slim` | `x86_64-unknown-linux-gnu` | Debian |

**⚠️ LECON ALPINE (2026-04-05 session)** :
La migration musl+Alpine a cause 2h de bugs en cascade (ring/aws-lc-sys CC,
x86_64-linux-musl-gcc manquant, CC env vars, indentation YAML).
**debian:bookworm-slim** est le bon compromis: glibc native (pas de CC hacks),
~80MB mais zero Google, stable, testé. NE PAS re-tenter Alpine/musl.

## Common Issues Found

### Issue: "No jobs were run"
**Cause**: Raw ANSI escape byte (0x1b) embedded in YAML string.
**Fix**: Replace `0x1b` with `\x1b` in the YAML string.

### Issue: Playwright tests run but show 0/0 in TEST_REPORT.md
**Cause**: `--reporter=list --reporter=json` on CLI overrides config.
**Fix**: DO NOT add `--reporter` flags. Let `playwright.config.ts` handle reporters.

### Issue: workflow_dispatch returns 422
**Workaround**: Push to trigger auto-run instead of manual dispatch.

## API Pattern — Push workflow file without git clone

```python
import json, base64, urllib.request

token = "YOUR_GITHUB_TOKEN"

# 1. Get SHA (ALWAYS fetch fresh before each push)
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

## test-nook.yml — Current Structure (after consolidation)

| Step | Blocking? | Notes |
|------|-----------|-------|
| Frontend Build | ✅ yes | Must pass |
| Clean + mkdir + chmod | ✅ yes | Ensures bind mount perms |
| Build Docker image | ✅ yes | |
| Start stack | ✅ yes | Healthcheck loop 60s |
| **E2E Playwright** | ✅ **yes** | THE critical test, 157 tests |

### ⚠️ $ADMIN_COOKIE Persists ONLY Within Single `run:` Block
GitHub Actions spawns a **fresh shell** for every `- name:` step. Variables defined
in one step are **lost** in the next. Shell integration tests are each in their own
`run:` block because they don't share variables (each re-does auth if needed).

### Admin Auth Flow (MUST match helpers.ts)
```
1. Login: changeme2026 → needs_password_change=true
2. Change password → AdminCI2026!
3. Re-login → AdminCI2026! → fresh session
4. Verify: GET /auth/me → role=admin
5. Register e2e_ci, approve, run other tests
```

### Common Shell Gotchas
- **Nested `$(...)` in curl `-d`**: Extract IDs first, use in next line
- **`mkdir -p data logs && chmod 777 data logs`** before docker compose up — avoids PermissionDenied with bind-mounted volumes
- **`docker compose down -v`** before build — removes stale volumes

## Docker Build Architecture — debian:bookworm-slim (Zero Google)

### Nook Backend (Dockerfile)
```
Stage 1: rust:1.88-bookworm
  → cargo build --release (glibc, +crt-static)
Stage 2: debian:bookworm-slim
  → apt-get install libsqlite3-0 libsodium23 ca-certificates
  → RUN addgroup --system nook && adduser --system nook
  → COPY binary from stage 1
  → USER nook
```

### Turn-rs (services/turn-rs/Dockerfile)
```
Stage 1: rustlang/rust:nightly-bookworm
  → git clone + cargo build --release
Stage 2: debian:bookworm-slim
  → COPY binary from stage 1
```

### Key debian:bookworm-slim Rules
- **Package names**: `libsqlite3-0`, `libsodium23` (Debian names, not Alpine `sqlite-libs`)
- **User setup**: `addgroup --system nook && adduser --system --ingroup nook nook` + `chown -R nook:nook /app`
- **NEVER use `gcr.io`, `distroless`, or any Google service**
- Health check: `wget -qO- http://localhost:3000/api/health`

## Backend.yml — Current Configuration (gnu, NOT musl)

```yaml
strategy:
  matrix:
    target:
      - x86_64-unknown-linux-gnu    # Native host target
      - aarch64-unknown-linux-gnu   # Cross-compile

steps:
  - name: Install cross-compilation tools
    run: |
      sudo apt-get install -y libsodium-dev pkg-config
      if [ "${{ matrix.target }}" = "aarch64-unknown-linux-gnu" ]; then
        sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
        rustup target add aarch64-unknown-linux-gnu  # cross-compilation only
      fi
      # DO NOT add x86_64 — it's the native host target
```

### ⚠️ `rustup target add x86_64-unknown-linux-gnu` corrupts proc-macro
**Error**: `cannot produce proc-macro for asn1-rs-derive`
**Cause**: On `ubuntu-latest`, x86_64-gnu is the native host target.
DO NOT add it via rustup. Only add `aarch64-unknown-linux-gnu`.

### `.cargo/config.toml` (gnu targets)
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
# NO x86_64 section — native target, no linker needed
```

## GitHub API Push Patterns

### Always fetch fresh SHA before each push
When pushing multiple files, call GET immediately before PUT. Never reuse SHA.

### CI runs often show errors from BEFORE the latest fix
Push triggers runs on each commit. Check `head_sha` vs current HEAD.
Cancel stale runs: `POST /runs/{id}/cancel`.

### Cancel-in-Progress
Workflow has `cancel-in-progress: true`. Rapid pushes = no complete runs.
Wait for current run to finish before pushing next fix.

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