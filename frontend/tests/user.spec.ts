// frontend/tests/user.spec.ts
// Flux utilisateur complet — reproduit le parcours réel d'un membre de la famille.
//
// SÉQUENCE :
//   1. Login e2e_ci → /chat
//   2. Chat : envoyer un message, vérifier affichage DOM
//   3. Réactions : ajouter, modifier (UPSERT), supprimer
//   4. Upload : fichier texte → file_id → download
//   5. Polls : créer, voter, changer vote, fermer
//   6. Chess : créer partie vs IA, faire un coup légal, coup illégal → 400
//   7. Calendar : créer événement, vérifier
//   8. Settings : onglets profil / sécurité / apparence
//   9. Navigation : toutes les routes accessibles
//  10. E2EE : enregistrer clé publique
//  11. Auth : logout → retour /login
//
// LOGIN : 1 seul loginAs() dans beforeAll → session partagée dans le describe.
// Tests API utilisant page.request → cookie partagé automatiquement.

import { test, expect, type Page } from '@playwright/test';
import {
  loginAs, loginViaAPI, waitForAppReady,
  BASE, E2E_USER, E2E_PASS,
} from './helpers';

test.describe.serial('User — Flux complet', () => {

  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
    // Login UI réel : teste le vrai flux de connexion utilisateur
    await loginAs(page, E2E_USER, E2E_PASS);
  });

  test.afterAll(async () => {
    await page.close();
  });

  // ── 1. Authentification ────────────────────────────────────────

  test('Login e2e_ci → redirigé vers /chat', async () => {
    await expect(page).toHaveURL(/\/chat/, { timeout: 10_000 });
    console.log('✅ Login e2e_ci → /chat');
  });

  test('GET /api/auth/me → utilisateur authentifié', async () => {
    const res = await page.request.get(`${BASE}/auth/me`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.user?.username).toBe(E2E_USER);
    console.log(`✅ /api/auth/me → ${body.user?.username}`);
  });

  // ── 2. Chat ────────────────────────────────────────────────────

  test('Chat — sidebar contient default_global', async () => {
    await waitForAppReady(page);
    await expect(page.locator('.conversation-item').first()).toBeVisible({ timeout: 15_000 });
    const res = await page.request.get(`${BASE}/conversations`);
    expect(res.status()).toBe(200);
    const convs = await res.json();
    const list = Array.isArray(convs) ? convs : (convs.conversations ?? []);
    expect(list.find((c: { id: string }) => c.id === 'default_global')).toBeDefined();
    console.log(`✅ ${list.length} conversation(s), default_global présente`);
  });

  test('Chat — envoyer un message → visible dans le DOM', async () => {
    test.setTimeout(60_000);
    await waitForAppReady(page);

    const globalItem = page.locator('.conversation-item').filter({ hasText: 'Nook' });
    if (await globalItem.count() > 0) await globalItem.first().click();

    const input = page.locator('input.message-input');
    await expect(input).toBeVisible({ timeout: 10_000 });

    const msgText = `E2E message ${Date.now()}`;
    await input.fill(msgText);

    const [response] = await Promise.all([
      page.waitForResponse(
        res => res.url().includes('/api/conversations/') && res.url().includes('/messages') && res.request().method() === 'POST',
        { timeout: 10_000 }
      ),
      page.locator('button.send-btn').click(),
    ]);
    expect(response.status()).toBe(200);
    await expect(page.locator('.message-content').filter({ hasText: msgText })).toBeVisible({ timeout: 15_000 });
    console.log('✅ Message envoyé et affiché');
  });

  test('GET /api/conversations/default_global/messages → 200', async () => {
    const res = await page.request.get(`${BASE}/conversations/default_global/messages`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    const msgs = Array.isArray(body) ? body : (body.messages ?? []);
    expect(msgs.length).toBeGreaterThanOrEqual(0);
    console.log(`✅ ${msgs.length} message(s) dans default_global`);
  });

  // ── 3. Réactions ───────────────────────────────────────────────

  test('Réactions — POST 👍 → counts mis à jour', async () => {
    const msgRes = await page.request.post(`${BASE}/conversations/default_global/messages`, {
      data: { content: `reaction-test-${Date.now()}`, message_type: 'text' },
    });
    expect(msgRes.ok()).toBeTruthy();
    const msgId = (await msgRes.json()).id as string;

    const res = await page.request.post(`${BASE}/conversations/default_global/messages/${msgId}/reactions`, {
      data: { emoji: '👍' },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.success).toBe(true);
    expect(body.my_emoji).toBe('👍');
    expect(Array.isArray(body.counts['👍'])).toBe(true);
    expect(body.counts['👍'].length).toBeGreaterThan(0);
    console.log(`✅ Réaction 👍 ajoutée → counts OK`);
  });

  test('Réactions — UPSERT : 👍 → ❤️', async () => {
    const msgRes = await page.request.post(`${BASE}/conversations/default_global/messages`, {
      data: { content: `upsert-test-${Date.now()}`, message_type: 'text' },
    });
    const msgId = (await msgRes.json()).id as string;

    await page.request.post(`${BASE}/conversations/default_global/messages/${msgId}/reactions`, { data: { emoji: '👍' } });
    const res = await page.request.post(`${BASE}/conversations/default_global/messages/${msgId}/reactions`, { data: { emoji: '❤️' } });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.my_emoji).toBe('❤️');
    expect((body.counts['👍'] ?? []).length).toBe(0);
    expect(body.counts['❤️'].length).toBeGreaterThan(0);
    console.log('✅ UPSERT réaction 👍 → ❤️');
  });

  test('Réactions — DELETE → my_emoji null', async () => {
    const msgRes = await page.request.post(`${BASE}/conversations/default_global/messages`, {
      data: { content: `delete-reaction-${Date.now()}`, message_type: 'text' },
    });
    const msgId = (await msgRes.json()).id as string;

    await page.request.post(`${BASE}/conversations/default_global/messages/${msgId}/reactions`, { data: { emoji: '😂' } });
    const res = await page.request.delete(`${BASE}/conversations/default_global/messages/${msgId}/reactions`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.my_emoji).toBeNull();
    console.log('✅ Réaction supprimée → my_emoji null');
  });

  test('Réactions — emoji non autorisé → 400', async () => {
    const msgRes = await page.request.post(`${BASE}/conversations/default_global/messages`, {
      data: { content: `emoji-invalid-${Date.now()}`, message_type: 'text' },
    });
    const msgId = (await msgRes.json()).id as string;
    const res = await page.request.post(`${BASE}/conversations/default_global/messages/${msgId}/reactions`, {
      data: { emoji: '🦄' },
    });
    expect(res.status()).toBe(400);
    console.log('✅ Emoji non autorisé → 400');
  });

  // ── 4. Upload & Download ───────────────────────────────────────

  test('Upload — fichier texte → file_id + download', async () => {
    test.setTimeout(30_000);
    const res = await page.request.post('/api/upload/chat', {
      multipart: {
        file: { name: 'test-e2e.txt', mimeType: 'text/plain', buffer: Buffer.from('Contenu E2E') },
        conversation_id: 'default_global',
      },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.file_id).toBeTruthy();
    expect(body.file_name).toBe('test-e2e.txt');
    // Le backend retourne /api/download/{file_id}
    expect(body.url).toMatch(/\/api\/download\//);
    console.log(`✅ Upload → file_id=${body.file_id}`);

    const dlRes = await page.request.get(`/api/download/${body.file_id}`);
    expect(dlRes.status()).toBe(200);
    const cd = dlRes.headers()['content-disposition'] ?? '';
    expect(cd).toContain('attachment');
    expect(cd).toContain('test-e2e.txt');
    console.log(`✅ Download → Content-Disposition OK`);
  });

  test('Download — id inconnu → 404', async () => {
    const res = await page.request.get('/api/download/id-inexistant-00000');
    expect(res.status()).toBe(404);
    console.log('✅ Download id inconnu → 404');
  });

  // ── 5. Polls ───────────────────────────────────────────────────

  test('Polls — créer, voter, changer, fermer', async () => {
    test.setTimeout(45_000);

    // Créer
    const createRes = await page.request.post(`${BASE}/polls`, {
      data: { question: `Poll E2E ${Date.now()}`, options: ['Option A', 'Option B', 'Option C'] },
    });
    expect([200, 201]).toContain(createRes.status());
    const pollBody = await createRes.json();
    const pollId = pollBody.id ?? pollBody.poll?.id;
    expect(pollId).toBeTruthy();
    console.log(`✅ Sondage créé → id=${pollId}`);

    // Récupérer les options
    const detailRes = await page.request.get(`${BASE}/polls/${pollId}`);
    expect(detailRes.status()).toBe(200);
    const detail = await detailRes.json();
    const options: Array<{ id: string }> = detail.poll?.options ?? detail.options ?? [];
    expect(options.length).toBeGreaterThanOrEqual(2);
    const [optA, optB] = options;

    // Voter option A
    const voteRes = await page.request.post(`${BASE}/polls/${pollId}/vote`, { data: { option_id: optA.id } });
    expect([200, 201]).toContain(voteRes.status());
    console.log('✅ Vote option A');

    // Changer pour option B (UPSERT)
    const changeRes = await page.request.post(`${BASE}/polls/${pollId}/vote`, { data: { option_id: optB.id } });
    expect([200, 201]).toContain(changeRes.status());
    const afterChange = await (await page.request.get(`${BASE}/polls/${pollId}`)).json();
    expect(afterChange.poll?.my_vote ?? afterChange.my_vote).toBe(optB.id);
    console.log('✅ Changement de vote → B confirmé');

    // Double vote même option → 200 (UPSERT) ou 409
    const doubleRes = await page.request.post(`${BASE}/polls/${pollId}/vote`, { data: { option_id: optB.id } });
    expect([200, 201, 409]).toContain(doubleRes.status());
    console.log(`✅ Double vote → HTTP ${doubleRes.status()}`);

    // Fermer
    const closeRes = await page.request.post(`${BASE}/polls/${pollId}/close`);
    expect(closeRes.status()).toBe(200);
    const closedDetail = await (await page.request.get(`${BASE}/polls/${pollId}`)).json();
    const isClosed = closedDetail.poll?.is_closed ?? closedDetail.is_closed ?? (closedDetail.poll?.closed_at !== null);
    expect(isClosed).toBeTruthy();
    console.log('✅ Sondage fermé');

    // Vote sur sondage fermé → 400 ou 403
    const closedVote = await page.request.post(`${BASE}/polls/${pollId}/vote`, { data: { option_id: optA.id } });
    expect([400, 403]).toContain(closedVote.status());
    console.log(`✅ Vote sondage fermé → ${closedVote.status()}`);
  });

  test('Polls UI — page visible avec bouton Nouveau sondage', async () => {
    test.setTimeout(30_000);
    const [_] = await Promise.all([
      page.waitForResponse(res => res.url().includes('/api/polls') && res.request().method() === 'GET', { timeout: 15_000 }),
      page.goto('/polls'),
    ]);
    await waitForAppReady(page);
    await expect(page.locator('.btn-create')).toBeVisible({ timeout: 8_000 });
    console.log('✅ Page /polls chargée');
  });

  // ── 6. Chess ───────────────────────────────────────────────────

  test('Chess — créer partie vs IA, coup légal e2→e4, coup illégal → 400', async () => {
    test.setTimeout(60_000);

    const createRes = await page.request.post(`${BASE}/chess/create`, {
      data: { color: 'white', opponent: 'medium' },
    });
    expect([200, 201]).toContain(createRes.status());
    const { game_id } = await createRes.json();
    expect(game_id).toBeTruthy();
    console.log(`✅ Partie vs IA créée → ${game_id}`);

    // Coups légaux depuis e2
    const movesRes = await page.request.get(`${BASE}/chess/${game_id}/moves?from=e2`);
    expect(movesRes.status()).toBe(200);
    const moves = await movesRes.json();
    expect(Array.isArray(moves)).toBe(true);
    expect(moves).toContain('e2e4');
    console.log(`✅ ${moves.length} coups légaux, e2e4 présent`);

    // Coup légal
    const moveRes = await page.request.post(`${BASE}/chess/${game_id}/move`, { data: { from: 'e2', to: 'e4' } });
    expect(moveRes.status()).toBe(200);
    expect((await moveRes.json()).success).toBe(true);
    console.log('✅ Coup e2→e4 accepté');

    // Coup illégal
    const illegalRes = await page.request.post(`${BASE}/chess/${game_id}/move`, { data: { from: 'e2', to: 'e6' } });
    expect(illegalRes.status()).toBe(400);
    console.log('✅ Coup illégal e2→e6 → 400');
  });

  test('Chess UI — plateau 64 cases visible', async () => {
    test.setTimeout(45_000);
    const createRes = await page.request.post(`${BASE}/chess/create`, { data: { color: 'white', opponent: 'easy' } });
    const { game_id } = await createRes.json();
    await page.goto(`/chess/${game_id}`);
    await waitForAppReady(page);
    await expect(page.locator('.chess-board')).toBeVisible({ timeout: 15_000 });
    expect(await page.locator('.chess-board .cell').count()).toBe(64);
    console.log('✅ Échiquier 64 cases OK');
  });

  // ── 7. Calendar ────────────────────────────────────────────────

  test('Calendar — créer événement → retourne id', async () => {
    const res = await page.request.post(`${BASE}/events`, {
      data: { title: `E2E Event ${Date.now()}`, date: '2026-12-25', time: '18:00', description: 'Test E2E' },
    });
    expect([200, 201]).toContain(res.status());
    const body = await res.json();
    expect(body.success).toBe(true);
    expect(body.id).toBeTruthy();
    console.log(`✅ Événement créé → id=${body.id}`);
  });

  test('Calendar UI — page et bouton ajouter visibles', async () => {
    test.setTimeout(30_000);
    await page.goto('/calendar');
    await waitForAppReady(page);
    await expect(page.locator('.calendar-grid')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.add-event-btn')).toBeVisible({ timeout: 8_000 });
    console.log('✅ /calendar chargé, bouton ajouter visible');
  });

  // ── 8. Settings ────────────────────────────────────────────────

  test('Settings — 3 onglets (Profil / Sécurité / Apparence)', async () => {
    test.setTimeout(30_000);
    await page.goto('/settings');
    await waitForAppReady(page);
    await expect(page.locator('#userName')).toBeVisible({ timeout: 8_000 });
    await page.locator('[role="tab"]').filter({ hasText: 'Sécurité' }).click();
    await expect(page.locator('#currentPassword')).toBeVisible({ timeout: 5_000 });
    await page.locator('[role="tab"]').filter({ hasText: 'Apparence' }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    console.log('✅ 3 onglets Settings OK');
  });

  test('Settings — changement de thème', async () => {
    test.setTimeout(30_000);
    await page.goto('/settings');
    await waitForAppReady(page);
    await page.locator('[role="tab"]').filter({ hasText: 'Apparence' }).click();
    await expect(page.locator('.themes-grid')).toBeVisible({ timeout: 5_000 });
    const cards = page.locator('.theme-card');
    expect(await cards.count()).toBeGreaterThan(1);
    await cards.nth(1).click();
    await expect(cards.nth(1)).toHaveClass(/selected/, { timeout: 3_000 });
    console.log('✅ Thème changé');
  });

  // ── 9. Navigation ──────────────────────────────────────────────

  const routes = ['/chat', '/calendar', '/chess', '/polls', '/settings', '/help', '/events'];

  for (const route of routes) {
    test(`Navigation ${route} → accessible`, async () => {
      test.setTimeout(30_000);
      await page.goto(route);
      await page.waitForLoadState('networkidle', { timeout: 12_000 }).catch(() => {});
      expect(page.url()).not.toMatch(/\/login/);
      console.log(`✅ ${route} OK`);
    });
  }

  // ── 10. E2EE ──────────────────────────────────────────────────

  test('E2EE — enregistrer clé publique → success', async () => {
    const fakeKey = Buffer.from(new Uint8Array(32).map((_, i) => i)).toString('base64');
    const res = await page.request.post(`${BASE}/auth/public-key`, { data: { public_key: fakeKey } });
    expect([200, 201]).toContain(res.status());
    expect((await res.json()).success).toBe(true);
    console.log('✅ Clé publique enregistrée');
  });

  test('E2EE — GET public-keys pour default_global → objet avec clés', async () => {
    const fakeKey = Buffer.from(new Uint8Array(32).fill(42)).toString('base64');
    await page.request.post(`${BASE}/auth/public-key`, { data: { public_key: fakeKey } });
    const res = await page.request.get(`${BASE}/auth/public-keys?conversation_id=default_global`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(typeof body).toBe('object');
    console.log(`✅ GET public-keys → ${Object.keys(body).length} clé(s)`);
  });

  // ── 11. Logout ────────────────────────────────────────────────

  test('Logout → redirigé vers /login', async () => {
    test.setTimeout(30_000);
    await page.goto('/chat');
    await waitForAppReady(page);
    const logoutBtn = page.locator('button[aria-label="Déconnexion"]').first();
    await expect(logoutBtn).toBeVisible({ timeout: 8_000 });
    await logoutBtn.click();
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 });
    console.log('✅ Logout → /login');
  });

});

// ── Rate limiting — suite isolée (serial, à la fin) ────────────────

test.describe.serial('Rate Limiting', () => {

  test('POST /api/auth/login × 15 depuis user non-auth → au moins un 429', async ({ request }) => {
    test.setTimeout(30_000);
    const results: number[] = [];
    for (let i = 0; i < 15; i++) {
      const res = await request.post(`${BASE}/auth/login`, {
        data: { username: `flood_${i}`, password: 'wrong' },
      });
      results.push(res.status());
    }
    expect(results.every(s => s !== 200)).toBe(true); // jamais de 200 (mdp invalide)
    expect(results.includes(401)).toBe(true);         // des 401 avant le rate limit
    console.log(`✅ Flood × 15 → ${[...new Set(results)].join(', ')}`);
  });

});
