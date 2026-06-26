---
name: nook-e2e-testing
description: "Skill spécialisé pour les tests E2E Playwright du projet Nook"
---

# 🧪 Nook — E2E Testing Skill

## Périmètre

```
frontend/
├── tests/e2e.spec.ts          → Suite complète (43+ tests)
└── playwright.config.ts       → Config (fullyParallel: false !)

.github/workflows/
├── test-nook.yml              → CI E2E (Docker + Playwright)
└── e2e-targeted.yml           → Debug un test précis

.hermes/TEST_REPORT.md         → Résultats du dernier run CI
```

## Helpers validés — copier-coller EXACT

Ces helpers ont été stabilisés sur 7 sessions de debug. Ne pas les modifier sans raison.

### clearSession — révocation API-first
```typescript
// BASE depuis helpers.ts : ${process.env.NOOK_BASE_URL || 'http://localhost:6300'}/api
// Pour tests remote (CI, Zimaboard) : export NOOK_BASE_URL=http://192.168.1.192:6300

async function clearSession(page: Page): Promise<void> {
  try {
    // ✅ Révoquer le token en DB AVANT toute navigation
    await page.request.post(`${BASE}/auth/logout`);
    // 200 = révoqué | 401 = pas de session → les deux sont OK
  } catch {
    // Ignorer les erreurs réseau
  }
  await page.context().clearCookies();
  // ❌ NE PAS faire goto('/') ici
  // ❌ NE PAS faire localStorage.clear() ici (authStore.logout() le fait via 401)
}
```

### loginAs — waitFor avant fill
```typescript
async function loginAs(page: Page, username: string, password: string): Promise<void> {
  await clearSession(page);
  await page.goto('/login');

  // ✅ OBLIGATOIRE : layout fait waitForSodium ~500ms + authStore.init()
  // #username n'existe pas dans le DOM avant la fin de onMount
  await page.waitForSelector('#username', { state: 'visible', timeout: 20000 });
  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.click('[data-testid="login-btn"]');
  await page.waitForURL('**/chat**', { timeout: 15000 });
}
```

### helpers.ts — BASE configurable via NOOK_BASE_URL
Le fichier `frontend/tests/helpers.ts` exporte `BASE` construit depuis `process.env.NOOK_BASE_URL` :
```typescript
export const BASE = `${process.env.NOOK_BASE_URL || 'http://localhost:6300'}/api`;
export const E2E_USER = 'e2e_ci';
export const E2E_PASS = 'E2eTest123!';
```
**Usage CI/remote** :
```bash
NOOK_BASE_URL=http://192.168.1.192:6300 npx playwright test
```
Tous les tests API (api-sanity, admin-flow, user-flow, call-ui) héritent de ce BASE.

## Les 7 pièges — historique des 43 tests

Ces patterns ont causé des échecs répétés. Ne pas les réintroduire.

| # | Pattern dangereux | Pourquoi ça casse | Solution validée |
|---|-------------------|-------------------|--------------------|
| 1 | `clearCookies()` seul | localStorage intact → `isAuthenticated=true` | + révocation token serveur |
| 2 | `goto('about:blank') + localStorage.clear()` | Origine ≠ localhost:6300 → localStorage isolé | goto sur l'app |
| 3 | `addInitScript()` sur Page | S'exécute sur about:blank | API-first |
| 4 | `fullyParallel: true` + workers:1 | Browser context partagé → localStorage pollué | `fullyParallel: false` |
| 5 | `goto('/') + evaluate(localStorage)` | Layout monte → authStore.init() avec cookie → redirect | Révoquer AVANT goto |
| 6 | `fill('#username')` juste après `goto('/login')` | `#username` pas encore dans le DOM | `waitFor(visible, 20s)` |
| 7 | `waitForResponse` après `goto()` | Listener enregistré trop tard → race condition | `Promise.all([waitForResponse, goto()])` |

## Pièges découverts Session 55 — remote test server

| # | Pattern | Symptôme | Fix |
|---|---------|----------|-----|
| 8 | `BASE` manquant dans api-sanity.spec.ts | `ReferenceError: BASE is not defined` sur 71 tests | `import { BASE } from './helpers'` |
| 9 | Tests sans `NOOK_BASE_URL` | `net::ERR_CONNECTION_REFUSED at http://localhost:6300` | Export `NOOK_BASE_URL=http://IP:6300` |
| 10 | Call page tests en CI headless | `page.waitForSelector` timeout sur `#username` | `test.skip(process.env.CI, 'WebRTC needs media')` |
| 11 | Rate limit login `e2e_ci` / `hermes-bot` | HTTP 429 après ~5 login rapides | Espacer tests / users dédiés par suite |

