# Pièges critiques du projet Nook

- `rand::rng()` → utiliser `thread_rng()` ou `rng()`
- Routes Axum 0.8 : `{param}` au lieu de `:param`
- `$state` Svelte 5 → utiliser `Object.assign()` ou `$effect`
- CORS + credentials → origins explicites uniquement
- sqlx : éviter les macros quand `queries.json` est vide
- Ne jamais utiliser `?` dans les queries SQLx sans `query!` macro
- `tokio::spawn` sans `move` sur les closures qui capturent des variables
- Oublier de mettre à jour `Cargo.lock` après un changement de dépendance