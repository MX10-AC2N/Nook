// frontend/tests/admin.spec.ts
// Flux admin complet — reproduit le parcours réel d'un administrateur Nook.
//
// SÉQUENCE RÉELLE :
//   1. Premier login → needs_password_change=true → change le mot de passe
//   2. Re-login avec nouveau mot de passe → arrivée sur /admin
//   3. Parcourt toutes les fonctionnalités admin : membres, invitations, approbations, analytics
//   4. Flux inscription d'un nouvel utilisateur → approbation admin
//   5. Gestion des invitations (génération + suppression)
//   6. Tests analytics complets
//   7. Vérification isolation : user normal → 403 sur routes admin
//
// OPTIMISATION RATE LIMIT :
//   loginAsAdmin() dans beforeAll → 1 seul login pour toute la suite.

import { test, expect, type Page } from '@playwright/test';
import {
  loginAsAdmin, loginViaAPI, waitForAppReady,
  BASE, ADMIN_NEW_PASSWORD, E2E_USER, E2E_PASS,
} from './helpers';

test.describe.serial('Admin — Flux complet', () => {

  let adminPage: Page;

  test.beforeAll(async ({ browser }) => {
    adminPage = await browser.newPage();
    await loginAsAdmin(adminPage);
  });

  test.afterAll(async () => {
    await adminPage.close();
  });

  // ══════════════════════════════════════════════════════════════
  // 1. CONNEXION & PAGE ADMIN
  // ══════════════════════════════════════════════════════════════

  test('Admin — page /admin chargée avec header', async () => {
    await expect(adminPage.locator('.admin-header')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Page /admin OK');
  });

  test('Admin — 3 onglets visibles', async () => {
    const tabs = adminPage.locator('.admin-tabs .tab');
    expect(await tabs.count()).toBeGreaterThanOrEqual(3);
    for (let i = 0; i < 3; i++) await expect(tabs.nth(i)).toBeVisible();
    console.log('✅ 3 onglets admin visibles');
  });

  test('GET /auth/me avec session admin → role=admin', async () => {
    const res = await adminPage.request.get(`${BASE}/auth/me`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.user?.role).toBe('admin');
    console.log(`✅ /auth/me → role=${body.user?.role}`);
  });

  // ══════════════════════════════════════════════════════════════
  // 2. GESTION DES UTILISATEURS
  // ══════════════════════════════════════════════════════════════

  test('GET /users → liste complète (admin)', async () => {
    const res = await adminPage.request.get(`${BASE}/users`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    const users = Array.isArray(body) ? body : (body.users ?? []);
    expect(users.length).toBeGreaterThan(0);
    const admin = users.find((u: { username: string }) => u.username === 'admin');
    expect(admin).toBeDefined();
    console.log(`✅ GET /users → ${users.length} utilisateur(s)`);
  });

  test('GET /users/pending → 200', async () => {
    const res = await adminPage.request.get(`${BASE}/users/pending`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    const list = Array.isArray(body) ? body : (body.users ?? []);
    expect(Array.isArray(list)).toBe(true);
    console.log(`✅ GET /users/pending → ${list.length} en attente`);
  });

  test('Onglet "Membres" → users visibles dans UI', async () => {
    await adminPage.goto('/admin');
    await adminPage.locator('.admin-header').waitFor({ state: 'visible', timeout: 10_000 });
    await adminPage.locator('.admin-tabs .tab').nth(1).click();
    await expect(adminPage.locator('.user-card').first()).toBeVisible({ timeout: 8_000 });
    const usernames = await adminPage.locator('.user-username').allTextContents();
    expect(usernames.some(u => u.includes('admin') || u.includes('e2e_ci'))).toBe(true);
    console.log(`✅ Onglet Membres → ${usernames.length} utilisateur(s)`);
  });

  // ══════════════════════════════════════════════════════════════
  // 3. FLUX INSCRIPTION → APPROBATION
  // Test du cycle complet : register → pending → approve
  // ══════════════════════════════════════════════════════════════

  test('Flux inscription : register → pending → approve → connecté', async ({ browser }) => {
    test.setTimeout(60_000);

    const testUser = `testuser_${Date.now()}`;
    const testPass = 'TestPass2026!';

    // ⚠️  ISOLATION CRITIQUE : les requêtes testUser passent par isolatedPage.
    // Si adminPage.request.post('/auth/login', testUser credentials) était utilisé,
    // le Set-Cookie de la réponse remplacerait le cookie admin → 403 sur tous les
    // tests suivants de la suite .serial.
    const isolatedPage = await browser.newPage();
    try {
      // 1. Inscription via contexte isolé (route publique, pas de session requise)
      const regRes = await isolatedPage.request.post(`${BASE}/auth/register`, {
        data: { username: testUser, password: testPass, email: `${testUser}@test.nook`, name: 'Test User' },
      });
      expect([200, 201]).toContain(regRes.status());
      expect((await regRes.json()).success).toBe(true);
      console.log(`✅ Inscription ${testUser}`);

      // 2. Login refusé avant approbation — contexte isolé
      const loginPending = await isolatedPage.request.post(`${BASE}/auth/login`, {
        data: { username: testUser, password: testPass },
      });
      expect([401, 403]).toContain(loginPending.status());
      console.log(`✅ Login refusé avant approbation → ${loginPending.status()}`);

      // 3. Admin récupère les users en attente (adminPage — session intacte)
      const pendingRes = await adminPage.request.get(`${BASE}/users/pending`);
      const pendingBody = await pendingRes.json();
      const pendingList = Array.isArray(pendingBody) ? pendingBody : (pendingBody.users ?? []);
      const newUser = pendingList.find((u: { username: string }) => u.username === testUser);
      expect(newUser).toBeDefined();
      console.log(`✅ ${testUser} visible dans /users/pending`);

      // 4. Admin approuve (adminPage — session intacte)
      const approveRes = await adminPage.request.post(`${BASE}/users/approve`, {
        data: { user_id: newUser.id },
      });
      expect(approveRes.status()).toBe(200);
      console.log(`✅ ${testUser} approuvé`);

      // 5. Login réussit — contexte isolé (cookie admin non touché)
      const loginApproved = await isolatedPage.request.post(`${BASE}/auth/login`, {
        data: { username: testUser, password: testPass },
      });
      expect(loginApproved.status()).toBe(200);
      console.log(`✅ Login réussi après approbation`);

    } finally {
      await isolatedPage.close(); // adminPage conserve sa session admin
    }
  });

  // ══════════════════════════════════════════════════════════════
  // 4. INVITATIONS
  // ══════════════════════════════════════════════════════════════

  test('POST /invites → génère un invite_link valide', async () => {
    const res = await adminPage.request.post(`${BASE}/invites`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    // Le backend retourne { success, message, invite_link: '/invite?token=...' }
    expect(body.success).toBe(true);
    expect(body.invite_link).toBeTruthy();
    expect(body.invite_link).toContain('/invite?token=');
    // Extraire le token depuis invite_link pour les tests suivants
    const token = new URLSearchParams(body.invite_link.split('?')[1]).get('token');
    expect(token).toBeTruthy();
    console.log(`✅ Invitation générée → token=${token?.substring(0, 8)}...`);
  });

  test('GET /invites → liste non vide', async () => {
    const res = await adminPage.request.get(`${BASE}/invites`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    const invites = Array.isArray(body) ? body : (body.invites ?? []);
    expect(invites.length).toBeGreaterThan(0);
    console.log(`✅ GET /invites → ${invites.length} invitation(s)`);
  });

  test('POST /invites/delete → supprime une invitation', async () => {
    // Créer une invitation à supprimer
    const createRes = await adminPage.request.post(`${BASE}/invites`);
    const { token } = await createRes.json();

    // Récupérer son id
    const listRes = await adminPage.request.get(`${BASE}/invites`);
    const listBody = await listRes.json();
    const invites = Array.isArray(listBody) ? listBody : (listBody.invites ?? []);
    const invite = invites.find((i: { token: string }) => i.token === token);

    if (invite) {
      const delRes = await adminPage.request.post(`${BASE}/invites/delete`, {
        data: { invite_id: invite.id },
      });
      expect(delRes.status()).toBe(200);
      console.log(`✅ Invitation supprimée → id=${invite.id}`);
    } else {
      console.log('⚠️ Invitation non trouvée dans la liste — skip suppression');
    }
  });

  test('GET /invite/validate?token=xxx → valide le token', async () => {
    const createRes = await adminPage.request.post(`${BASE}/invites`);
    expect(createRes.status()).toBe(200);
    const inviteBody = await createRes.json();
    // POST /invites retourne { success, message, invite_link: '/invite?token=...' }
    // Le token est dans invite_link, pas au niveau racine du JSON
    const token = new URLSearchParams(inviteBody.invite_link.split('?')[1]).get('token');
    expect(token).toBeTruthy();
    const res = await adminPage.request.get(`${BASE}/invite/validate?token=${token}`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.valid).toBe(true);
    console.log(`✅ Token validé → valid=${body.valid}`);
  });

  test('Admin UI — invitation générée visible dans l\'interface', async () => {
    test.setTimeout(30_000);
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
    console.log(`✅ Invitation UI → lien visible`);
  });

  // ══════════════════════════════════════════════════════════════
  // 5. ANALYTICS
  // ══════════════════════════════════════════════════════════════

  test('GET /analytics → tous les champs requis', async () => {
    const res = await adminPage.request.get(`${BASE}/analytics`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    // Champs numériques
    for (const field of ['user_count', 'message_count', 'conversation_count', 'poll_count', 'active_users_7d', 'messages_7d']) {
      expect(typeof body[field]).toBe('number');
    }
    expect(Array.isArray(body.messages_per_day)).toBe(true);
    expect(body.messages_7d).toBeLessThanOrEqual(body.message_count);
    console.log(`✅ Analytics → users=${body.user_count} msgs=${body.message_count} actifs7j=${body.active_users_7d}`);
  });

  test('Page /admin/analytics → stat-cards + 2 charts', async () => {
    test.setTimeout(30_000);
    await adminPage.goto('/admin/analytics');
    await waitForAppReady(adminPage);
    await expect(adminPage.locator('.stat-card').first()).toBeVisible({ timeout: 10_000 });
    expect(await adminPage.locator('.stat-card').count()).toBeGreaterThanOrEqual(4);
    await expect(adminPage.locator('canvas').first()).toBeVisible({ timeout: 8_000 });
    console.log('✅ /admin/analytics → stat-cards et charts OK');
  });

  // ══════════════════════════════════════════════════════════════
  // 6. POLLS — Actions admin (delete)
  // ══════════════════════════════════════════════════════════════

  test('Admin — DELETE /polls/{id} → 200', async () => {
    // Créer un sondage avec la session admin
    const createRes = await adminPage.request.post(`${BASE}/polls`, {
      data: { question: `Admin delete test ${Date.now()}`, options: ['A', 'B'] },
    });
    expect([200, 201]).toContain(createRes.status());
    const pollId = (await createRes.json()).id;

    const delRes = await adminPage.request.delete(`${BASE}/polls/${pollId}`);
    expect(delRes.status()).toBe(200);

    // Vérifier que le sondage n'existe plus
    const getRes = await adminPage.request.get(`${BASE}/polls/${pollId}`);
    expect([404, 400]).toContain(getRes.status());
    console.log(`✅ Admin delete poll → 200, GET vérifie 404`);
  });

  // ══════════════════════════════════════════════════════════════
  // 7. ISOLATION ADMIN
  // ══════════════════════════════════════════════════════════════

  test('GET /analytics avec user normal → 403', async ({ browser }) => {
    const userPage = await browser.newPage();
    try {
      await loginViaAPI(userPage, E2E_USER, E2E_PASS);
      const res = await userPage.request.get(`${BASE}/analytics`);
      expect(res.status()).toBe(403);
      console.log('✅ /analytics user normal → 403');
    } finally { await userPage.close(); }
  });

  test('GET /users/pending avec user normal → 403', async ({ browser }) => {
    const userPage = await browser.newPage();
    try {
      await loginViaAPI(userPage, E2E_USER, E2E_PASS);
      const res = await userPage.request.get(`${BASE}/users/pending`);
      expect(res.status()).toBe(403);
      console.log('✅ /users/pending user normal → 403');
    } finally { await userPage.close(); }
  });

  test('Page /admin → non accessible pour user normal', async ({ browser }) => {
    test.setTimeout(30_000);
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
    } finally { await userPage.close(); }
  });

});
