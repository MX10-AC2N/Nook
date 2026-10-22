# ADR-014 : rand_core 0.6 → 0.10 / argon2 0.5 → 0.6 — migration bloquée

**Date** : 2026-08-07 (rédaction d'origine) · **Status** : Proposé (bloqué — security-auditor) · **Supersede** : —
**Owner** : architect · **Recréé** : 2026-10-22 (carte `t_566014c5`)

> **Provenance.** Fichier orphelin `0014-rand_core-argon2-migration.md`, rédigé le 2026-08-07
> (carte `t_dd44fc37`), **jamais commité**. Recréé ici au nom normalisé `014-rand-core-argon2-migration.md`
> (convention `docs/adr/NAMING.md`), corps repris tel quel, statut corrigé vers le statut réel.
> Décision de recréation : `@architect`, carte `t_566014c5`.

> **Preuve d'état (2026-10-22).** `backend/Cargo.toml:35` = `rand_core 0.6`, `:38` = `argon2 0.5` ;
> `backend/Cargo.lock` compile **trois** copies de `rand_core` (0.6.4, 0.9.5, 0.10.1). La décision n'a
> pas abouti : la migration reste bloquée, le statut « Proposé » est donc exact.

> **Collision de numérotation.** Le fichier orphelin portait le numéro **014** alors que le commentaire
> de l'Architecte sur `t_dd44fc37` (2026-08-07) l'attribuait au **013**. La chronologie (ADR-011 le
> 2026-07-02, ADR-012 le 2026-07-11, ADR-013 dual message store le 2026-07-11) et le **titre du fichier**
> l'emportent : **014 = rand_core/argon2**. Le numéro 013 reste réservé au dual message store (clos).
> Voir `docs/adr/README.md` §4.4 et `docs/adr/HISTORY.md`.

## Context

`rand_core` is pinned at `=0.6` in `Cargo.toml` (line 36) explicitly for `argon2` 0.5 compatibility. This is a diamond dependency fix — `argon2` 0.5 depends on `rand_core` 0.6, while the rest of the crate graph (including `rand` 0.9) has moved to `rand_core` 0.9/0.10.

Current state:
- `rand_core` 0.6.4 (pinned, used by `argon2` 0.5.3)
- `rand_core` 0.9.5 (used by `rand` 0.9.5)
- `rand_core` 0.10.1 (latest, required by `rand` 0.10+)

This creates three copies of `rand_core` in the dependency tree, increasing binary size and creating potential trait incompatibility risks.

## Decision

**Defer migration until `argon2` 0.5→0.6 upgrade is coordinated with security-auditor.**

### Rationale

1. **argon2 0.6 is a major version** with API changes — the `Argon2` struct and `verify`/`hash` methods have signature changes
2. **Security impact**: `argon2` is our password hashing KDF — any change requires security-auditor review
3. **Migration scope**: Requires updating all password hash/verify call sites in `auth.rs`
4. **Testing requirement**: Must verify existing password hashes remain verifiable after migration

## Consequences

### Positive (after migration)
- Single `rand_core` version (0.10) across entire crate graph
- Enables `rand` 0.9→0.10 upgrade (currently blocked by this)
- Reduces binary size (one less `rand_core` copy)
- Aligns with ecosystem (most crates on `rand_core` 0.9+)

### Negative / Risks (current blocked state)
- Three `rand_core` versions compiled in (0.6, 0.9, 0.10)
- `rand` 0.9→0.10 blocked (needs `rand_core` 0.10)
- Trait incompatibility risk if `RngCore`/`CryptoRng` used across versions
- Security audit surface unchanged (argon2 0.5 still maintained but older)

### Migration Path (when unblocked)
1. Security-auditor reviews argon2 0.6 changelog + API diff
2. Update `argon2 = "0.5"` → `argon2 = "0.6"` in Cargo.toml
3. Fix compile errors in `auth.rs` (hash/verify signatures)
4. Add integration test: verify existing bcrypt/argon2 hashes still work
5. Remove `rand_core = { version = "0.6", ... }` explicit dependency
6. Update `rand` to 0.10 (separate PR, unblocked by this)
7. Run full test suite + `cargo audit`

## Alternatives Considered

### Option: Keep argon2 0.5, upgrade rand_core anyway
- **Rejected**: Cargo will fail — argon2 0.5 *requires* rand_core 0.6 (semver incompatible with 0.9+)

### Option: Fork argon2 0.5 to use rand_core 0.10
- **Rejected**: Maintains security-critical crypto code — unacceptable risk

### Option: Use argon2 0.4 (older)
- **Rejected**: Downgrade reduces security, no benefit

## Related Decisions

- **ADR-004**: E2EE Double Ratchet — argon2 used for key derivation from passwords
- **DT-04**: Rate limiting governor — uses `rand` for tokens (blocked on rand 0.10)
- **DEP-01**: Dependency analysis report — this is item #8 in blocked section

## Verification

- [ ] security-auditor approves argon2 0.5→0.6 migration plan
- [ ] Migration PR passes all auth-related tests
- [ ] Existing user password hashes verify correctly post-migration
- [ ] `cargo audit` clean (no new advisories)
- [ ] Binary size reduced (single rand_core)

## Next Steps

**Awaiting security-auditor coordination.** Create follow-up task for security-auditor to review argon2 0.6 changelog and approve migration.