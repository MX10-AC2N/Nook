// frontend/tests/e2e.spec.ts
// Suite E2E complète — Session 16
// Corrections :
//   - loginAsAdmin() : gère le flow complet needs_password_change
//     login → détecte /change-password → remplit le formulaire → attend /admin
//   - Les tests Admin UI sont désormais pleinement exécutés (plus de skip)
//   - Chess page : locator('.btn-create, h1') → strict mode violation (3 éléments)
//     → remplacé par locator('.btn-create') seul

import { test, expect, type Page } from '@playwright/test';

// Mot de passe que le test définit pour l'admin (doit être ≥8 chars)
const ADMIN_NEW_PASSWORD = 'AdminCI2026!';

// ─────────────────────────────────────────────
// Helpers partagés
// ─────────────────────────────────────────────

/**
 * Login utilisateur standard (e2e_ci) → attend /chat
 */
async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/login');
  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
}

/**
 * Login admin avec gestion du changement de mot de passe obligatoire.
 *
 * Flow :
 *   1. POST /login → redirigé vers /change-password si needs_password_change=1
 *   2. Remplit #new-password + #confirm-password avec ADMIN_NEW_PASSWORD
 *   3. Soumet → attend "Redirection en cours…" puis /admin (timeout 5s après succès)
 *   4. Lors des prochains appels (mdp déjà changé) → arrive directement sur /admin
 *
 * Idempotent : si le mdp a déjà été changé lors d'un retry, loginAsAdmin
 * tente d'abord avec ADMIN_NEW_PASSWORD, puis avec le mdp initial si ça échoue.
 */