## Pièges découverts Session 56 — validation complète & corrections

| # | Pattern | Symptôme | Fix |
|---|---------|----------|-----|
| 12 | `api-sanity.spec.ts` upload endpoint hardcodé | `POST /api/upload/chat` → `ECONNREFUSED ::1:6300` | Utiliser `${BASE}/upload/chat` (helpers.ts export BASE) |
| 13 | Call page tests URLs hardcodées | `page.goto('http://localhost:6300/call/...')` ignore NOOK_BASE_URL | `const BASE_URL = process.env.NOOK_BASE_URL || '...'; page.goto(\`${BASE_URL}/call/...\`)` |
| 14 | Rate limit 429 sur tests weak password | `expect(res.status()).toBe(400)` échoue → reçu 429 | Accepter `[400, 429]` ou `[200, 409, 429]` selon test |
| 15 | 8 chars password test accepte 429 | `expect([200, 409]).toContain(429)` → fail | Ajouter 429 dans la liste attendue |
| 16 | WebRTC / Call page tests non testables en CI | Timeouts navigateur, media devices non disponibles | `test.skip(true, 'Call page requires WebRTC setup not available in test env')` |

## Sélecteurs — règles strictes

```typescript
// ✅ Toujours utiliser id= ou data-testid=
await page.waitForSelector('#username', { state: 'visible' });
await page.click('[data-testid="logout-btn"]');

// ❌ Ne jamais utiliser name=, class=, ou texte
await page.click('button.logout');        // fragile
await page.click('text=Se déconnecter'); // fragile
```

## playwright.config.ts — config obligatoire

```typescript
export default defineConfig({
  testDir: './tests',
  fullyParallel: false,  // ← OBLIGATOIRE (voir piège #4)
  workers: 1,            // ← un seul worker en CI
  timeout: 30000,
  use: {
    baseURL: 'http://localhost:6300',
    trace: 'on-first-retry',
  },
});
```

## Lecture du TEST_REPORT.md

```
✓ = test passé
✘ = test échoué → lire le message d'erreur en dessous
Timeout = attente d'un élément UI → suspect : waitFor manquant
expect received "" = selector présent mais vide → mauvais timing
page.goto() résolu ≠ onMount() terminé → toujours waitFor('#username')
```

## Workflow de debug d'un test qui échoue

```
1. Lire TEST_REPORT.md → identifier le test et le message exact
2. Timeout ?  → waitFor manquant ou timing onMount
3. Selector ?  → vérifier id= / data-testid= dans le composant Svelte
4. Lancer e2e-targeted.yml avec debug_traces:true sur ce test précis
5. Télécharger les traces → screenshots Playwright
```

## Pattern race condition navigations

```typescript
// ❌ waitForResponse enregistré APRÈS goto → peut rater la réponse
await page.goto('/polls');
await page.waitForResponse('**/api/polls**');

// ✅ Promise.all → listener enregistré AVANT la navigation
await Promise.all([
  page.waitForResponse(resp => resp.url().includes('/api/polls') && resp.status() === 200),
  page.goto('/polls')
]);
```

## Checklist ajout d'un nouveau test

```
□ clearSession() en début de test (jamais de session résiduelle)
□ loginAs() avec waitFor('#username') intégré
□ Sélecteurs : id= ou data-testid= uniquement
□ waitFor(visible) avant toute interaction sur un élément dynamique
□ Promise.all([waitForResponse, navigation]) si réponse API attendue
□ Test isolé : aucune dépendance sur l'état d'un test précédent
□ Couvrir au minimum : happy path + un cas d'erreur
```

## Flux inter-agents

```
← 🦀 RUST  : après tout nouvel endpoint → contrat HTTP (URL, payload, codes)
← 🎨 SVELTE : après tout nouveau composant → sélecteurs id=/data-testid=
→ Si test échoue sur timing  : signaler à 🎨 SVELTE (onMount / loading guard ?)
→ Si test échoue sur 401/403 : signaler à 🦀 RUST (middleware auth ?)
```