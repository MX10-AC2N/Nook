---
name: nook-testing-specialist
description: Skill for the Testing Specialist agent - Unit/integration/E2E testing, test automation, coverage reports, TDD workflows for Nook
---

# Nook Testing Specialist Skill

Use this skill when asked to write tests, set up test automation, audit test coverage, or implement TDD workflows for Nook.

## Responsibilities
- Write unit tests for Rust (backend, chess engine, turn-server) and TypeScript (frontend)
- Set up integration tests for API endpoints, WebSocket signaling, E2EE flows
- Configure E2E tests for WebRTC calls, calendar, polls, chess features
- Generate coverage reports (cargo tarpaulin, vitest coverage)
- Enforce TDD workflows (write test first, then implementation)
- Audit existing tests, identify gaps

## Tools Required
- `cargo test` (Rust unit/integration tests)
- `vitest` (Frontend unit tests)
- `playwright` or `cypress` (E2E tests, optional)
- `cargo tarpaulin` (Rust coverage)
- `npm run test` (Frontend tests)

## Key Test Areas
1. **Backend (Rust/Axum)**:
   - Auth endpoints (login, register, token validation)
   - API endpoints (messages, polls, chess, WebRTC)
   - E2EE encryption/decryption flows
   - Database migrations, CRUD operations
   - Rate limiting, auth middleware

2. **Frontend (Svelte 5)**:
   - Stores (auth, chess, poll, calendar)
   - Components (forms, modals, chat UI)
   - E2EE client-side encryption
   - PWA service worker registration
   - Accessibility (WCAG 2.1 AA)

3. **Special Features**:
   - Chess engine SAN/PGN export, AI difficulty
   - WebRTC signaling, ICE config fetch
   - Poll creation, voting, expiration
   - Calendar event CRUD

## Test Standards
- Rust: Minimum 80% coverage for backend crates
- Frontend: Minimum 70% coverage for stores/utils
- All PRs must include tests for new functionality
- E2E tests for critical user flows (login, send message, start call)

## Example Workflow
1. Load this skill: `skill_view("nook-testing-specialist")`
2. For new feature: Write failing test first (TDD)
3. Implement feature to pass test
4. Run `cargo test` (backend) or `npm run test` (frontend)
5. Generate coverage report: `cargo tarpaulin --out html`
6. Commit tests with feature code

## Pitfalls
- WebRTC E2E tests require TURN server running
- E2EE tests need valid key pairs, use test fixtures
- Chess engine tests should cover all SAN disambiguation cases
- Database tests should use isolated test SQLite databases

## Verification
- [ ] All new features have unit tests
- [ ] Coverage reports meet minimum thresholds
- [ ] CI workflows run tests on every PR
- [ ] No skipped/ignored tests without justification
