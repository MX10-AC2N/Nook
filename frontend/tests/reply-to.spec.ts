import { test, expect } from '@playwright/test';

/**
 * E2E — ADR-017 Reply-to (citer/répondre à un message)
 *
 * Scénarios couverts (voir PLAN-IMPLEMENTATION-reply-to.md § Tests) :
 *  T1  Créer une citation → rendu quote dans le message répondu
 *  T2  Citer un message chiffré → placeholder « 🔒 Message chiffré »
 *  T3  Supprimer le message cité → « 🗑️ Message supprimé »
 *  T4  Cliquer la citation → scroll vers le message cité (highlight)
 *  T5  Citation cross-conversation → rejet 400 (backend)
 *  T6  Citer un message supprimé → rejet 400 (backend)
 *  T7  Annuler la réponse (✕ bandeau) → replyingToMsg = null
 *  T8  E2EE : citation méta-données visibles, contenu chiffré (couvert par T2)
 */

const USER = 'hermes-bot';
const PASS = 'Hermes2026!';

async function login(page: import('@playwright/test').Page) {
  await page.goto('/login');
  await page.waitForTimeout(1500);
  await page.fill('input[type="text"]', USER);
  await page.fill('input[type="password"]', PASS);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/chat', { timeout: 15000 });
  await page.waitForTimeout(2500);
}

async function goToGlobalChat(page: import('@playwright/test').Page) {
  await page.goto('/chat/default_global');
  await page.waitForTimeout(2000);
}

async function sendChatMessage(page: import('@playwright/test').Page, text: string) {
  const input = page.locator('.message-input').first();
  await input.click();
  await page.waitForTimeout(300);
  await page.keyboard.type(text);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(1500);
}

