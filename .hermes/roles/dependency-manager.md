# 📦 Rôle : Dependency-Manager — Nook

> Gère les dépendances Cargo/pnpm, les CVEs et les mises à jour de version.

## Responsabilités
1. **Cargo dependencies** : Mettre à jour `Cargo.lock`, pinner les versions sûres
2. **pnpm dependencies** : Mettre à jour `package-lock.json`, gérer les peer deps
3. **CVEs** : Rechercher et appliquer les correctifs de sécurité
4. **rand_core 0.6 pin** : Forcer `rand_core 0.6` pour argon2 (ne jamais importer `rand::rngs::OsRng`)

## Pièges Critiques
- ⚠️ **rand 0.9** : utiliser `rng()` (NE PAS utiliser `thread_rng()` — supprimé en rand 0.9)
- ⚠️ **rand 0.9** : imports = `use rand::{rng, distr::Alphanumeric, Rng}` (NE PAS utiliser `distributions` — déplacé vers `distr`)
- ⚠️ **rand_core 0.6 forcé** pour argon2 — ne JAMAIS importer `rand::rngs::OsRng`
- ⚠️ **Mettre à jour Cargo.lock** après chaque changement de dépendance

## Commandes Utiles
```bash
# Mise à jour Cargo
cargo update
cudit  # Check for outdated deps

# Sécurité
cargo audit  # Vérifier les vulnérabilités connues

# Mise à jour pnpm
pnpm update
pcii  # Check peer deps
```

## Liens Rapides
- ← Code: `Cargo.toml`, `package.json`
- ← Pièces: `rules/critical-pitfalls.md`
- ← Sécurité: `rules/security-management.md` (quand disponible)
- ← Audit: `cargo audit` report