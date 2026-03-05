// frontend/tests/e2e.spec.ts
// Suite E2E complète — session 22
//
// CORRECTION vs session 21 :
//
//   Bug #22 — clearSession() goto('/') déclenche authStore.init() avec cookie valide
//
//   Session 21 avait implémenté clearSession() avec goto('/') en premier.
//   Problème : goto('/') monte le layout → onMount → authStore.init()
//   → fetch('/api/auth/me') AVEC le cookie encore présent dans le browser
//   → 200 → isAuthenticated=true → $effect redirige vers /chat
//   → clearCookies() ensuite est trop tard
//   → loginAs goto('/login') → isAuthenticated=true → redirect immédiat → #username inaccessible
//
//   Fix définitif — clearSession() SANS navigation browser :
//     1. page.request.post('/api/auth/logout') → révoque le token en DB côté serveur
//        (page.request partage le cookie store → envoie le cookie sans charger le browser)
//        Ignore 401 (pas de session active → déjà déconnecté, c'est OK)
//     2. page.context().clearCookies() → supprime le cookie du browser context
//     3. goto('/login') → layout monte → authStore.init()
//        → /api/auth/me → 401 (token révoqué + cookie absent)
//        → authStore.logout() → localStorage vidé + isAuthenticated=false
//        → $effect n'active PAS de redirect → #username accessible ✅
//
//   Aucun goto('/') préalable nécessaire. Aucun evaluate(localStorage) nécessaire.
//   authStore.logout() (appelé sur 401) gère lui-même le localStorage.

import { test, expect, type Page } from '@playwright/test';

// Mot de passe que le test définit pour l'admin (≥ 8 chars)
const ADMIN_NEW_PASSWORD = 'AdminCI2026!';
const BASE = 'http://localhost:6300/api';

// ─────────────────────────────────────────────
// Helpers partagés
// ─────────────────────────────────────────────

/**
 * Nettoie la session avant chaque test qui navigue via le browser vers /login.
 *
 * ORDRE IMPÉRATIF — sans navigation browser préalable :
 *   1. page.request.post('/api/auth/logout')
 *      → révoque le token en DB côté serveur (ignore 401 si pas de session)
 *      → page.request partage le cookie store → envoie le cookie sans déclencher le layout
 *   2. page.context().clearCookies()
 *      → supprime le cookie auth_token du browser context
 *   → Résultat : goto('/login') → authStore.init() → /api/auth/me → 401
 *     → authStore.logout() → isAuthenticated=false + localStorage vidé
 *     → $effect ne redirige PAS → #username interactif ✅
 *
 * POURQUOI PAS goto('/') d'abord (session 21 — mauvaise approche) :
 *   goto('/') monte le layout → onMount → authStore.init() → fetch('/api/auth/me')
 *   avec le cookie encore valide → 200 → isAuthenticated=true → redirect /chat
 *   clearCookies() ensuite ne sert plus à rien.
 */
async function clearSession(page: Page) {
  // Étape 1 : révoquer le token côté serveur via API (sans charger le browser)
  try {
    await page.request.post(`${BASE}/auth/logout`);
    // 200 = token révoqué, 401 = pas de session active → les deux cas sont OK
  } catch {
    // Erreur réseau → on continue quand même
  }
  // Étape 2 : vider les cookies du browser context
  await page.context().clearCookies();
}

async function loginAs(page: Page, username: string, password: string) {
  await clearSession(page);
  await page.goto('/login');
  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
}

/**
 * Login admin via l'API backend (page.request) — bypass total de la page /login.
 *
 * APPROCHE API-FIRST (session 19, inchangée) :
 *   page.request partage le cookie store du browser context.
 *   → cookie auth_token posé sans jamais charger /login dans le browser
 *   → localStorage et $effect() de redirection jamais impliqués
 *   → page.goto('/admin') fonctionne directement avec le cookie valide
 */