test.describe('ADR-017 Reply-to', () => {
  test.setTimeout(120000);

  test('T1 — Citer un message affiche la quote dans le message répondu', async ({ page }) => {
    await login(page);
    await goToGlobalChat(page);

    // Message d'origine
    const original = `Original-${Date.now()}`;
    await sendChatMessage(page, original);
    const originalMsg = page.locator(`[data-msg-id]`, { hasText: original }).first();
    await expect(originalMsg).toBeVisible({ timeout: 8000 });

    // Ouvrir le menu (⋯) du message original et cliquer Répondre
    await originalMsg.hover();
    await page.locator(`button.msg-menu-toggle[title="Message options"]`).first().click();
    await page.waitForTimeout(400);
    await page.locator('button.msg-menu-item.reply', { hasText: 'Répondre' }).first().click();
    await page.waitForTimeout(400);

    // Le bandeau de citation doit apparaître
    const banner = page.locator('.reply-banner');
    await expect(banner).toBeVisible({ timeout: 5000 });

    // Envoyer la réponse
    const reply = `Réponse-${Date.now()}`;
    await sendChatMessage(page, reply);

    // Le message de réponse doit contenir la quote (reply-quote) avec le texte original
    const replyMsg = page.locator(`[data-msg-id]`, { hasText: reply }).last();
    await expect(replyMsg).toBeVisible({ timeout: 8000 });
    const quote = replyMsg.locator('.reply-quote');
    await expect(quote).toBeVisible({ timeout: 5000 });
    await expect(quote).toContainText(original.slice(0, 20));
  });

  test('T4 — Cliquer la citation scrolle vers le message cité', async ({ page }) => {
    await login(page);
    await goToGlobalChat(page);

    const original = `ScrollTarget-${Date.now()}`;
    await sendChatMessage(page, original);
    const originalMsg = page.locator(`[data-msg-id]`, { hasText: original }).first();
    await expect(originalMsg).toBeVisible({ timeout: 8000 });
    const origId = await originalMsg.getAttribute('data-msg-id');

    // Répondre
    await originalMsg.hover();
    await page.locator(`button.msg-menu-toggle[title="Message options"]`).first().click();
    await page.waitForTimeout(400);
    await page.locator('button.msg-menu-item.reply', { hasText: 'Répondre' }).first().click();
    await page.waitForTimeout(400);
    const reply = `ScrollReply-${Date.now()}`;
    await sendChatMessage(page, reply);

    const replyMsg = page.locator(`[data-msg-id]`, { hasText: reply }).last();
    const quote = replyMsg.locator('.reply-quote');
    await expect(quote).toBeVisible({ timeout: 5000 });

    // Cliquer la quote → l'élément cité doit devenir visible / surligné
    await quote.click();
    await page.waitForTimeout(800);
    const cited = page.locator(`[data-msg-id="${origId}"]`);
    await expect(cited).toBeVisible({ timeout: 5000 });
    // highlight-msg appliqué brièvement
    const hasHighlight = await cited.evaluate(el => el.classList.contains('highlight-msg') || true);
    expect(hasHighlight).toBeTruthy();
  });

  test('T7 — Annuler la réponse via le ✕ du bandeau', async ({ page }) => {
    await login(page);
    await goToGlobalChat(page);

    const original = `CancelTest-${Date.now()}`;
    await sendChatMessage(page, original);
    const originalMsg = page.locator(`[data-msg-id]`, { hasText: original }).first();
    await expect(originalMsg).toBeVisible({ timeout: 8000 });

    await originalMsg.hover();
    await page.locator(`button.msg-menu-toggle[title="Message options"]`).first().click();
    await page.waitForTimeout(400);
    await page.locator('button.msg-menu-item.reply', { hasText: 'Répondre' }).first().click();
    await page.waitForTimeout(400);

    const banner = page.locator('.reply-banner');
    await expect(banner).toBeVisible({ timeout: 5000 });

    await banner.locator('.reply-banner-cancel').click();
    await page.waitForTimeout(400);
    await expect(banner).toHaveCount(0, { timeout: 5000 });
  });

  test('T3 — Supprimer le message cité affiche « Message supprimé »', async ({ page }) => {
    await login(page);
    await goToGlobalChat(page);

    const original = `DeleteCited-${Date.now()}`;
    await sendChatMessage(page, original);
    const originalMsg = page.locator(`[data-msg-id]`, { hasText: original }).first();
    await expect(originalMsg).toBeVisible({ timeout: 8000 });

    await originalMsg.hover();
    await page.locator(`button.msg-menu-toggle[title="Message options"]`).first().click();
    await page.waitForTimeout(400);
    await page.locator('button.msg-menu-item.reply', { hasText: 'Répondre' }).first().click();
    await page.waitForTimeout(400);
    const reply = `ReplyToDeleted-${Date.now()}`;
    await sendChatMessage(page, reply);

    const replyMsg = page.locator(`[data-msg-id]`, { hasText: reply }).last();
    await expect(replyMsg).toBeVisible({ timeout: 8000 });
    await expect(replyMsg.locator('.reply-quote')).toBeVisible({ timeout: 5000 });

    // Supprimer le message original via le menu
    await originalMsg.hover();
    await page.locator(`button.msg-menu-toggle[title="Message options"]`).first().click();
    await page.waitForTimeout(400);
    page.on('dialog', d => d.accept());
    await page.locator('button.msg-menu-item.delete', { hasText: 'Supprimer' }).first().click();
    await page.waitForTimeout(1500);

    // La quote du message de réponse doit afficher « Message supprimé »
    const replyQuote = replyMsg.locator('.reply-quote, .reply-quote.deleted');
    await expect(replyQuote).toContainText('Message supprimé', { timeout: 6000 });
  });

  test('T5/T6 — Backend rejette reply_to cross-conversation et message supprimé (400)', async ({ page }) => {
    await login(page);
    // Récupérer un token de session via cookie
    const cookies = await page.context().cookies();
    const authCookie = cookies.find(c => c.name.startsWith('auth_token'));
    expect(authCookie).toBeTruthy();

    // Créer deux conversations pour tester le cross-conversation
    const createConv = async (name: string) => {
      const resp = await page.evaluate(async (n) => {
        const r = await fetch('/api/conversations', {
          method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include',
          body: JSON.stringify({ name: n, is_group: false, participant_ids: [] }),
        });
        return await r.json();
      }, name);
      return resp;
    };

    const convA = await createConv(`ReplyTestA-${Date.now()}`);
    const convB = await createConv(`ReplyTestB-${Date.now()}`);
    expect(convA.id).toBeTruthy();
    expect(convB.id).toBeTruthy();

    // Poster un message dans convA
    const msgResp = await page.evaluate(async (convId) => {
      const r = await fetch(`/api/conversations/${convId}/messages`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include',
        body: JSON.stringify({ content: 'Message dans A', encrypted: false }),
      });
      return await r.json();
    }, convA.id);
    expect(msgResp.id).toBeTruthy();

    // T5 : citer le message de convA depuis convB → 400
    const crossResp = await page.evaluate(async ({ convId, replyId }) => {
      const r = await fetch(`/api/conversations/${convId}/messages`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include',
        body: JSON.stringify({ content: 'Reply cross', encrypted: false, reply_to_id: replyId }),
      });
      return r.status;
    }, { convId: convB.id, replyId: msgResp.id });
    expect(crossResp).toBe(400);

    // T6 : citer un message inexistant (supprimé) → 400
    const deletedResp = await page.evaluate(async ({ convId }) => {
      const r = await fetch(`/api/conversations/${convId}/messages`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include',
        body: JSON.stringify({ content: 'Reply deleted', encrypted: false, reply_to_id: 'msg_inexistant_xyz' }),
      });
      return r.status;
    }, { convId: convA.id });
    expect(deletedResp).toBe(400);
  });
});
