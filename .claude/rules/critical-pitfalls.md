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
---
### Tests E2E — regles de validation (2026-04-03)
1. **TOUJOURS** faire `npx playwright test --list` avant push pour valider la syntaxe
2. Chaque `test()` utilise `page` DOIT avoir `async ({ page }) =>` (pas `async () =>`)
3. Chaque `describe` qui utilise `adminPage` DOIT avoir son propre `let adminPage: Page;` + `test.beforeAll`
4. Les titres de test doivent etre UNIQUES dans la meme scope describe — pas de doublons
5. Chaque test doit se terminer par `});` — verifier les blocs try/catch ne laissent pas de test ouvert
6. Quand le backend renvoie 201 au lieu de 200, inclure 201 dans les assertions `.toContain([200, 201, 409])`
---
### 🧪 E2E Tests (2026-04-03)
1. **TOUJOURS** `npx playwright test --list` local avant push — jamais pusher sans validation
2. Si test utilise `page` ⇒ signature DOIT etre `async ({ page }) => {` pas `async () => {`
3. Si describe utilise `adminPage` ⇒ chaque describe besoin de son `let adminPage: Page;` + `test.beforeAll`
4. Titres de test UNIQUES dans chaque describe scope — pas de doublons
5. CHAQUE test doit se fermer avec `});` — verifier try/catch ne laisse pas test ouvert
6. Backend peut retourner 201 (Created) pas juste 200 — inclure dans assertions: `.toContain([200, 201, 409])`
7. `test.describe.serial` pour tests avec state share — `test.describe` standard sinon
8. Helpers partages dans `tests/helpers.ts` — loginAs, loginAsAdmin, clearSession, waitForAppReady
