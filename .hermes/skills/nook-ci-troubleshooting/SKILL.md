---
name: nook-ci-troubleshooting
category: devops
description: Troubleshooting common CI errors in Nook — cargo config, toolchain, cache, and Playwright scope issues
---

# Nook CI Troubleshooting

## Cargo config + proc-macros issue

**.cargo/config.toml with `+crt-static` breaks proc-macro compilation on x86_64 CI**:
- The `[target.x86_64-unknown-linux-gnu]` section with `rustflags = ["-C", "target-feature=+crt-static"]` prevents proc-macro compilation in CI
- Error: `cannot produce proc-macro for asn1-rs-derive ... as the target x86_64-unknown-linux-gnu does not support these crate types`
- **Fix**: Remove the `[target.x86_64-unknown-linux-gnu]` section from `.cargo/config.toml`. Keep only `[target.aarch64-unknown-linux-gnu]` for cross-compilation.
- The `+crt-static` flag is already passed via `RUSTFLAGS` env var in `Backend.yml` for release builds only.

## rustup target add corrupts toolchain

**Adding native target during CI corrupts proc-macro compilation**:
- In `test-nook.yml`, `rustup target add x86_64-unknown-linux-gnu` corrupts the nightly toolchain
- Error: same `cannot produce proc-macro` error with `asn1-rs-derive`
- **Fix**: Only add non-native targets (aarch64). x86_64 is already the host target on ubuntu-latest.
- In `test-nook.yml`, remove `rustup target add` entirely (no cross-compilation needed).
- In `Backend.yml`, only add `aarch64-unknown-linux-gnu` for the arm64 matrix job.

## Rust cache corruption in Swatinem/rust-cache

**After `.cargo/config.toml` or toolchain changes, stale cache causes build errors**:
- Symptoms: unexpected compile errors on previously working code, proc-macro failures
- **Fix 1**: Change the `prefix-key` value in the rust-cache step (e.g., `v1-rust` → `v2-rust`)
- **Fix 2**: Delete the cache manually via GitHub Actions UI → Actions → Caches → Delete cache

## Rust edition 2024 requires nightly

**rustrtc 0.3.40 uses `edition = "2024"` which requires Rust >= 1.85**:
- Available only on `nightly` channel (not stable)
- **All** workflows that compile the backend must use `dtolnay/rust-toolchain@nightly`
- Affected: `Backend.yml`, `test-nook.yml`, `sqlx-prepare.yml`

## Playwright adminPage scope issue in admin.spec.ts

**`adminPage` ReferenceError in sibling describe blocks**:
- If `let adminPage: Page` is declared inside `test.describe.serial('Admin — Flux complet')`, sibling describe blocks (`Admin — Complément`, `Admin — Analytics`) cannot access it
- Error: `ReferenceError: adminPage is not defined`
- **Fix pattern**:
  1. Move `let adminPage: Page;` to module scope (after imports, before first describe)
  2. Move `test.beforeAll(async ({ browser }) => { ... })` to module scope
  3. Move `test.afterAll(async () => { ... })` to module scope
  4. Remove duplicate `let adminPage`, `test.beforeAll`, `test.afterAll` from inside any describe blocks

## Shell variables lost between GitHub Actions `run:` steps

**`$ADMIN_COOKIE` (or any shell variable) is EMPTY in steps after the one that defines it**:
- GitHub Actions does NOT persist shell variables between separate `run:` blocks
- Symptoms: All curl commands after auth step return `{"message":"Non authentifié","success":false}`
- Bash errors like `syntax error near unexpected token '('` from broken nested `$()` in `-F` flags
- **Fix**: Consolidate ALL shell test steps that share variables into a SINGLE `run: |` block
- In `test-nook.yml`: merged 6 separate `run:` blocks (Test Auth, Test Security, Test Chat, Test WebSocket, Test Polls, Test Calendar/Events) into one "Shell Integration Tests" block

## Python heredoc inside YAML `run: |` block breaks YAML parsing

**`python3 - "arg" << 'EOF'` at column 0 causes YAML error**:
- Inside a YAML `run: |` block, content MUST be indented consistently
- A heredoc with Python code starting at column 0 breaks YAML block scalar parsing
- Error: `Invalid workflow file: ...You have an error in your yaml syntax on line XX`
- **Fix pattern 1** (simple): Use `python3 -c '...'` inline
- **Fix pattern 2** (multi-line): `cat > /tmp/script.py << 'EOF'` with indented content, then `python3 /tmp/script.py "arg"`
- **Fix pattern 3** (bash arg): `python3 - "$WS_COOKIE" << 'EOF'` BUT indent the heredoc content to match YAML block minimum (10+ spaces), and the `EOF` terminator at same min indentation