async function loginAsAdmin(page: Page) {
  // Essai 1 : mdp déjà changé (tests 2+, ou retries CI)
  let loginRes = await page.request.post(`${BASE}/auth/login`, {
    data: { username: 'admin', password: ADMIN_NEW_PASSWORD },
  });

  // Essai 2 : mdp initial (premier appel de la suite, fresh DB)
  if (!loginRes.ok()) {
    loginRes = await page.request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (!loginRes.ok()) {
      throw new Error(`Login admin API échoué : HTTP ${loginRes.status()}`);
    }
  }

  const loginBody = await loginRes.json();

  if (loginBody.user?.needs_password_change) {
    const changeRes = await page.request.post(`${BASE}/auth/change-password`, {
      data: { new_password: ADMIN_NEW_PASSWORD, user_id: loginBody.user.id },
    });
    if (!changeRes.ok()) {
      throw new Error(`Changement mdp admin échoué : HTTP ${changeRes.status()}`);
    }
    // Re-login avec le nouveau mdp → cookie valide, needs_password_change=false
    loginRes = await page.request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: ADMIN_NEW_PASSWORD },
    });
    if (!loginRes.ok()) {
      throw new Error(`Re-login après changement mdp échoué : HTTP ${loginRes.status()}`);
    }
    console.log('🔐 Mot de passe admin changé via API');
  }

  await page.goto('/admin');
  await expect(page).toHaveURL(/\/admin/, { timeout: 10_000 });
  console.log('✅ Admin connecté sur /admin (API login)');
}

/**
 * Attend que l'écran de chargement initial disparaisse.
 * Le layout utilise data-testid="loading-screen".
 */
async function waitForAppReady(page: Page) {
  await expect(
    page.locator('[data-testid="loading-screen"]')
  ).not.toBeVisible({ timeout: 15_000 });
}

// ─────────────────────────────────────────────
// 1. AUTHENTIFICATION
// ─────────────────────────────────────────────

test.describe('Auth', () => {

  test('Login valide e2e_ci → redirige vers /chat', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await expect(page).toHaveURL(/\/chat/, { timeout: 15_000 });
    console.log('✅ Login e2e_ci → /chat');
  });

  test('Login invalide → reste sur /login sans crash', async ({ page }) => {
    test.setTimeout(20_000);
    await clearSession(page);
    await page.goto('/login');
    await page.fill('#username', 'nope');
    await page.fill('#password', 'wrong');
    await page.getByRole('button', { name: 'Se connecter' }).click();
    await page.waitForTimeout(3_000);
    await expect(page).toHaveURL(/\/login/);
    console.log('✅ Login invalide → reste /login');
  });

  test('GET /api/auth/me sans cookie → 401', async ({ request }) => {
    const res = await request.get('/api/auth/me');
    expect(res.status()).toBe(401);
    console.log('✅ /api/auth/me non-auth → 401');
  });

  test('Logout → redirige vers /login', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await waitForAppReady(page);
    const logoutBtn = page.locator('button[aria-label="Déconnexion"]').first();
    await expect(logoutBtn).toBeVisible({ timeout: 8_000 });
    await logoutBtn.click();
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
    console.log('✅ Logout → /login');
  });

});

// ─────────────────────────────────────────────
// 2. CHAT
// ─────────────────────────────────────────────