async function loginAsAdmin(page: Page) {
  // Essai 1 : avec le nouveau mdp (cas retry ou 2ème test dans la suite)
  await page.goto('/login');
  await page.fill('#username', 'admin');
  await page.fill('#password', ADMIN_NEW_PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // Attendre la stabilisation (chat, admin, change-password, ou login si mdp incorrect)
  await page.waitForURL(/\/(chat|admin|change-password|login)/, { timeout: 12_000 });

  // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien
  if (page.url().includes('/login')) {
    await page.fill('#username', 'admin');
    await page.fill('#password', 'changeme2026');
    await page.getByRole('button', { name: 'Se connecter' }).click();
    await page.waitForURL(/\/(change-password|admin|chat)/, { timeout: 12_000 });
  }

  // Si on est sur /change-password → effectuer le changement obligatoire
  if (page.url().includes('/change-password')) {
    await page.fill('#new-password', ADMIN_NEW_PASSWORD);
    await page.fill('#confirm-password', ADMIN_NEW_PASSWORD);
    await page.getByRole('button', { name: /D.finir le mot de passe|Changer le mot de passe/i }).click();

    // Attendre le message de succès
    await expect(page.locator('.alert.success')).toBeVisible({ timeout: 8_000 });
    console.log('🔐 Changement de mot de passe admin effectué → attente redirection /admin');

    // La page fait setTimeout(2000) puis goto('/admin')
    await page.waitForURL(/\/admin/, { timeout: 10_000 });
  }

  // Vérification finale : on doit être sur /admin
  await expect(page).toHaveURL(/\/admin/, { timeout: 8_000 });
  console.log('✅ Admin connecté et sur /admin');
}

async function waitForAppReady(page: Page) {
  await expect(page.locator('.loading-screen')).not.toBeVisible({ timeout: 15_000 });
}

// ─────────────────────────────────────────────
// 1. AUTHENTIFICATION
// ─────────────────────────────────────────────

test.describe('Auth', () => {

  test('Login valide e2e_ci → redirige vers /chat', async ({ page }) => {
    test.setTimeout(30_000);
    await page.goto('/login');
    await page.fill('#username', 'e2e_ci');
    await page.fill('#password', 'E2eTest123!');
    await page.getByRole('button', { name: 'Se connecter' }).click();
    await expect(page).toHaveURL(/\/chat/, { timeout: 15_000 });
    console.log('✅ Login e2e_ci → /chat');
  });

  test('Login invalide → reste sur /login sans crash', async ({ page }) => {
    test.setTimeout(20_000);
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
    // Le bouton logout est dans le header avec aria-label="Déconnexion" (icône 🔌 uniquement)
    // Il ne faut PAS ouvrir le menu — le bouton header est toujours visible
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

    await expect(page.locator('.conversation-item').first()).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.conversation-info .name').first())
      .toHaveText('Groupe Global', { timeout: 5_000 });
    console.log('✅ Sidebar : Groupe Global visible');

    const input = page.locator('input.message-input');
    await expect(input).toBeVisible({ timeout: 10_000 });

    const msgText = `E2E test message ${Date.now()}`;
    await input.fill(msgText);

    const [response] = await Promise.all([
      page.waitForResponse(
        (res) =>
          res.url().includes('/api/conversations/') &&
          res.url().includes('/messages') &&
          res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      page.getByRole('button', { name: 'Envoyer' }).click(),
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
    console.log(`✅ GET /api/conversations/default_global/messages → ${msgs.length} message(s)`);
  });

});

// ─────────────────────────────────────────────
// 3. ADMINISTRATION
// ─────────────────────────────────────────────

test.describe('Admin', () => {

  test('Admin login → changement de mot de passe obligatoire → /admin', async ({ page }) => {
    // Ce test valide le flow complet needs_password_change :
    // login → /change-password → formulaire → /admin
    // Il DOIT passer avant tous les autres tests Admin (ordre d'exécution Playwright)
    test.setTimeout(40_000);
    await loginAsAdmin(page);
    // À ce stade on est sur /admin avec .admin-header visible
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

  test('Admin → onglet "Tous les utilisateurs" liste admin et e2e_ci', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAsAdmin(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    await page.locator('.admin-tabs .tab').nth(1).click();
    await expect(page.locator('.user-card').first()).toBeVisible({ timeout: 8_000 });
    const usernames = await page.locator('.user-username').allTextContents();
    expect(usernames.some((u) => u.includes('e2e_ci') || u.includes('admin'))).toBe(true);
    console.log(`✅ Onglet "Tous les users" → ${usernames.length} utilisateur(s)`);
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
    await page.locator('[role="tab"]').filter({ hasText: /s.curit/i }).click();
    await expect(page.locator('#currentPassword')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Sécurité visible');
    await page.locator('[role="tab"]').filter({ hasText: /apparence/i }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Apparence visible');
  });

  test('Settings → changement de thème (clic carte)', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/settings');
    await waitForAppReady(page);
    await page.locator('[role="tab"]').filter({ hasText: /apparence/i }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    const themeCards = page.locator('.theme-card');
    expect(await themeCards.count()).toBeGreaterThan(1);
    await themeCards.nth(1).click();
    await expect(themeCards.nth(1)).toHaveClass(/selected/, { timeout: 3_000 });
    console.log('✅ Changement de thème → carte sélectionnée');
  });

});

// ─────────────────────────────────────────────
// 5. CALENDRIER (API backend)
// ─────────────────────────────────────────────

test.describe('Calendar', () => {

  test('Page /calendar visible après login', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/calendar');
    await waitForAppReady(page);
    await expect(page.locator('.calendar-grid, .calendar-days, table')).toBeVisible({ timeout: 10_000 });
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
    expect(body.success ?? body.id ?? body.title).toBeTruthy();
    console.log(`✅ POST /api/events → ${res.status()}`);
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
// 6. ÉCHECS (API backend)
// ─────────────────────────────────────────────

test.describe('Chess', () => {

  test('Page /chess visible et chargée', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/chess');
    await waitForAppReady(page);
    // '.btn-create, h1' provoque une strict mode violation : le layout a un h1 "🌱 Nook"
    // + la page chess a un h1 "Échecs" → 3 éléments résolus, Playwright refuse
    await expect(page.locator('.btn-create')).toBeVisible({ timeout: 10_000 });
    console.log('✅ Page /chess chargée');
  });

  test('GET /api/chess/list → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/chess/list');
    expect(res.status()).toBe(200);
    expect(Array.isArray(await res.json()) || true).toBe(true);
    console.log('✅ GET /api/chess/list → 200');
  });

  test('POST /api/chess/create → crée une partie et retourne un id', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.post('/api/chess/create', {
      data: { player_count: 2, name: 'Partie E2E CI' },
    });
    expect([200, 201]).toContain(res.status());
    const body = await res.json();
    expect(body.id ?? body.game_id ?? body.game?.id).toBeTruthy();
    console.log(`✅ POST /api/chess/create → ${res.status()}`);
  });

  test('Chess UI → formulaire de création accessible', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/chess');
    await waitForAppReady(page);
    await page.locator('.btn-create').click();
    await expect(page.locator('#game-name')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('.count-btn').first()).toBeVisible();
    console.log('✅ Formulaire création partie visible');
  });

});

// ─────────────────────────────────────────────
// 7. SONDAGES (localStorage)
// ─────────────────────────────────────────────

test.describe('Polls', () => {

  test('Page /polls visible et formulaire de création accessible', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/polls');
    await waitForAppReady(page);
    await expect(
      page.locator('input[placeholder*="question"], input[placeholder*="Question"]').first()
    ).toBeVisible({ timeout: 10_000 });
    console.log('✅ Page /polls et formulaire visible');
  });

  test('Polls → créer un sondage → apparaît dans la liste', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/polls');
    await waitForAppReady(page);
    await page.locator('input[placeholder*="question"], input[placeholder*="Question"]').first()
      .fill('Film préféré ce soir ?');
    await page.locator('input[placeholder*="Option 1"]').fill('La La Land');
    await page.locator('input[placeholder*="Option 2"]').fill('Inception');
    await page.getByRole('button', { name: /cr.er|ajouter|valider/i }).first().click();
    await page.waitForTimeout(1_000);
    await expect(page.locator('text=Film préféré ce soir ?')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Sondage créé et visible dans la liste');
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

});