## Playwright flaky test: auth session expires between tests

**Chess resign and GET analytics return 401 intermittently**:
- `api-sanity.spec.ts:345` Chess resign → `request.post()` without auth cookie context
- `admin.spec.ts:468` GET /analytics → `adminPage.request.get()` session expired due to timing
- These are **flaky** (pass on retry), not hard failures
- Root cause: Playwright `request` fixture uses `storageState` which can become stale
- Mitigation: `continue-on-error: true` in workflow is acceptable for these tests

## Docker PermissionDenied (code: 13) on container startup

**Container crashes with `Error: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }`**:
- Happens when `nook` user (UID 1000) can't write to `/app/data` or `/app/logs`
- Root cause: Volume mount directories created by Docker as root override the `chown` in Dockerfile
- **Symptom**: Container loads config, prints CORS origins, then crashes before DB init
- **Check**: Do CI workflow `mkdir` names match `docker-compose.yml` volume mount paths?
- **Fix**: CI creates `data/` and `logs/` but docker-compose defaults to `./nook-data` and `./nook-logs`
  ```yaml
  # docker-compose.yml defaults
  volumes:
    - "${DATA_DIR:-./nook-data}:/app/data"
    - "${LOGS_DIR:-./nook-logs}:/app/logs"
  ```
  CI must create matching directories:
  ```yaml
  - name: Clean previous runs
    run: |
      rm -rf nook-data nook-logs
      mkdir -p nook-data nook-logs
      chmod 0777 nook-data nook-logs
  ```

## Cargo.lock out of sync after Cargo.toml changes

**Docker build with `--locked` fails but CI backend build succeeds**:
- Backend build doesn't use `--locked` → succeeds even with stale lockfile
- Docker build uses `cargo build --release --locked` → fails if lockfile doesn't match
- Error: `error: the lock file ... needs to be updated but --locked was passed`
- **Fix sequence**:
  1. Update Cargo.toml with new deps
  2. Trigger `update-cargo-lock.yml` workflow
  3. Trigger `sqlx-prepare.yml` if new SQL queries added
  4. Then trigger Backend + Docker builds
- Or run locally: `cargo update -p <package>` then commit Cargo.lock

## git push conflict in CI workflows

**Multiple CI workflows push to `.claude/TEST_REPORT.md` causing conflicts**:
- Error: `! [rejected] develop -> develop (fetch first)` or `failed to push some refs`
- Root cause: Workflow A pushes, then Workflow B tries to push on stale HEAD
- **Fix**: Add `git fetch + rebase` before push:
  ```yaml
  - name: Commit report
    if: always()
    run: |
      git config user.name "github-actions[bot]"
      git config user.email "github-actions[bot]@users.noreply.github.com"
      git add .claude/REPORT.md
      if git diff --staged --quiet; then
        echo "No changes"
      else
        git fetch origin ${{ github.ref_name }}
        git rebase origin/${{ github.ref_name }} || git reset origin/${{ github.ref_name }}
        git add .claude/REPORT.md
        git commit -m "ci: report [run ${{ github.run_id }}]"
        git push --force-with-lease origin HEAD:${{ github.ref_name }}
      fi
  ```

## pnpm vs npm mismatch in workflows

**Workflow references pnpm-lock.yaml but project uses package-lock.json**:
- Error: `ERR_PNPM_NO_LOCKFILE` or cache miss
- Symptoms: `npm audit` runs but `pnpm install` step exists, weekly audits never run
- **Fix**: Replace all pnpm references with npm:
  ```yaml
  # ❌ Wrong
  cache: 'pnpm'
  cache-dependency-path: frontend/pnpm-lock.yaml
  run: pnpm install --frozen-lockfile

  # ✅ Correct
  cache: 'npm'
  cache-dependency-path: frontend/package-lock.json
  run: npm ci
  ```

## Playwright E2E flaky: WebSocket-dependent UI updates

**Test sends message via POST but message doesn't appear in DOM**:
- UI relies on WebSocket broadcast to show new messages
- In CI, WS connection may be slow or not established
- Error: `expect(locator).toBeVisible() failed` after 15s timeout
- **Fix**: Add optimistic update — push data to local state immediately after API call:
  ```typescript
  // In sendMessage function
  const res = await fetch(`/api/conversations/${id}/messages`, { method: 'POST', ... });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  // Optimistic update: add locally immediately
  const responseData = await res.json();
  if (responseData?.id && !messages.some(m => m.id === responseData.id)) {
    messages = [...messages, responseData];
  }
  ```
- WS will reconcile with server data when it arrives