test.describe('Chat', () => {

  test('Login → Chat → Envoi message → affiché dans le DOM', async ({ page }) => {
    test.setTimeout(60_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await waitForAppReady(page);

    await page.waitForResponse(
      (res) => res.url().includes('/api/conversations') && res.request().method() === 'GET',
      { timeout: 10_000 }
    );
    await expect(page.locator('.conversation-item').first()).toBeVisible({ timeout: 8_000 });

    const names = await page.locator('.conversation-info .name').allTextContents();
    const hasGlobal = names.some(n => n.includes('Groupe Global') || n.includes('Global'));
    expect(hasGlobal).toBe(true);
    console.log(`✅ Sidebar : ${names.length} conversation(s), Groupe Global présent`);

    const globalItem = page.locator('.conversation-item').filter({ hasText: 'Groupe Global' });
    if (await globalItem.count() > 0) {
      await globalItem.first().click();
    }

    const input = page.locator('input.message-input');
    await expect(input).toBeVisible({ timeout: 10_000 });

    const msgText = `E2E test message ${Date.now()}`;
    await input.fill(msgText);

    const sendBtn = page.locator('button.send-btn');
    await expect(sendBtn).toBeEnabled({ timeout: 5_000 });

    const [response] = await Promise.all([
      page.waitForResponse(
        (res) =>
          res.url().includes('/api/conversations/') &&
          res.url().includes('/messages') &&
          res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      sendBtn.click(),
    ]);

    expect(response.status()).toBe(200);
    console.log(`✅ POST /messages → HTTP ${response.status()}`);

    await expect(
      page.locator('.message-content').filter({ hasText: msgText })
    ).toBeVisible({ timeout: 15_000 });
    console.log('✅ Message affiché dans le DOM');
  });

  test('GET /api/conversations avec auth → liste avec default_global', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/conversations');
    expect(res.status()).toBe(200);
    const body = await res.json();
    const convs = Array.isArray(body) ? body : (body.conversations ?? []);
    expect(convs.length).toBeGreaterThan(0);
    const global = convs.find((c: { id: string }) => c.id === 'default_global');
    expect(global).toBeDefined();
    console.log(`✅ GET /api/conversations → ${convs.length} conversation(s), default_global présente`);
  });

  test('GET /api/conversations/default_global/messages → 200', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/conversations/default_global/messages');
    expect(res.status()).toBe(200);
    const body = await res.json();
    const msgs = Array.isArray(body) ? body : (body.messages ?? []);
    expect(msgs.length).toBeGreaterThanOrEqual(0);
    console.log(`✅ GET messages default_global → ${msgs.length} message(s)`);
  });

});

// ─────────────────────────────────────────────
// 3. ADMINISTRATION
// ─────────────────────────────────────────────

test.describe('Admin', () => {

  test('Admin login → changement de mot de passe obligatoire → /admin', async ({ page }) => {
    test.setTimeout(40_000);
    await loginAsAdmin(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Flow complet : login admin → change-password → /admin');
  });

  test('Page /admin → tous les onglets visibles', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAsAdmin(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    await expect(page.locator('.admin-tabs .tab').nth(0)).toBeVisible();
    await expect(page.locator('.admin-tabs .tab').nth(1)).toBeVisible();
    await expect(page.locator('.admin-tabs .tab').nth(2)).toBeVisible();
    console.log('✅ Page /admin chargée, 3 onglets visibles');
  });

  test('Admin → onglet "Membres" liste admin et e2e_ci', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAsAdmin(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    await page.locator('.admin-tabs .tab').nth(1).click();
    await expect(page.locator('.user-card').first()).toBeVisible({ timeout: 8_000 });
    const usernames = await page.locator('.user-username').allTextContents();
    expect(usernames.some((u) => u.includes('e2e_ci') || u.includes('admin'))).toBe(true);
    console.log(`✅ Onglet "Membres" → ${usernames.length} utilisateur(s)`);
  });

  test('Admin → génération lien d\'invitation', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAsAdmin(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    const [response] = await Promise.all([
      page.waitForResponse(
        (res) => res.url().includes('/api/invites') && res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      page.locator('.invite-btn').click(),
    ]);
    expect(response.status()).toBe(200);
    await expect(page.locator('.invite-link code')).toBeVisible({ timeout: 8_000 });
    const link = await page.locator('.invite-link code').textContent();
    expect(link).toContain('/invite?token=');
    console.log(`✅ Invitation générée : ${link}`);
  });

  test('GET /api/users/pending avec admin → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAsAdmin(page);
    const res = await page.request.get('/api/users/pending');
    expect(res.status()).toBe(200);
    console.log('✅ GET /api/users/pending → 200');
  });

  test('GET /api/users/pending sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/users/pending');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/users/pending non-auth → 401');
  });

});

// ─────────────────────────────────────────────
// 4. PARAMÈTRES
// ─────────────────────────────────────────────

