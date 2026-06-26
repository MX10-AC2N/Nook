# OOM Prevention — CARGO_BUILD_JOBS=1

## Contexte

Sur GitHub Actions (ubuntu-24.04-arm64 / ubuntu-latest amd64), le build `--release --target *-unknown-linux-musl` avec LTO + 1 codegen unit consomme ~14GB RAM en parallèle (cargo lance N jobs = CPU cores). Les runners GitHub ont ~7GB RAM → kill OOM (exit code 101).

## Fix

```yaml
# .github/workflows/Backend.yml — ajouter dans env du step cargo build --release
env:
  CARGO_BUILD_JOBS: "1"
  CARGO_PROFILE_RELEASE_LTO: "true"
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1"
  CARGO_PROFILE_RELEASE_OPT_LEVEL: "z"
  CARGO_PROFILE_RELEASE_STRIP: "true"
```

- `CARGO_BUILD_JOBS: "1"` → compilation mono-thread, RAM ~7GB max
- Sans ce flag : cargo utilise tous les cores → OOM arm64 + amd64
- Trade-off: build ~2x plus lent (~12min vs ~6min) mais **fiable**

## Vérification locale

```bash
# Simuler contrainte mémoire
CARGO_BUILD_JOBS=1 cargo build --release --target aarch64-unknown-linux-musl
```

Ajouté après 4 échecs OOM consécutifs (S50).