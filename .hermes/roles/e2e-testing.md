# 🧪 Rôle : Ingénieur Tests E2E — Nook

> Spécialiste Playwright + stabilisation test suite Nook.
> Activer ce rôle pour : nouveaux tests, debug timeout, analyse TEST_REPORT.md, fixtures.

---

## 🎯 Périmètre exclusif

```
frontend/
├── tests/e2e.spec.ts          → 43 tests E2E (source de vérité)
├── playwright.config.ts       → Config Playwright (fullyParallel: false !)
└── package.json               → @playwright/test inclus

.github/workflows/test-nook.yml → CI E2E
.hermes/TEST_REPORT.md          → Résultats du dernier run CI
```

---

## ⚠️ Historique des échecs — NE PAS RÉINTRODUIRE

7 sessions de debugging (17-23) pour stabiliser 43 tests. Résumé des pièges :

| # | Pattern dangereux | Pourquoi ça casse | Solution validée |
|---|-------------------|-------------------|-----------------|
| 1 | `clearCookies()` seul | localStorage intact → `isAuthenticated=true` | + révocation token serveur |
| 2 | `goto('about:blank') + localStorage.clear()` | Origine ≠ localhost:6300 → localStorage isolé | goto sur l'app |
| 3 | `addInitScript()` sur Page | S'exécute sur about:blank, pas sur l'app | API-first |
| 4 | `fullyParallel: true` + workers:1 | Même browser context → localStorage partagé | `fullyParallel: false` |
| 5 | `goto('/') + evaluate(localStorage)` | `goto('/')` monte le layout → `authStore.init()` avec cookie valide → redirect | Révoquer AVANT goto |
| 6 | `fill('#username')` juste après `goto('/login')` | Layout `loading=true` → `#username` pas dans le DOM | `waitFor(visible)` |

---

## ✅ Helpers validés — COPIER-COLLER EXACT

```typescript
const BASE = 'http://localhost:6300/api';

// ✅ clearSession — SESSION 22 — API-first, pas de goto préalable
async function clearSession(page: Page): Promise<void> {
  try {
    // Révoquer le token en DB AVANT toute navigation browser
    await page.request.post(`${BASE}/auth/logout`);
    // 200 = révoqué | 401 = pas de session active → les deux sont OK
  } catch {
    // Ignorer les erreurs réseau (server pas encore démarré, etc.)
  }
  // Vider les cookies du browser context
  await page.context().clearCookies();
  // ❌ NE PAS faire goto('/') ici → authStore.init() se déclencherait
  // ❌ NE PAS faire localStorage.clear() ici → authStore.logout() le fait via 401
}

// ✅ loginAs — SESSION 23 — waitFor avant fill
async function loginAs(
  page: Page,
  username: string,
  password: string
): Promise<void> {
  await clearSession(page);
  await page.goto('/login');
  // Layout : onMount → waitForSodium (~500ms) → initCrypto → authStore.init → loading=false
  // #username n'est dans le DOM QU'APRÈS loading=false
  await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });
  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.click('button[type="submit"]');
}

// ✅ loginAsAdmin — SESSION 19 — API-first (bypass browser totalement)
async function loginAsAdmin(page: Page): Promise<void> {
  const res = await page.request.post(`${BASE}/auth/login`, {
    data: { username: 'admin', password: process.env.ADMIN_PASSWORD ?? 'changeme2026' },
  });
  // Le cookie est posé par la réponse, pas besoin de naviguer
  // Pour les tests admin qui ont besoin d'une page, faire goto après
}
```

---

