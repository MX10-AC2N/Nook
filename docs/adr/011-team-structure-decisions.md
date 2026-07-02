# ADR-011: Team Nook — 16-Profile Agent Structure Decisions

**Date**: 2026-07-02
**Status**: Accepted
**Owner**: architect
**Participants**: orchestrator, team-upgrader, supervisor

---

## Context

Nook is a self-hosted private family messaging platform (E2EE chat, WebRTC calls, calendar, chess, polls, themes, push notifications, single Docker container). As the project grew in complexity, the original 7-profile team (orchestrator + 6 workers) became insufficient to cover all technical domains with adequate depth.

Key drivers for expansion:
1. **Technical debt accumulation** (DT-01..07) required dedicated ownership
2. **Security/crypto complexity** (E2EE rotation, rate limiting) needed specialist focus
3. **Release engineering** (5-workflow CI/CD pipeline) needed dedicated enforcement
4. **Performance budget** (libsodium 938kB, WebRTC latency) needed continuous profiling
5. **Accessibility compliance** (WCAG 2.1 AA, Svelte 5 runes) needed review gates
6. **Supply chain security** (rand_core 0.6 pin, Dependabot, CVEs) needed active management
7. **Token budget governance** (174K daily, 5M ceiling) needed enforcement
8. **Team lifecycle management** (profile create/update/deprecate) needed automation

---

## Decision

Expand from 7 to **16 active profiles** with the following structure:

### Core Profiles (Original 7)
| Profile | Role | Daily Budget | Key Responsibility |
|---------|------|--------------|-------------------|
| orchestrator | Coordinator | 10,000 | Task decomposition, delegation, synthesis |
| coder | Rust/Svelte dev | 50,000 | Backend (Axum 0.8, SQLx) + Frontend (SvelteKit 5) |
| tester | E2E Playwright | 20,000 | 156 E2E tests, unit/integration, mobile/PWA |
| researcher | WebRTC/E2EE/Rust | 15,000 | WebRTC/SFU, Double Ratchet, crypto research |
| github-manager | PR/Release/GHCR | 5,000 | PR lifecycle, releases, GHCR, dawidd6 |
| ci-monitor | GitHub Actions | 5,000 | 22 workflows, Docker, musl |
| deployer | Homeserver/Docker | 5,000 | Zimaboard (192.168.1.192), Docker compose, rollback |

### Extended Profiles (Added 2026-07-01)
| Profile | Role | Daily Budget | Key Responsibility | DT Ownership |
|---------|------|--------------|-------------------|--------------|
| architect | System design | 15,000 | ADRs, tech debt DT-01..07, design reviews | DT-02, DT-05 (co-own) |
| security-auditor | E2EE/Crypto/Threats | 10,000 | E2EE audit, threat model, crypto hygiene | DT-04 (rate limiting) |
| docs-writer | Documentation | 8,000 | SOUL.md, ARCHITECTURE.md, CHANGELOG, API docs | — |
| release-manager | Version/Changelog | 3,000 | 5-workflow pipeline (FE→BE→Turn→Docker→Release) | — |
| perf-engineer | Perf/Profiling | 10,000 | Bundle, WebRTC latency, SQLx perf | **DT-01 (libsodium)** |
| ux-reviewer | UX/A11y | 5,000 | WCAG 2.1 AA, theme system, Svelte 5 runes | DT-02 (chess realtime) |
| dependency-manager | Deps/Security | 3,000 | Cargo/pnpm, Dependabot, CVEs, rand_core 0.6 pin | DT-06 (analytics) |
| supervisor | Token budget | 10,000 | Daily/monthly limits, rate limit monitoring, auto-throttle | — |
| team-upgrader | Profile lifecycle | 5,000 | Create/update/deprecate profiles, TEAM_REGISTRY.md | — |

### Budget Summary
- **Total Daily**: 174,000 tokens
- **Total Monthly**: 5,220,000 tokens
- **Global Ceiling**: 5,000,000/day | 150,000,000/month

---

## Technical Debt Ownership Mapping

| DT ID | Description | Owner(s) | Status |
|-------|-------------|----------|--------|
| DT-01 | libsodium 938 kB — no dynamic import | perf-engineer | 🔴 Active |
| DT-02 | Chess not realtime | architect + ux-reviewer | 🔴 Active |
| DT-03 | Polls backend localStorage only | coder | 🟡 Active |
| DT-04 | Rate limiting governor not configured | security-auditor | 🟡 Active |
| DT-05 | E2EE partially implemented | architect + security-auditor | 🟡 Active |
| DT-06 | Analytics endpoint incomplete | dependency-manager + coder | 🟢 Active |
| DT-07 | Bug state_invalid_export conversationStore | coder + ux-reviewer | 🟢 Active |

---

## Delegation Map

