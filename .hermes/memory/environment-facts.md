# Environment Facts — Nook Development

## GitHub & Repo
- **Repo**: `MX10-AC2N/Nook` (https://github.com/MX10-AC2N/Nook)
- **Branch**: `develop` (working branch, all changes pushed here)
- **GitHub Token**: Valid, push OK (updated 2026-05-02)
- **Workflow Order**: NEVER schedule workflows (free account). Manual trigger only, order: Frontend → Backend → Turn → Docker.
- **Workflow Check**: Always run `git log --oneline -5` + `gh run list --limit 5` before triggering workflows.

## Local Environment
- **Persistent Volume**: `/opt/data` (Docker startup volume, persists between sessions)
- **Repo Location**: `/opt/data/repos/Nook/` (cloned, branch `develop`)
- **Hermes Extension Directory**: `/opt/data/repos/Nook/.hermes/` (operational files, pushed to GitHub)
- **Skills Directory**: `/opt/data/repos/Nook/.hermes/skills/` (Nook-specific skills stored here)

## Nook Current Status (2026-05-06)
- **Backend**: 🟡 Build IN_PROGRESS (x86_64 OK, arm64 compiling). Clippy warnings to fix.
- **Frontend**: ✅ Build OK (commit 834f5b62 fixed +layout.svelte & package-lock).
- **Docker**: ⏳ Waiting for Backend+Frontend artifacts.
- **Deployed**: 🔴 Unhealthy (Axum 0.8 panic in events.rs:316, fixed in code, needs new build).
- **Test URL**: http://192.168.1.192:6300 | https://192.168.1.192:6443
- **Credentials**: hermes-bot / Hermes2026!

## Tool Quirks
- `read_file` truncates output at ~100K characters (use offset/limit for large files)
- `patch` tool's built-in linter gives false positives for Rust edition (ignorer les erreurs `E0670` si Cargo.toml a `edition = "2021"`)
- `gh run list` returns full JSON, use `| head -N` or `| grep` to filter
- `cargo build` for musl targets requires `musl-tools` + `RUSTFLAGS="-C linker=musl-gcc"` (no `-C target=` in RUSTFLAGS)
