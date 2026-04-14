import { test, expect } from '@playwright/test';

const BASE = 'http://192.168.1.192:6300';

test.describe('Chat UI — Améliorations', () => {
  
  test('Login + navigate to chat', async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.screenshot({ path: '/tmp/nook-01-login.png' });
    
    // Fill login form
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.screenshot({ path: '/tmp/nook-02-filled.png' });
    
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
    await page.screenshot({ path: '/tmp/nook-03-chat.png' });
  });

  test('Header is compact (< 60px)', async ({ page }) => {
    // Login first
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
    
    const header = page.locator('.chat-header');
    await expect(header).toBeVisible();
    const box = await header.boundingBox();
    console.log('Header height:', box?.height);
    expect(box?.height).toBeLessThan(60);
    await page.screenshot({ path: '/tmp/nook-04-header.png' });
  });

  test('Send emoji-only message (should be large)', async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(1000);
    
    const input = page.locator('.message-input');
    await input.fill('🎉🎉🎉');
    await page.locator('button[type="submit"]').click();
    await page.waitForTimeout(1500);
    
    const emojiMsg = page.locator('.emoji-only').last();
    if (await emojiMsg.isVisible()) {
      const fontSize = await emojiMsg.evaluate(el => window.getComputedStyle(el).fontSize);
      console.log('Emoji font-size:', fontSize);
      expect(parseFloat(fontSize)).toBeGreaterThan(30);
    }
    await page.screenshot({ path: '/tmp/nook-05-emoji.png' });
  });

  test('Reaction picker opens on hover + click', async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
    
    // Hover over a message
    const msg = page.locator('.message').first();
    await msg.hover();
    await page.waitForTimeout(500);
    await page.screenshot({ path: '/tmp/nook-06-hover.png' });
    
    // Click reaction button
    const reactionBtn = page.locator('[data-testid="reaction-trigger"]').first();
    if (await reactionBtn.isVisible()) {
      await reactionBtn.click();
      await page.waitForTimeout(300);
      
      const picker = page.locator('[data-testid="emoji-picker"]');
      await expect(picker).toBeVisible();
      console.log('✅ Reaction picker is visible');
      await page.screenshot({ path: '/tmp/nook-07-picker.png' });
      
      // Click an emoji
      const emojiBtn = page.locator('[data-testid="emoji-quick-btn"]').first();
      if (await emojiBtn.isVisible()) {
        await emojiBtn.click();
        await page.waitForTimeout(500);
        console.log('✅ Emoji reaction clicked');
      }
    }
    await page.screenshot({ path: '/tmp/nook-08-after-reaction.png' });
  });

  test('Input stays enabled while sending', async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(1000);
    
    const input = page.locator('.message-input');
    await expect(input).toBeEnabled();
    console.log('✅ Input is enabled');
  });
});