```
orchestrator
├── coder (Rust backend, Svelte frontend)
├── tester (E2E 156 tests)
├── researcher (WebRTC, E2EE, crypto, Rust)
├── github-manager (PRs, releases, GHCR)
├── ci-monitor (22 GitHub Actions)
├── deployer (homeserver 192.168.1.192)
├── architect (design decisions, DT-01..07)
├── security-auditor (E2EE audit, threat model)
├── docs-writer (SOUL, ARCH, CHANGELOG)
├── release-manager (version, tag, GHCR)
├── perf-engineer (bundle, WebRTC, SQLx)
├── ux-reviewer (a11y, mobile, themes)
├── dependency-manager (cargo, pnpm, advisories)
├── supervisor (token budget enforcement)
└── team-upgrader (profile lifecycle)

supervisor (can delegate to)
├── team-upgrader (patch profile configs)
└── orchestrator (pause/resume agents)

team-upgrader (can delegate to)
├── (creates/updates profiles directly via file ops)
```

---

## MCP Server Assignments (Global: 7 Active)

| Server | Profiles Using |
|--------|----------------|
| codegraph | orchestrator, coder, tester, researcher, architect, security-auditor, perf-engineer, supervisor, team-upgrader |
| github | orchestrator, coder, researcher, github-manager, ci-monitor, deployer, docs-writer, release-manager, dependency-manager, supervisor, team-upgrader |
| playwright | orchestrator, coder, tester, ux-reviewer, supervisor, team-upgrader |
| filesystem | all |
| svelte | orchestrator, coder, supervisor, team-upgrader |
| svelte-docs | orchestrator, coder, supervisor, team-upgrader |
| lightpanda | orchestrator, coder, tester, deployer, ux-reviewer, supervisor, team-upgrader |

Disabled: sqlite (DB in Docker on 192.168.1.192), docker (daemon not available), rust-analyzer (not installed)

---

## Profile Configuration Standard

All 16 profiles follow this pattern:
- `config.yaml` — Profile config with token_budget entry
- `.env` — Environment with required secrets (TELEGRAM_BOT_TOKEN, GITHUB_TOKEN, OPENCODE_ZEN_API_KEY, API_SERVER_KEY)
- `SOUL.md` — Profile-specific identity, rules, Nook context, pushback triggers
- Registered in `TEAM_REGISTRY.md` (single source of truth)
- Token budget added to `supervisor/config.yaml` token_budget.per_profile_limits
- Test spawn validation: `hermes run --profile <name> "echo test"`

---

## Consequences

### Positive
- **Domain expertise**: Each technical area has dedicated ownership
- **Parallel throughput**: Up to 14 leaf agents can work simultaneously (max_concurrent_children=3 for orchestrator)
- **Clear accountability**: DT-01..07 have explicit owners
- **Budget governance**: Supervisor enforces ceilings with auto-throttle
- **Lifecycle automation**: Team-upgrader maintains registry and configs
- **Security focus**: Security-auditor + dependency-manager cover crypto + supply chain

### Negative
- **Coordination overhead**: Orchestrator must decompose more granularly
- **Token budget pressure**: 174K daily requires active monitoring
- **Profile sync risk**: Config drift across 16 profiles needs team-upgrader vigilance
- **Gateway complexity**: 16 Telegram bots + topics to maintain

### Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Profile proliferation | team-upgrader pushback triggers: consolidate before create |
| Budget overrun | supervisor auto-throttle at 80% (WARN → reduce max_tokens → switch to haiku → pause) |
| Config drift | team-upgrader validates all profiles on create/update |
| Silent failures | Kanban dispatch + 📊 reporting to orchestrator topic mandatory |

---

## Related Decisions
- **ADR-001**: Single Docker container deployment model
- **ADR-005**: E2EE Double Ratchet implementation
- **ADR-007**: 5-workflow CI/CD order (FE→BE→Turn→Docker→Release)
- **ADR-010**: Svelte 5 runes only (no `$:` syntax)

---

## Implementation Checklist

- [x] 16 profile directories created under `/root/.hermes/profiles/`
- [x] `config.yaml` for each profile with token_budget entry
- [x] `.env` for each profile with required secrets
- [x] `SOUL.md` for each profile with Nook context + pushback triggers
- [x] `TEAM_REGISTRY.md` updated with all 16 profiles
- [x] `supervisor/config.yaml` token_budget.per_profile_limits updated
- [x] Kanban board `nook` initialized with shared DB
- [x] GitHub Dependabot config created (`.github/dependabot.yml`)
- [x] Test spawn validation for all profiles
- [ ] Weekly dependency scan cron job (dependency-manager + supervisor)
- [ ] ADR-011 committed to repo

---

## Review Schedule

| Trigger | Reviewer | Action |
|---------|----------|--------|
| Quarterly | architect + team-upgrader | Assess profile relevance, consolidate if needed |
| Budget > 80% monthly | supervisor | Alert, investigate top consumers |
| New DT identified | architect | Assign owner, update registry |
| Profile unused > 30 days | team-upgrader | Deprecate or merge |
| Major version upgrade | all | Update model/toolsets in config.yaml |