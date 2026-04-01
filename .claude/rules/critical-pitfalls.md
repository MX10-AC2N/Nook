# Pièges critiques du projet Nook

- `rand 0.9` : utiliser `rng()` (NE PAS utiliser `thread_rng()` — supprimé en rand 0.9)
- `rand 0.9` : imports = `use rand::{{rng, distr::Alphanumeric, Rng}}` (NE PAS utiliser `distributions` — déplacé vers `distr`)
- `rand_core 0.6` forcé explicitement pour argon2 — ne JAMAIS importer `rand::rngs::OsRng`
- Routes Axum 0.8 : `{param}` au lieu de `:param`
- `$state` Svelte 5 → utiliser `Object.assign()` ou `$effect`
- CORS + credentials → origins explicites uniquement
- sqlx : éviter les macros quand `queries.json` est vide
- Ne jamais utiliser `?` dans les queries SQLx sans `query!` macro
- `tokio::spawn` sans `move` sur les closures qui capturent des variables
- Oublier de mettre à jour `Cargo.lock` après un changement de dépendance