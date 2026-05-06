# User Preferences — MX10-AC2N

## Identity & Context
- **Name/Pseudo**: MX10-AC2N
- **Global Goal**: Build the best possible self-hosted family messaging platform — private, robust, enjoyable, no compromises on security or UX.
- **Work Standard**: Very high. Prefers slow but impeccable work over fast and approximate.
- **Language**: Always respond in clear, professional French.

## Work Style
- **"Sois méticuleux"**: Deep analysis before any modification. List potential impacts.
- **"On continue"**: Keep momentum, don't stop on minor details.
- **"On reprend"**: If something is broken, restart from a clean base instead of patching.
- **Direct action**: Can create commits, PRs, push directly (to develop).
- **Reports**: Always include raw logs (CI, errors, cargo check) + clear diagnosis.
- **Update frequency**: Update `active-session.md` and memory after each important action.

## Non-Negotiable Technical Rules
### Rust / Backend
- Always `cargo check` + `clippy` before push.
- Never modify dependency versions in a fix commit.
- A fix commit only touches the reported bug.
- rand 0.9 → `rng()` not `thread_rng()`.
- Axum 0.8: current syntax (`{param}` etc.).

### Svelte 5 / Frontend
- Strict respect of Runes (`$state`, `$derived`, `$derived.by`, `$effect`).
- Mandatory use of MCP Svelte + autofixer before delivery.
- No direct reassignment on `$state`.

### CI/CD & DevOps
- Commit first → rebase for lockfiles if needed.
- One atomic commit per clear fix.
- Always check multi-arch builds (amd64 + arm64).

## Anti-Loop & Vigilance Points
- If user says "stop", "change", "nouveau" or "reset" → immediate stop of current plan, ask for new objective confirmation.
- After 2 failed attempts on same approach → propose alternative or ask for instructions.
- Always reread first: `preferences.md` + `known-issues.md` + `active-session.md` + `project-state.md`.

## UX & Design
- Prioritize simplicity and clarity (family = all ages).
- Use SVG icons (no emojis in code).
- Carefully designed themes, mandatory dark mode, mobile-first responsive.

## Current Priorities (update regularly)
1. Stability & reliability (E2EE, calls, persistence)
2. Family user experience
3. Performance on low hardware (AMD64 and ARM64)
4. Security & privacy compliance

## Security & Privacy
- Zero hard-coded secrets.
- Maximum encryption (E2EE messages, files encrypted at rest).
- Principle of least privilege everywhere.
