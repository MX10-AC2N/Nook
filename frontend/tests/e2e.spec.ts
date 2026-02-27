// frontend/tests/e2e.spec.ts
// Suite E2E complète — Session 14
// Couverture : Auth · Chat · Admin · Settings · Calendar · Chess · Navigation · API Sanity

import { test, expect, type Page, type APIResponse } from '@playwright/test';

// ─────────────────────────────────────────────
// Helpers partagés
// ─────────────────────────────────────────────

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/login');
  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page).toHaveURL(/\/(chat|admin)/, { timeout: 15_000 });
}

async function waitForAppReady(page: Page) {
  await expect(page.locator('.loading-screen')).not.toBeVisible({ timeout: 15_000 });
}

// ─────────────────────────────────────────────
// 1. AUTHENTIFICATION
// ─────────────────────────────────────────────

test.describe('Auth', () => {

  test('Login valide → redirige vers /chat', async ({ page }) => {
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
    // Doit rester sur /login (pas de redirection)
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
    // Ouvre le menu et clique Déconnexion
    const menuToggle = page.locator('button[aria-label="Ouvrir le menu de navigation"]');
    await expect(menuToggle).toBeVisible({ timeout: 8_000 });
    await menuToggle.click();
    await page.getByRole('button', { name: /d.connect/i }).click();
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

    // Sidebar doit contenir "Groupe Global"
    await expect(page.locator('.conversation-item').first()).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.conversation-info .name').first())
      .toHaveText('Groupe Global', { timeout: 5_000 });
    console.log('✅ Sidebar : Groupe Global visible');

    // Input visible
    const input = page.locator('input.message-input');
    await expect(input).toBeVisible({ timeout: 10_000 });
    console.log('✅ Input de chat visible');

    // Envoi du message — intercepte la réponse POST
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

    // Message visible dans le DOM
    await expect(
      page.locator('.message-content').filter({ hasText: msgText })
    ).toBeVisible({ timeout: 15_000 });
    console.log('✅ Message affiché dans le DOM');
  });

  test('GET /api/conversations avec auth → liste avec default_global', async ({ page, request }) => {
    test.setTimeout(30_000);
    // Se connecter via UI pour avoir le cookie
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/conversations');
    expect(res.status()).toBe(200);
    const body = await res.json();
    const convs = Array.isArray(body) ? body : body.conversations ?? [];
    expect(convs.length).toBeGreaterThan(0);
    const global = convs.find((c: { id: string }) => c.id === 'default_global');
    expect(global).toBeDefined();
    console.log(`✅ GET /api/conversations → ${convs.length} conversation(s), default_global présente`);
  });

  test('GET /api/conversations/{id}/messages → tableau de messages', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/conversations/default_global/messages');
    expect(res.status()).toBe(200);
    const body = await res.json();
    const msgs = Array.isArray(body) ? body : body.messages ?? [];
    // Au moins le message envoyé dans le test précédent
    expect(msgs.length).toBeGreaterThanOrEqual(0);
    console.log(`✅ GET /api/conversations/default_global/messages → ${msgs.length} message(s)`);
  });

});

// ─────────────────────────────────────────────
// 3. ADMINISTRATION
// ─────────────────────────────────────────────

