# Project Conventions — Nook

## Commit & Branch Naming
- **Commit Format**: `<type>(<scope>): <description>` (ex: `fix(backend): suppress clippy warnings`, `feat(frontend): add dark mode toggle`)
- **Branch Naming**: `fix/xxx`, `feat/xxx`, `refactor/xxx`, `docs/xxx`, `test/xxx`
- **Examples (Session 50)**: PR #28 `refactor/remove-simple-peer`, PR #29 `feat/healthchecks`, PR #30 `fix/hardcoded-secrets`

## Code Style
### Rust / Backend
- Axum 0.8 syntax: `{param}` (not `:param`)
- rand 0.9: Use `rng()` not `thread_rng()`
- Always run `cargo check` + `clippy` before pushing
- Fix commits only touch the reported bug, no dependency version changes

### Svelte 5 / Frontend
- Strict Runes: `$state`, `$derived`, `$derived.by`, `$effect`
- Mandatory MCP Svelte + autofixer before delivery
- No direct reassignment on `$state`
- SVG icons only (no emojis in code)

## Security & Privacy
- **Zero hard-coded secrets**: TURN_SECRET, admin passwords, etc. in `.env` only
- E2EE for messages, files encrypted at rest
- Principle of least privilege everywhere

## CI/CD Rules
- **No scheduled workflows** (free GitHub account)
- Manual trigger only, order: Frontend → Backend → Turn → Docker
- Always check `git log --oneline -5` + `gh run list --limit 5` before triggering
- Multi-arch builds (amd64 + arm64) mandatory for all releases

## Testing
- Playwright for E2E tests (mandatory for new features)
- Rust unit tests for backend logic
- MCP Svelte autofixer validation for all frontend changes
