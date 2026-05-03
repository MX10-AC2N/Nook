# 🧑‍💻 Coding Style — Index

> Les règles de code vivent directement dans chaque rôle pour éviter la redondance.
> Ce fichier est un index de redirection.

| Domaine | Règles dans |
|---------|------------|
| Rust : rand, axum 0.8, CORS, SQLite, WebSocket | `roles/rust-backend.md` § Pièges critiques |
| Svelte 5 : $state, $derived, $effect, stores | `roles/svelte-frontend.md` § Règles Svelte 5 Runes |
| Playwright : clearSession, waitFor, config | `roles/e2e-testing.md` § Helpers validés |
| Crypto : argon2, OsRng, XChaCha20, cookie | `roles/security-crypto.md` § Pièges critiques |
| CI : heredoc, healthcheck, dawidd6 | `roles/ci-devops.md` § Règles d'or |
| Polls/sqlx sans macros | `roles/data-analytics.md` § Points critiques |

## Règles universelles (tous fichiers)

```
- Fetcher Raw GitHub avant d'intervenir (jamais de mémoire)
- Livrer le fichier complet (jamais de diff partiel)
- .svelte/.ts → livrer en .txt | .rs/.sql → direct
- Chemin explicite en tête de chaque bloc de code
```
