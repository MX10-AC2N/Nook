# ADR-015 : sqlx 0.8 → 0.9 — migration bloquée (compat Axum 0.8)

**Date** : 2026-08-07 (rédaction d'origine) · **Status** : Proposé (bloqué — spike coder) · **Supersede** : —
**Owner** : architect · **Recréé** : 2026-10-22 (carte `t_566014c5`)

> **Provenance.** Fichier orphelin `0015-sqlx-08-to-09-migration.md`, rédigé le 2026-08-07
> (carte `t_dd44fc37`), **jamais commité**. Recréé ici au nom normalisé `015-sqlx-08-to-09-migration.md`
> (convention `docs/adr/NAMING.md`), corps repris tel quel, statut corrigé vers le statut réel.
> Décision de recréation : `@architect`, carte `t_566014c5`.

> **Preuve d'état (2026-10-22).** `backend/Cargo.toml:17` = `sqlx 0.8.6` ; `backend/Cargo.lock` = `sqlx 0.8.6`.
> Le spike de compatibilité sqlx 0.9 / Axum 0.8 n'a jamais été lancé : la décision n'a pas abouti.

> **Collision de numérotation.** Le fichier orphelin portait le numéro **015** alors que le commentaire
> de l'Architecte sur `t_dd44fc37` (2026-08-07) l'attribuait au **014**. La chronologie et le **titre du fichier**
> l'emportent : **015 = sqlx**. Le numéro 014 reste réservé à la migration rand_core/argon2. Voir
> `docs/adr/README.md` §4.4 et `docs/adr/HISTORY.md`.

## Context

`sqlx` is pinned at `=0.8` in `Cargo.toml` (line 18) for Axum 0.8 compatibility. The project uses Axum 0.8.9 (latest 0.8.x, line 11).

sqlx 0.9.0 was released with breaking changes that may affect:
- Axum integration (extractors, middleware)
- Query macros (`sqlx::query!`, `sqlx::query_as!`)
- SQLite driver (`sqlx-sqlite`)
- Migration API (`sqlx::migrate!`)

Current state:
- `sqlx` 0.8.6 (pinned)
- `axum` 0.8.9 (latest 0.8.x)
- `sqlx-sqlite` 0.8.6 (transitive)

## Decision

**Require architect approval before upgrading sqlx to 0.9.**

### Rationale

1. **Axum 0.8 → 0.9 is a separate major upgrade** — sqlx 0.9 may require Axum 0.9 for full compatibility
2. **Breaking changes in sqlx 0.9** (from changelog):
   - `Row::get` signature changes
   - `Executor` trait changes
   - `Pool` configuration API changes
   - SQLite: `SqliteConnectOptions` changes
   - Macros: `query!` may have different type inference
3. **Migration scope**: All database files (`db.rs`, `auth.rs`, `e2ee.rs`, `webrtc.rs`, `chess.rs`, `polls.rs`, `upload.rs`, `calendar.rs`, `ws.rs`)
4. **Test coverage**: 156 E2E tests must pass — database layer is critical path

## Consequences

### Positive (after migration)
- sqlx 0.9 has performance improvements (query caching, connection pooling)
- Better async trait support (aligns with Rust 2024 edition)
- Security fixes in transitive dependencies
- Unblocks future Axum 0.9 upgrade path

### Negative / Risks (current blocked state)
- Pinned at 0.8.6 — missing 0.8.7+ patch fixes
- sqlx 0.8 will eventually stop receiving security updates
- Transitive deps (libsqlite3-sys 0.30→0.37, flume 0.11→0.12) also blocked

### Migration Path (when approved)
1. **Create spike branch** — test sqlx 0.9 with Axum 0.8.9 (may work with feature flags)
2. **If spike fails** — evaluate Axum 0.8→0.9 upgrade scope (separate ADR)
3. **Fix compile errors** in all database modules
4. **Run migration tests** — `sqlx migrate run` against test DB
5. **Run full E2E suite** (156 tests in `test-nook.yml`)
6. **Verify WebRTC signaling persistence** (chat history, presence)
6. **Update Cargo.toml** — remove `=0.8` pin, allow `0.9`
7. **Run `cargo audit`**

## Alternatives Considered

### Option: Upgrade sqlx to 0.9 with Axum 0.8 (if compatible)
- **Status**: Unknown — needs spike. sqlx 0.9 *may* work with Axum 0.8 via feature flags
- **Action**: Spike required before decision

### Option: Upgrade Axum 0.8→0.9 simultaneously
- **Rejected**: Axum 0.9 is a major rewrite (extractors, router, middleware). Scope too large for single PR. Separate ADR needed.

### Option: Stay on sqlx 0.8 indefinitely
- **Rejected**: Security updates will cease. Technical debt compounds.

## Related Decisions

- **ADR-001**: Single Docker container — Axum/sqlx must remain compatible in single image
- **ADR-002**: Rust/Axum stack — sqlx is core data layer
- **ADR-003**: SQLite for primary DB — sqlx-sqlite is the driver
- **DEP-01**: Dependency analysis — this is item #9 in blocked section
- **DT-03**: Polls backend localStorage only — sqlx upgrade needed for server-side polls

## Verification

- [ ] Spike: sqlx 0.9 compiles with Axum 0.8.9 (or document incompatibilities)
- [ ] If compatible: all DB modules compile, migrations run
- [ ] If incompatible: Axum 0.9 upgrade ADR created and approved
- [ ] All 156 E2E tests pass
- [ ] `cargo audit` clean
- [ ] No regression in: auth, E2EE keys, WebRTC signaling, chess, polls, uploads, calendar

## Next Steps

**Awaiting architect approval to proceed with spike.** Create spike task for coder to test sqlx 0.9 compatibility with current Axum 0.8.9.