## ⚙️ playwright.config.ts — Configuration correcte

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',

  // ✅ OBLIGATOIRE — évite le partage de browser context entre tests
  fullyParallel: false,

  // Fail fast en CI pour économiser du temps
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,  // 1 retry en CI pour les flaky tests

  // 1 worker en CI (container Docker mono-thread)
  workers: process.env.CI ? 1 : undefined,

  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],

  use: {
    // URL de base
    baseURL: 'http://localhost:6300',

    // Timeout par action (fill, click, etc.)
    actionTimeout: 10_000,

    // Trace en cas d'échec (utile pour debug CI)
    trace: 'on-first-retry',
  },

  // Timeout global par test
  timeout: 30_000,

  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],

  // ✅ Réutiliser le serveur Docker déjà démarré en CI
  webServer: {
    command: 'echo "Server already running"',
    url: 'http://localhost:6300/api/health',
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
```

---

## 🏗️ Structure des tests — Patterns recommandés

```typescript
import { test, expect, type Page } from '@playwright/test';

// Catégories de tests Nook
// 1. Tests "request" — API pure, pas de browser (les plus rapides et fiables)
// 2. Tests "browser" — navigation + UI
// 3. Tests "admin" — loginAsAdmin + actions admin

// ✅ Test API pur (pas de browser, toujours vert)
test('GET /api/health retourne OK', async ({ request }) => {
  const res = await request.get('http://localhost:6300/api/health');
  expect(res.ok()).toBeTruthy();
  expect(await res.text()).toBe('OK');
});

// ✅ Test auth browser
test('Login valide redirige vers /chat', async ({ page }) => {
  await loginAs(page, 'e2e_ci', 'E2eTest123!');
  await expect(page).toHaveURL(/\/chat/, { timeout: 10_000 });
});

// ✅ Test avec attente explicite d'élément
test('Admin voit la liste des users', async ({ page }) => {
  await loginAsAdmin(page);
  await page.goto('/admin');
  await page.locator('[data-testid="admin-header"]')
    .waitFor({ state: 'visible', timeout: 15_000 });
  await expect(page.locator('[data-testid="users-list"]')).toBeVisible();
});
```

---

## 🔍 Analyser un TEST_REPORT.md échoué

```
Lecture du rapport :
1. Identifier le ratio passé/total (ex: 12/43)
2. Regarder les noms des tests qui échouent
3. Lire le message d'erreur exact :
   - "waiting for locator('#username')" → layout timing ou clearSession
   - "Expected URL to be /chat" → login échoue (credentials ? server ?)
   - "waiting for locator('#xxx')" → sélecteur obsolète (UI renommée ?)
   - "net::ERR_CONNECTION_REFUSED" → backend pas démarré

Patterns fréquents :
- Tous les tests browser échouent, request OK → backend UP mais UI down
- Tests login échouent → authStore, layout loading, ou clearSession
- Tests admin échouent → loginAsAdmin, ou needs_password_change non géré
- Timeouts aléatoires → libsodium lent sur runner CI → augmenter waitFor timeout
```

---

## 📋 Ajouter un nouveau test — Checklist

1. **Identifier la catégorie** : `request` (API) ou `browser` (UI)
2. **Si browser** : utiliser `loginAs()` (jamais de login manuel inline)
3. **Ajouter `data-testid`** sur les éléments clés dans le composant Svelte
4. **Utiliser `waitFor`** sur les éléments dynamiques (jamais de timeout fixe)
5. **Vérifier** que le test passe 3 fois de suite localement avant commit
6. **Mettre à jour** le compte total de tests dans `CLAUDE.md` et `SESSIONS.md`

---

## 🏷️ data-testid recommandés (cohérence avec tests existants)

```
login-form          → formulaire de login
admin-header        → titre page admin
users-list          → liste des utilisateurs
invite-list         → tableau des invitations
chat-messages       → zone des messages
chat-input          → textarea d'envoi
send-button         → bouton envoyer
logout-button       → bouton déconnexion
```

---

## ⚡ Workflows dédiés

| Workflow | Déclencheur | Action |
|----------|-------------|--------|
| `e2e-targeted.yml` | Manuel + `test_grep` | Lance 1 test par nom — debug rapide sans relancer toute la suite |
| `test-nook.yml` | Manuel | Suite complète 43 tests + commit TEST_REPORT.md |

> Stratégie de debug recommandée :
> 1. `e2e-targeted.yml` avec `debug_traces: true` → identifier l'échec précis
> 2. Corriger + commit
> 3. `test-nook.yml` complet pour valider

## 🤝 Flux inter-agents

```
← 🦀 RUST / 🎨 SVELTE / 🚀 DEVOPS : stack up, sélecteurs stables, codes HTTP documentés
→ Tous                               : TEST_REPORT.md, liste tests à ajouter, data-testid manquants
```

---

## 📚 Apprentissages

> *Section mise à jour à chaque session. Contient l'historique complet de stabilisation.*

### [APP-E2E-01] fullyParallel:true partage le browser context — Session 21
→ **Promu** dans `playwright.config.ts` — Configuration correcte.

### [APP-E2E-02] clearSession doit révoquer le token AVANT goto — Session 22
→ **Promu** dans Helpers validés.

### [APP-E2E-03] fill() avant layout onMount = timeout — Session 23
→ **Promu** dans Helpers validés.

### [APP-E2E-04] /health vs /api/health — Session 21
→ **Promu** dans playwright.config.ts webServer.url.

### [APP-E2E-05] loginAsAdmin via API — Session 19
→ **Promu** dans Helpers validés.

### [APP-E2E-13] Cookie admin pollué par loginAs testUser — Sessions 38-39

Utiliser `adminPage.request.post('/auth/login', testUser)` dans une suite `.serial`
remplace le cookie `auth_token` de `adminPage` → tous les tests suivants obtiennent 403.
Fix : `isolatedPage = await browser.newPage()` pour tous les appels login/register du testUser,
fermé dans `finally`. Le `adminPage` conserve sa session admin intacte du début à la fin.

### [APP-E2E-14] Serde fields obligatoires → 422 — Session 39

Les champs Rust `bool` sans `#[serde(default)]` sont obligatoires dans le JSON body.
Tests qui oublient `encrypted: false` ou `is_group: true` → Axum retourne 422.
Règle : toujours inclure les champs requis, ou ajouter `#[serde(default)]` au backend.

### [APP-E2E-15] Réponse JSON imbriquée — vérifier le niveau — Session 39

`POST /polls` retourne `{ "poll": { "id": ..., ... } }` pas `{ "id": ... }`.
`POST /invites` retourne `{ "invite_link": "/invite?token=..." }` pas `{ "token": ... }`.
Règle : toujours vérifier avec un `console.log(await res.json())` ou les logs CI
avant d'écrire les assertions sur la structure de réponse.

### [APP-E2E-16] Hover CI headless + state Svelte — Session 39

`onmouseenter` Svelte (qui déclenche `hoveredMsgId = msg.id`) peut ne pas se déclencher
avec `page.hover()` seul en CI headless.
Fix validé : `await msg.hover()` + `await msg.dispatchEvent('mouseenter')` + `await page.waitForTimeout(300)`.
Pour les actions qui déclenchent un appel réseau : utiliser `Promise.all([waitForResponse(...), click()])`.

### [APP-E2E-06] addInitScript sur Page = about:blank — Session 18

`page.addInitScript()` s'exécute dans le contexte `about:blank` (avant la navigation).
LocalStorage de `about:blank` ≠ localStorage de `localhost:6300` → inefficace.
→ Ne jamais utiliser `addInitScript` pour manipuler le localStorage de l'app.
Status : Archivé (anti-pattern documenté).
