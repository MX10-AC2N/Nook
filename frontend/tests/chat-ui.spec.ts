import { test, expect } from '@playwright/test';

const BASE = 'http://192.168.1.192:6300';

test.describe('Chat UI Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"]', 'hermes-bot');
    await page.fill('input[name="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 10000 });
  });

  test('Header is compact', async ({ page }) => {
    const header = page.locator('.chat-header');
    await expect(header).toBeVisible();
    const box = await header.boundingBox();
    expect(box.height).toBeLessThan(60); // Should be ~42px
    console.log('Header height:', box.height);
  });

  test('Emoji-only message is large', async ({ page }) => {
    // Send an emoji-only message
    const input = page.locator('.message-input');
    await input.fill('😀😀😀');
    await page.locator('button[type="submit"]').click();
    await page.waitForTimeout(1000);
    
    const emojiMsg = page.locator('.emoji-only').last();
    if (await emojiMsg.isVisible()) {
      const style = await emojiMsg.evaluate(el => window.getComputedStyle(el).fontSize);
      console.log('Emoji font-size:', style);
      expect(parseFloat(style)).toBeGreaterThan(30); // 3.5rem = ~56px
    }
  });

  test('Reaction picker opens on click', async ({ page }) => {
    // Hover over a message to show actions
    const msg = page.locator('.message').first();
    await msg.hover();
    await page.waitForTimeout(300);
    
    // Click reaction button
    const reactionBtn = page.locator('[data-testid="reaction-trigger"]').first();
    if (await reactionBtn.isVisible()) {
      await reactionBtn.click();
      await page.waitForTimeout(200);
      
      const picker = page.locator('[data-testid="emoji-picker"]');
      await expect(picker).toBeVisible();
      
      // Click an emoji
      const emojiBtn = page.locator('[data-testid="emoji-quick-btn"]').first();
      await emojiBtn.click();
    }
  });

  test('Screenshot chat page', async ({ page }) => {
    await page.waitForTimeout(2000);
    await page.screenshot({ path: '/tmp/nook-chat.png', fullPage: false });
    console.log('Screenshot saved to /tmp/nook-chat.png');
  });
});