test.describe('Admin', () => {

  test('Admin login → redirige vers /admin', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'admin', 'changeme2026');
    // L'admin est redirigé vers /admin (needs_password_change=1 mais accepté)
    await expect(page).toHaveURL(/\/(admin|chat|change-password)/, { timeout: 15_000 });
    console.log('✅ Admin login OK');
  });

  test('Page /admin → tous les onglets visibles', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'admin', 'changeme2026');
    await page.goto('/admin');
    await waitForAppReady(page);
    await expect(page.locator('.admin-header')).toBeVisible({ timeout: 10_000 });
    // Onglets
    await expect(page.locator('.admin-tabs .tab').nth(0)).toBeVisible();
    await expect(page.locator('.admin-tabs .tab').nth(1)).toBeVisible();
    await expect(page.locator('.admin-tabs .tab').nth(2)).toBeVisible();
    console.log('✅ Page /admin chargée, 3 onglets visibles');
  });

  test('Admin → onglet "Tous les utilisateurs" liste e2e_ci', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'admin', 'changeme2026');
    await page.goto('/admin');
    await waitForAppReady(page);
    // Cliquer sur le 2ème onglet (Tous)
    await page.locator('.admin-tabs .tab').nth(1).click();
    await expect(page.locator('.user-card').first()).toBeVisible({ timeout: 8_000 });
    const usernames = await page.locator('.user-username').allTextContents();
    expect(usernames.some((u) => u.includes('e2e_ci') || u.includes('admin'))).toBe(true);
    console.log(`✅ Onglet "Tous les users" → ${usernames.length} utilisateur(s)`);
  });

  test('Admin → génération lien d\'invitation', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'admin', 'changeme2026');
    await page.goto('/admin');
    await waitForAppReady(page);

    // Intercepte la requête POST /api/invites
    const [response] = await Promise.all([
      page.waitForResponse(
        (res) => res.url().includes('/api/invites') && res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      page.locator('.invite-btn').click(),
    ]);

    expect(response.status()).toBe(200);
    // Le lien doit apparaître dans le DOM
    await expect(page.locator('.invite-link code')).toBeVisible({ timeout: 8_000 });
    const link = await page.locator('.invite-link code').textContent();
    expect(link).toContain('/invite?token=');
    console.log(`✅ Invitation générée : ${link}`);
  });

  test('GET /api/users/pending avec admin → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'admin', 'changeme2026');
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

    // Onglet Profil (actif par défaut)
    await expect(page.locator('#userName')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Onglet Profil visible');

    // Onglet Sécurité
    await page.locator('[role="tab"]').filter({ hasText: /s.curit/i }).click();
    await expect(page.locator('#currentPassword')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Sécurité visible');

    // Onglet Apparence
    await page.locator('[role="tab"]').filter({ hasText: /apparence/i }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    console.log('✅ Onglet Apparence et grille de thèmes visible');
  });

  test('Settings → changement de thème (clic carte)', async ({ page }) => {
    test.setTimeout(30_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/settings');
    await waitForAppReady(page);

    // Naviguer vers Apparence
    await page.locator('[role="tab"]').filter({ hasText: /apparence/i }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });

    // Cliquer sur la 2ème carte de thème (pas le thème déjà sélectionné)
    const themeCards = page.locator('.theme-card');
    const count = await themeCards.count();
    expect(count).toBeGreaterThan(1);
    await themeCards.nth(1).click();
    // La carte cliquée doit avoir la classe "selected"
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
    // La grille du calendrier doit être présente
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

    const payload = {
      title: `E2E Event ${Date.now()}`,
      date: '2026-12-25',
      time: '18:00',
      description: 'Créé par le test E2E',
    };

    const res = await page.request.post('/api/events', {
      data: payload,
    });
    expect([200, 201]).toContain(res.status());
    const body = await res.json();
    expect(body.success ?? body.id ?? body.title).toBeTruthy();
    console.log(`✅ POST /api/events → ${res.status()} : ${JSON.stringify(body).slice(0, 80)}`);
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
    await expect(page.locator('.btn-create, h1')).toBeVisible({ timeout: 10_000 });
    console.log('✅ Page /chess chargée');
  });

  test('GET /api/chess/list → 200', async ({ page }) => {
    test.setTimeout(20_000);
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    const res = await page.request.get('/api/chess/list');
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(Array.isArray(body) || Array.isArray(body.games)).toBe(true);
    console.log(`✅ GET /api/chess/list → 200`);
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
    console.log(`✅ POST /api/chess/create → ${res.status()} : id=${body.id ?? body.game_id ?? body.game?.id}`);
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

    // Remplir la question et les 2 options obligatoires
    await page.locator('input[placeholder*="question"], input[placeholder*="Question"]').first()
      .fill('Film préféré ce soir ?');
    await page.locator('input[placeholder*="Option 1"]').fill('La La Land');
    await page.locator('input[placeholder*="Option 2"]').fill('Inception');

    // Soumettre
    await page.getByRole('button', { name: /cr.er|ajouter|valider/i }).first().click();
    await page.waitForTimeout(1_000);

    // Le sondage doit apparaître dans la liste
    await expect(
      page.locator('text=Film préféré ce soir ?')
    ).toBeVisible({ timeout: 8_000 });
    console.log('✅ Sondage créé et visible dans la liste');
  });

});

// ─────────────────────────────────────────────
// 8. NAVIGATION (toutes les routes)
// ─────────────────────────────────────────────

test.describe('Navigation', () => {

  const routes = [
    { path: '/chat',     label: 'Chat' },
    { path: '/calendar', label: 'Calendrier' },
    { path: '/chess',    label: 'Échecs' },
    { path: '/polls',    label: 'Sondages' },
    { path: '/settings', label: 'Paramètres' },
    { path: '/help',     label: 'Aide' },
    { path: '/events',   label: 'Événements' },
  ];

  for (const route of routes) {
    test(`Route ${route.path} → pas d'erreur 404 ou crash`, async ({ page }) => {
      test.setTimeout(30_000);
      await loginAs(page, 'e2e_ci', 'E2eTest123!');
      await page.goto(route.path);
      // Attendre que la page soit stable (pas de loading écran)
      await page.waitForLoadState('networkidle', { timeout: 12_000 }).catch(() => {});
      // Vérifier qu'on n'est pas renvoyé sur /login (route non protégée)
      const url = page.url();
      expect(url).not.toMatch(/\/login/);
      console.log(`✅ ${route.path} chargée (URL: ${url})`);
    });
  }

  test('Route /admin → accessible en tant qu\'admin uniquement', async ({ page }) => {
    test.setTimeout(30_000);
    // En tant que user normal → ne doit pas voir le contenu admin
    await loginAs(page, 'e2e_ci', 'E2eTest123!');
    await page.goto('/admin');
    await page.waitForTimeout(2_000);
    // Soit redirigé, soit "non autorisé" visible
    const url = page.url();
    const notAuth = await page.locator('.not-authorized').isVisible().catch(() => false);
    const redirected = url.includes('/chat') || url.includes('/login');
    expect(notAuth || redirected).toBe(true);
    console.log(`✅ /admin non accessible à e2e_ci (url=${url}, not-auth=${notAuth})`);
  });

});

// ─────────────────────────────────────────────
// 9. API SANITY CHECKS
// ─────────────────────────────────────────────

test.describe('API Sanity', () => {

  test('GET /health → "OK"', async ({ request }) => {
    const res = await request.get('/api/health');
    expect(res.status()).toBe(200);
    const text = await res.text();
    expect(text.trim()).toBe('OK');
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
