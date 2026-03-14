// frontend/tests/admin.spec.ts
// Flux admin complet — reproduit exactement ce que fait un admin réel.
//
// SÉQUENCE :
//   1. Premier login → needs_password_change=true → changement mdp obligatoire
//   2. Re-login avec nouveau mdp → arrivée sur /admin
//   3. Tests UI admin : onglets, membres, invitations, analytics
//   4. Tests API admin : /users/pending, /users, /invites
//   5. Vérification isolation : user normal → 403 sur routes admin
//
// LOGIN : 1 seul login via API dans beforeAll → partagé par tous les tests du describe.
// Aucun loginAs() répété → rate limit impossible à atteindre.

import { test, expect, type Page } from '@playwright/test';
import { loginAsAdmin, loginViaAPI, waitForAppReady, BASE, E2E_USER, E2E_PASS } from './helpers';

test.describe.serial('Admin — Flux complet', () => {

  // Session admin partagée par tous les tests
  let adminPage: Page;

  test.beforeAll(async ({ browser }) => {
    adminPage = await browser.newPage();
    await loginAsAdmin(adminPage);
  });

  test.afterAll(async () => {
    await adminPage.close();
  });

  // ── 1. Arrivée sur /admin ──────────────────────────────────────

  test('Admin connecté → page /admin avec header visible', async () => {
    await expect(adminPage.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Page /admin chargée avec header');
  });

  test('Page /admin → 3 onglets visibles', async () => {
    const tabs = adminPage.locator('.admin-tabs .tab');
    expect(await tabs.count()).toBeGreaterThanOrEqual(3);
    await expect(tabs.nth(0)).toBeVisible();
    await expect(tabs.nth(1)).toBeVisible();
    await expect(tabs.nth(2)).toBeVisible();
    console.log('✅ 3 onglets admin visibles');
  });

  // ── 2. Gestion des membres ─────────────────────────────────────

  test('Onglet "Membres" → liste admin et e2e_ci', async () => {
    await adminPage.locator('.admin-tabs .tab').nth(1).click();
    await expect(adminPage.locator('.user-card').first()).toBeVisible({ timeout: 8_000 });
    const usernames = await adminPage.locator('.user-username').allTextContents();
    expect(usernames.some(u => u.includes('e2e_ci') || u.includes('admin'))).toBe(true);
    console.log(`✅ Onglet Membres → ${usernames.length} utilisateur(s)`);
  });

  test('GET /api/users/pending avec admin → 200', async () => {
    const res = await adminPage.request.get(`${BASE}/users/pending`);
    expect(res.status()).toBe(200);
    console.log('✅ GET /api/users/pending → 200');
  });

  test('GET /api/users avec admin → 200 et liste non vide', async () => {
    const res = await adminPage.request.get(`${BASE}/users`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    const users = Array.isArray(body) ? body : (body.users ?? []);
    expect(users.length).toBeGreaterThan(0);
    console.log(`✅ GET /api/users → ${users.length} utilisateur(s)`);
  });

  // ── 3. Invitations ─────────────────────────────────────────────

  test('Admin → génération lien d\'invitation via UI', async () => {
    // Revenir sur l'onglet invitations (onglet 0 ou chercher le bouton)
    await adminPage.goto('/admin');
    await adminPage.locator('.admin-header').waitFor({ state: 'visible', timeout: 10_000 });

    const [response] = await Promise.all([
      adminPage.waitForResponse(
        res => res.url().includes('/api/invites') && res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      adminPage.locator('.invite-btn').click(),
    ]);
    expect(response.status()).toBe(200);
    await expect(adminPage.locator('.invite-link code')).toBeVisible({ timeout: 8_000 });
    const link = await adminPage.locator('.invite-link code').textContent();
    expect(link).toContain('/invite?token=');
    console.log(`✅ Invitation générée : ${link}`);
  });

  test('GET /api/invites avec admin → 200', async () => {
    const res = await adminPage.request.get(`${BASE}/invites`);
    expect(res.status()).toBe(200);
    console.log('✅ GET /api/invites → 200');
  });

  // ── 4. Analytics ───────────────────────────────────────────────

  test('GET /api/analytics avec admin → 200 et champs requis', async () => {
    const res = await adminPage.request.get(`${BASE}/analytics`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(typeof body.user_count).toBe('number');
    expect(typeof body.message_count).toBe('number');
    expect(typeof body.conversation_count).toBe('number');
    expect(typeof body.poll_count).toBe('number');
    expect(typeof body.active_users_7d).toBe('number');
    expect(typeof body.messages_7d).toBe('number');
    expect(Array.isArray(body.messages_per_day)).toBe(true);
    expect(body.messages_7d).toBeLessThanOrEqual(body.message_count);
    console.log(`✅ /api/analytics → users=${body.user_count}, msgs=${body.message_count}`);
  });

  test('Page /admin/analytics → stat-cards et canvas visible', async () => {
    await adminPage.goto('/admin/analytics');
    await waitForAppReady(adminPage);
    await expect(adminPage.locator('.stat-card').first()).toBeVisible({ timeout: 10_000 });
    expect(await adminPage.locator('.stat-card').count()).toBeGreaterThanOrEqual(4);
    await expect(adminPage.locator('canvas').first()).toBeVisible({ timeout: 8_000 });
    console.log('✅ /admin/analytics → stat-cards et canvas OK');
  });

  // ── 5. Isolation : user normal ne peut pas accéder aux routes admin ──

  test('GET /api/analytics avec user normal → 403', async ({ browser }) => {
    const userPage = await browser.newPage();
    try {
      await loginViaAPI(userPage, E2E_USER, E2E_PASS);
      const res = await userPage.request.get(`${BASE}/analytics`);
      expect(res.status()).toBe(403);
      console.log('✅ /api/analytics user normal → 403');
    } finally {
      await userPage.close();
    }
  });

  test('Page /admin → non accessible pour user normal', async ({ browser }) => {
    const userPage = await browser.newPage();
    try {
      await loginViaAPI(userPage, E2E_USER, E2E_PASS);
      await userPage.goto('/admin');
      await userPage.waitForTimeout(2_000);
      const url = userPage.url();
      const notAuth = await userPage.locator('.not-authorized').isVisible().catch(() => false);
      const redirected = url.includes('/chat') || url.includes('/login');
      expect(notAuth || redirected).toBe(true);
      console.log(`✅ /admin protégé : not-auth=${notAuth}, redirected=${redirected}`);
    } finally {
      await userPage.close();
    }
  });

});
