# GitHub Actions Backend Build — OOM Fix (CARGO_BUILD_JOBS=1)

## Problème
Build `cargo build --release --target x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl` échoue avec **OOM (exit code 101)** sur GitHub Actions runners (7GB RAM arm64, 14GB amd64).

Cause : LTO (`CARGO_PROFILE_RELEASE_LTO=true`) + parallelisation par défaut (tous les CPU cores) consomme trop de RAM lors du linking/rustc.

## Fix appliqué (session 2026-06-13)

Dans `.github/workflows/Backend.yml`, step `cargo build --release` :

```yaml
env:
  CARGO_BUILD_JOBS: "1"
  CARGO_PROFILE_RELEASE_LTO: "true"
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1"
  CARGO_PROFILE_RELEASE_OPT_LEVEL: "z"
  CARGO_PROFILE_RELEASE_STRIP: "true"
```

## Résultat
- **Avant** : OOM à ~4 min (arm64) / ~6 min (amd64) — build incomplete, artifacts manquants
- **Après** : Build complet 6-7 min — artifacts produits pour amd64 + arm64, upload OK

## Notes
- `CARGO_BUILD_JOBS=1` force compilation mono-thread (un seul codegen unit à la fois)
- Perte de vitesse vs parallélisation max, mais **fiable** sur runners limités (7GB RAM)
- Alternative testée mais non-fiable : `cargo --jobs 1` en flag au lieu d'env — l'env est plus propre
- Ne pas désactiver LTO (gain taille binaire ~30%) — juste limiter la parallélisation