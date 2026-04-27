import { test, expect } from '@playwright/test';

test.describe('Chat Functionality', () => {
  test.setTimeout(120000);

  test('Send and view message in chat', async ({ browser }) => {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    
    // Login as hermes-bot
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Check login success
    const url = page.url();
    if (url.includes('/login')) {
      console.log('❌ Login failed');
      await ctx.close();
      return;
    }
    console.log('✅ Logged in as hermes-bot');
    
    // Navigate to a conversation
    const convId = await page.evaluate(async () => {
      const r = await fetch('/api/conversations', { credentials: 'include' });
      const convs = await r.json();
      return convs.find((c: any) => !c.is_group && c.id !== 'default_global')?.id;
    });
    
    if (!convId) {
      console.log('⚠️ No 1-to-1 conversation found');
      await ctx.close();
      return;
    }
    
    await page.goto(`https://192.168.1.192:6443/chat/${convId}`);
    await page.waitForTimeout(2000);
    
    // Send a test message
    const testMsg = `Chat test ${Date.now()}`;
    const input = await page.locator('.message-input, textarea, [contenteditable]').first();
    await input.click();
    await page.keyboard.type(testMsg);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2000);
    
    // Verify message is visible
    const msgVisible = await page.locator(`text=${testMsg}`).count();
    console.log(`Message sent and visible: ${msgVisible > 0 ? '✅' : '❌'}`);
    
    expect(msgVisible).toBeGreaterThan(0);
    
    await ctx.close();
  });

  test('View chat history (pagination)', async ({ browser }) => {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    
    // Login as hermes-bot
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to conversation
    const convId = await page.evaluate(async () => {
      const r = await fetch('/api/conversations', { credentials: 'include' });
      const convs = await r.json();
      return convs.find((c: any) => !c.is_group && c.id !== 'default_global')?.id;
    });
    
    if (!convId) {
      console.log('⚠️ No conversation found');
      await ctx.close();
      return;
    }
    
    await page.goto(`https://192.168.1.192:6443/chat/${convId}`);
    await page.waitForTimeout(2000);
    
    // Check if messages are loaded
    const msgCount = await page.locator('[data-test-id="message"], .message-item').count();
    console.log(`Messages loaded: ${msgCount}`);
    
    // If there are more than 20 messages, try to load more
    if (msgCount >= 20) {
      const loadMoreBtn = await page.locator('button:has-text("Load more"), button:has-text("Voir plus")').count();
      if (loadMoreBtn > 0) {
        await page.click('button:has-text("Load more"), button:has-text("Voir plus")');
        await page.waitForTimeout(2000);
        const newMsgCount = await page.locator('[data-test-id="message"], .message-item').count();
        console.log(`After load more: ${newMsgCount} messages`);
      }
    }
    
    expect(msgCount).toBeGreaterThan(0);
    
    await ctx.close();
  });

  test('Chat UI elements present', async ({ page }) => {
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('https://192.168.1.192:6443/chat/default_global');
    await page.waitForTimeout(2000);
    
    // Check for chat UI elements
    const input = await page.locator('.message-input, textarea, [contenteditable]').count();
    const sendBtn = await page.locator('button:has-text("Send"), button:has(svg[data-icon="send"]), button[type="submit"]').count();
    
    console.log(`Chat UI - Input: ${input > 0 ? '✅' : '❌'}, Send button: ${sendBtn > 0 ? '✅' : '❌'}`);
    
    expect(input).toBeGreaterThan(0);
  });
});
