# ADR-015 : sqlx 0.8 → 0.9 — migration compatible Axum 0.8

**Date** : 2026-08-07 (rédaction d'origine) · **Status** : Accepté · **Supersede** : —
**Owner** : architect · **Recréé** : 2026-10-22 (carte `t_566014c5`)

> **Provenance.** Fichier orphelin `0015-sqlx-08-to-09-migration.md`, rédigé le 2026-08-07
> (carte `t_dd44fc37`), **jamais commité**. Recréé ici au nom normalisé `015-sqlx-08-to-09-migration.md`
> (convention `docs/adr/NAMING.md`), corps repris tel quel, statut corrigé vers le statut réel.
> Décision de recréation : `@architect`, carte `t_566014c5`.

> **Preuve d'état (2026-10-22).** `backend/Cargo.toml:17` = `sqlx 0.8.6` (résolu dans `Cargo.lock`, non verrouillé avec `=`) ; `backend/Cargo.toml:10` = `axum 0.8`.
> **Spike de compatibilité (coder, carte `t_60a24c82`, 2026-10-22) — TERMINÉ.** Verdict : migration sqlx 0.8→0.9 **faisable en une passe** avec Axum 0.8.9. Le blocage « spike coder » est levé ; le statut `Proposé` passe à `Accepté` sur décision de l'architecte, le verdict du spike servant d'entrée de gate. Voir §4 (Résultats du spike).

> **Collision de numérotation.** Le fichier orphelin portait le numéro **015** alors que le commentaire
> de l'Architecte sur `t_dd44fc37` (2026-08-07) l'attribuait au **014**. La chronologie et le **titre du fichier**
> l'emportent : **015 = sqlx**. Le numéro 014 reste réservé à la migration rand_core/argon2. Voir
> `docs/adr/README.md` §4.4 et §6 et `docs/adr/HISTORY.md`.

## Context

`sqlx` 0.9.0 was released with breaking changes that the original ADR feared might affect:
- Axum integration (extracteurs, middleware)
- Query macros (`sqlx::query!`, `sqlx::query_as!`)
- SQLite driver (`sqlx-sqlite`)
- Migration API (`sqlx::migrate!`)

Current state (2026-10-22) :
- `sqlx` 0.8.6 in `backend/Cargo.toml:17` (resolved in `Cargo.lock`)
- `axum` 0.8.9 in `backend/Cargo.toml:10` (latest 0.8.x)
- `sqlx-sqlite` 0.8.6 (transitive)

The ADR was blocked awaiting a **coder spike** to *measure* — not design — the breakage with Axum 0.8.9.

## Résultats du spike de compatibilité (2026-10-22)

**Méthode.** Workspace jetable `backend-spike` (copie de `backend/`). `Cargo.toml:17` passé de `sqlx 0.8.6` → `sqlx 0.9` (features conservées : `runtime-tokio`, `tls-rustls`, `sqlite`, `macros`, `migrate`). `cargo check` **sans `cargo update`** (Cargo.lock préservé → rustrtc reste à 0.3.53). Livrable d'état : `spike-verdict-015-sqlx.md` + `cargo-check-spike.log` / `cargo-check-isolated.log`.

**Mesure — sqlx 0.9 + Axum 0.8.9 + rustrtc 0.3.53 (lock préservé) :**

| # | Catégorie | Élément | Code original | Après fix | Note |
|---|-----------|---------|---------------|-----------|------|
| 1 | `SqlSafeStr` | `main.rs:129` `sqlx::query_as(&format!(...))` | E0277 | OK | SQL dynamique |
| 2 | `SqlSafeStr` | `main.rs:137` `sqlx::query(&alter_sql)` | E0277 | OK | `alter_sql = format!(...)` |
| 3 | Macros | `sqlx::query!` / `query_as!` | 0 usage | 0 usage | absent du code |
| 4 | Axum 0.8 | extracteurs / middleware | OK | OK | **aucune rupture** |

- **Compilation (`cargo check`) :** code original → 2 erreurs (`E0277 SqlSafeStr`). Après fix (`sqlx::AssertSqlSafe(...)`) → **0 erreur, exit 0** en 4.05s, 8 warnings pré-existantes (`snake_case` dans `e2ee.rs`, non sqlx, non regression).
- **Volume des call sites :** 288 sites `sqlx::query`/`query_as`/`query(` sur 19 fichiers (`db`, `auth`, `e2ee`, `polls`, `chess`, `push`, `events`, …). **Seulement 2** construisent le SQL dynamiquement (`format!`). Les 286 autres utilisent des littéraux `&'static str` (implémentent `SqlSafeStr` en 0.9) → **compilés sans changement**.
- **Macros :** 0 usage de `sqlx::query!`/`query_as!`. `migrate!` et `.sqlx/queries.json` (offline) sont compatibles 0.9 (compilés OK).

**Conclusion de mesure :** l'unique rupture SQLx réelle est le nouveau trait `SqlSafeStr` (0.9 rejette les strings dynamiques à la compilation). Elle concerne **2 lignes** dans `main.rs` (`fix_events_schema` :122-142), corrigeable par `sqlx::AssertSqlSafe(...)` ou par refactorisation vers `QueryBuilder::push_bind` (le motif déjà utilisé dans `events.rs:368`).

**Rupture Axum 0.8 — le cœur du blocage — est FUMÉE :** sqlx 0.9 compile nettoyement AVEC Axum 0.8.9 (aucune erreur extracteur/middleware). sqlx et axum sont des crates indépendantes ; 0.9 ne force pas 0.9. La crainte initiale (« sqlx 0.9 may require Axum 0.9 ») est réfutée par la mesure.

**Risque orthogonal (non sqlx) — drift rustrtc :** un `cargo update` fait glisser `rustrtc` 0.3.53 → 0.3.146 (version réelle, vérifiée dans le registry local), ce qui brise `sfu.rs` avec 2 erreurs d'API rustrtc media (NON sqlx) :
```
error[E0063]: missing field `name` in initializer of `RtpCodecParameters`  -> src/sfu.rs:358
error[E0277]: `Result<(), MediaError>` is not a future                  -> src/sfu.rs:448 (send() devient synchrone)
```
L'API rustrtc `media` n'est utilisée que dans `sfu.rs` (1 fichier). À isoler du passage sqlx : le bump sqlx seul (lock préservé) ne touche pas rustrtc.

## Decision

**Accepter et procéder à la migration sqlx 0.8 → 0.9 en une passe, en conservant Axum 0.8.9.**

La migration est évaluée comme **faible risque / faisable en une passe** :
- 2 sites à corriger (`main.rs:129`, `main.rs:137`) via `sqlx::AssertSqlSafe(...)` (ou `QueryBuilder`).
- 0 rupture Axum, 0 macro à réécrire, 0 autre module à toucher.
- rustrtc à garder bloqué à 0.3.53 (ne pas lancer `cargo update` global), ou traiter le drift rustrtc dans une carte séparée.

## Consequences

### Positive (after migration)
- sqlx 0.9 : améliorations query caching, le trait `SqlSafeStr` force l'audit du SQL dynamique (sécurité), fixes transitifs (libsqlite3-sys 0.30→0.37, flume 0.11→0.12).
- Débloque la voie future d'Axum 0.9 (séparément) sans dépendance croisée sqlx↔axum.
- DT-03 (server-side polls) et DT-04 (rate limiting) peuvent avancer.

### Negative / Risks
- `AssertSqlSafe` délègue l'audit au développeur → obligation de revoir `fix_events_schema` (`main.rs:122-142`) pour valider qu'aucune injection n'est possible sur `col_name`/`col_def`. → revoir par `@security-auditor` recommandé.
- Le drift rustrtc (0.3.146) reste un piège si `cargo update` est lancé globalement.

## Related Decisions

- **ADR-001** (single container) : Axum/sqlx restent compatibles dans l'image unique — sqlx 0.9 ne change rien à l'image.
- **ADR-002** (Rust/Axum stack) : sqlx est la couche data — compatible 0.9.
- **ADR-003** (SQLite) : `sqlx-sqlite` 0.9 compilé OK.
- **ADR-014** (rand_core/argon2) : migration séparée, toujours bloquée à `security-auditor`.

## Verification
- [x] Spike : sqlx 0.9 compile avec Axum 0.8.9 (`cargo check` exit 0, 2 sites corrigés via `AssertSqlSafe`)
- [x] Compatibilité Axum 0.8 confirmée (aucune rupture d'extracteur/middleware)
- [x] Volume des call sites mesuré (288 sites, 2 à modifier)
- [x] Macros `query!`/`query_as!` absentes (N/A)
- [ ] Migration réellement appliquée sur `backend/` (prod) — tâche suivante
- [ ] `cargo test` (unit + 156 E2E) passent
- [ ] `cargo audit` clean
- [ ] Revu par `@security-auditor` sur `AssertSqlSafe` (`main.rs` `fix_events_schema`)

## Migration Path (approuvée)
1. `backend/Cargo.toml:17` — `sqlx = "0.8.6"` → `sqlx = "0.9"` (features conservées).
2. `backend/src/main.rs:129` et `:137` — wrapper `sqlx::AssertSqlSafe(...)` (ou refactor `QueryBuilder`).
3. Ne pas faire `cargo update` global : garder `rustrtc` à 0.3.53 (ou créer une carte dédiée pour le drift rustrtc 0.3.53→0.3.146).
4. `cargo check` + `cargo test`.
5. `cargo audit` + revue `@security-auditor` sur le `AssertSqlSafe` de `fix_events_schema`.

## Alternatives Considered

### Option: Upgrade sqlx to 0.9 with Axum 0.8 (if compatible)
- **Status**: **Compatible — measureé.** sqlx 0.9 *works* with Axum 0.8.9 ; aucune feature flag nécessaire.

### Option: Upgrade Axum 0.8→0.9 simultaneously
- **Rejected**: Axum 0.9 is a major rewrite (extracteurs, router, middleware). Le spike confirme qu'il n'est **pas** nécessaire pour sqlx 0.9 — scope inutile.

### Option: Stay on sqlx 0.8 indefinitely
- **Rejected**: Security updates will cease. Technical debt (DT-03/DT-04) stays bloqué.

## Next Steps

**Spike terminé — migration recommandée (migrer).** Verdict détaillé dans §4. La migration prod proprement dite sur `backend/` (fix + bump Cargo.toml) est une tâche distincte, à créer après acceptation de cette ADR.