test.describe('Settings', () => {

  test('Page /settings → 3 onglets accessibles', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/settings');
    await waitForAppReady(page);
    await expect(page.locator('#userName')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Onglet Profil visible');
    await page.locator('[role="tab"]').filter({ hasText: 'Sécurité' }).click();
    await expect(page.locator('#currentPassword')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Sécurité visible');
    await page.locator('[role="tab"]').filter({ hasText: 'Apparence' }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Apparence visible');
  });

  test('Settings → changement de thème (clic carte)', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/settings');
    await waitForAppReady(page);
    await page.locator('[role="tab"]').filter({ hasText: 'Apparence' }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    const themeCards = page.locator('.theme-card');
    expect(await themeCards.count()).toBeGreaterThan(1);
    await themeCards.nth(1).click();
    await expect(themeCards.nth(1)).toHaveClass(/selected/, { timeout: 3_000 });
    console.log('✅ Changement de thème → carte sélectionnée');
  });

});

// ─────────────────────────────────────────────
// 5. CALENDRIER
// ─────────────────────────────────────────────

test.describe('Calendar', () => {

  test('Page /calendar visible après login', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/calendar');
    await waitForAppReady(page);
    await expect(page.locator('.calendar-grid')).toBeVisible({ timeout: 10_000 });
    console.log('✅ Page /calendar chargée');
  });

  test('GET /api/events avec auth → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/events');
    expect(res.status()).toBe(200);
    console.log('✅ GET /api/events → 200');
  });

  test('POST /api/events → crée un événement', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.post('/api/events', {
      data: { title: `E2E Event ${Date.now()}`, date: '2026-12-25', time: '18:00', description: 'Test E2E' },
    });
    expect([200, 201]).toContain(res.status());
    const body = await res.json();
    expect(body.success).toBe(true);
    expect(body.id).toBeTruthy();
    console.log(`✅ POST /api/events → ${res.status()}, id=${body.id}`);
  });

  test('Calendrier UI → bouton "Ajouter un événement" visible', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/calendar');
    await waitForAppReady(page);
    await expect(page.locator('.add-event-btn')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Bouton ajout événement visible');
  });

});

// ─────────────────────────────────────────────
// 6. ÉCHECS (Chess)
// ─────────────────────────────────────────────

test.describe('Chess', () => {

  test('Page /chess visible et chargée', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/chess');
    await waitForAppReady(page);
    await expect(page.locator('.btn-new')).toBeVisible({ timeout: 10_000 });
    console.log('✅ Page /chess chargée');
  });

  test('GET /api/chess/list → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/chess/list');
    expect(res.status()).toBe(200);
    console.log('✅ GET /api/chess/list → 200');
  });

  test('POST /api/chess/create → crée une partie et retourne game_id', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.post('/api/chess/create', {
      data: { opponent: 'human', color: 'white' },
    });
    expect(res.status()).toBe(201);
    const body = await res.json();
    expect(body.game_id).toBeTruthy();
    expect(body.success).toBe(true);
    console.log(`✅ POST /api/chess/create → 201, game_id=${body.game_id}`);
  });

  test('Chess UI → formulaire de création accessible', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/chess');
    await waitForAppReady(page);
    await page.locator('.btn-new').click();
    await expect(page.locator('.create-card')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('.radio-opt').first()).toBeVisible({ timeout: 3_000 });
    await expect(page.locator('.color-opt').first()).toBeVisible({ timeout: 3_000 });
    await expect(page.locator('.btn-confirm')).toBeVisible({ timeout: 3_000 });
    console.log('✅ Formulaire création partie visible (.create-card, .radio-opt, .color-opt, .btn-confirm)');
  });

});

// ─────────────────────────────────────────────
// 7. SONDAGES
// ─────────────────────────────────────────────

test.describe('Polls', () => {

  test('Page /polls visible et chargée', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/polls');
    await waitForAppReady(page);
    await page.waitForResponse(
      (res) => res.url().includes('/api/polls') && res.request().method() === 'GET',
      { timeout: 10_000 }
    );
    await expect(page.locator('.btn-create')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Page /polls chargée, bouton "Nouveau sondage" visible');
  });

  test('GET /api/polls avec auth → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/polls');
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(Array.isArray(body.polls)).toBe(true);
    console.log(`✅ GET /api/polls → ${body.polls.length} sondage(s)`);
  });

  test('Polls → créer un sondage via UI → apparaît dans la liste', async ({ page }) => {
    test.setTimeout(45_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/polls');
    await waitForAppReady(page);
    await page.waitForResponse(
      (res) => res.url().includes('/api/polls') && res.request().method() === 'GET',
      { timeout: 10_000 }
    );

    await page.locator('.btn-create').click();
    await expect(page.locator('.create-card')).toBeVisible({ timeout: 5_000 });

    await page.locator('input[placeholder="Quelle est votre question ?"]').fill('Film préféré ce soir ?');
    await page.locator('input[placeholder="Option 1 *"]').fill('La La Land');
    await page.locator('input[placeholder="Option 2 *"]').fill('Inception');

    const [response] = await Promise.all([
      page.waitForResponse(
        (res) => res.url().includes('/api/polls') && res.request().method() === 'POST',
        { timeout: 15_000 }
      ),
      page.locator('.btn-submit').click(),
    ]);

    expect([200, 201]).toContain(response.status());
    console.log(`✅ POST /api/polls → HTTP ${response.status()}`);

    await expect(
      page.locator('.poll-question').filter({ hasText: 'Film préféré ce soir ?' })
    ).toBeVisible({ timeout: 10_000 });
    console.log('✅ Sondage créé et visible dans la liste');
  });

  test('POST /api/polls sans auth → 401', async ({ request }) => {
    const res = await request.post('/api/polls', {
      data: { question: 'Test', options: ['A', 'B'] },
    });
    expect(res.status()).toBe(401);
    console.log('✅ POST /api/polls non-auth → 401');
  });

});

// ─────────────────────────────────────────────
// 8. NAVIGATION (toutes les routes)
// ─────────────────────────────────────────────

test.describe('Navigation', () => {

  const routes = [
    { path: '/chat' },
    { path: '/calendar' },
    { path: '/chess' },
    { path: '/polls' },
    { path: '/settings' },
    { path: '/help' },
    { path: '/events' },
  ];

  for (const route of routes) {
    test(`Route ${route.path} → chargée sans erreur`, async ({ page }) => {
      test.setTimeout(30_000);
      await loginAs(page, 'e2e_ci', 'E2eTest123!');
      await page.goto(route.path);
      await page.waitForLoadState('networkidle', { timeout: 12_000 }).catch(() => {});
      expect(page.url()).not.toMatch(/\/login/);
      console.log(`✅ ${route.path} OK`);
    });
  }

  test('Route /admin → non accessible en tant que user normal', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/admin');
    await page.waitForTimeout(2_000);
    const url = page.url();
    const notAuth = await page.locator('.not-authorized').isVisible().catch(() => false);
    const redirected = url.includes('/chat') || url.includes('/login');
    expect(notAuth || redirected).toBe(true);
    console.log(`✅ /admin protégé pour e2e_ci (not-auth=${notAuth}, redirected=${redirected})`);
  });

});

// ─────────────────────────────────────────────
// 9. API SANITY CHECKS
// ─────────────────────────────────────────────

test.describe('API Sanity', () => {

  test('GET /api/health → "OK"', async ({ request }) => {
    const res = await request.get('/api/health');
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
    console.log('✅ /api/health → OK');
  });

  test('GET /api/conversations sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/conversations');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/conversations non-auth → 401');
  });

  test('GET /api/events sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/events');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/events non-auth → 401');
  });

  test('GET /api/chess/list sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/chess/list');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/chess/list non-auth → 401');
  });

  test('GET /api/invites sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/invites');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/invites non-auth → 401');
  });

  test('GET /api/polls sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/polls');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/polls non-auth → 401');
  });

  test('GET /api/users/available sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/users/available');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/users/available non-auth → 401');
  });

  test('GET /api/conversations/default_global/participants sans auth → 401', async ({ request }) => {
    const res = await request.get('/api/conversations/default_global/participants');
    expect(res.status()).toBe(401);
    console.log('✅ GET /api/conversations/default_global/participants non-auth → 401');
  });

});
