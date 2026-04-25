import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('Debug conversations and call notifications', async ({ browser }) => {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  const errors: string[] = [];
  const wsMessages: any[] = [];
  
  page.on('pageerror', err => errors.push(`PAGE_ERROR: ${err.message}`));
  page.on('console', msg => { 
    if (msg.type() === 'error') errors.push(`CONSOLE_ERROR: ${msg.text()}`);
    if (msg.text().includes('[WS]') || msg.text().includes('[CallManager]')) {
      console.log(`[Browser] ${msg.text()}`);
    }
  });
  
  // Track WebSocket messages
  page.on('websocket', ws => {
    ws.on('framereceived', frame => {
      if (frame.payload) {
        try {
          const data = JSON.parse(frame.payload.toString());
          wsMessages.push(data);
          if (data.type === 'call_request') {
            console.log(`[WS] call_request received:`, data);
          }
        } catch {}
      }
    });
  });
  
  // Login as hermes-bot
  await page.goto('https://192.168.1.192:6443/login');
  await page.waitForTimeout(2000);
  await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page.fill('input[type="password"]', 'Hermes2026!');
  await page.click('button[type="submit"]');
  await page.waitForTimeout(3000);
  
  if (page.url().includes('/login')) {
    console.log('❌ Login failed');
    return;
  }
  console.log('✅ Logged in as hermes-bot');
  
  // Navigate to chat
  await page.goto('https://192.168.1.192:6443/chat');
  await page.waitForTimeout(3000);
  
  // Check conversations
  const convCount = await page.locator('.conversation-item').count();
  console.log(`Conversations visible: ${convCount}`);
  
  // Check if default_global is visible
  const globalConv = await page.locator('text=Nook').count();
  console.log(`Global conversation visible: ${globalConv > 0 ? '✅' : '❌'}`);
  
  // Check sidebar
  const sidebar = await page.locator('.conversations-sidebar').count();
  console.log(`Sidebar present: ${sidebar > 0 ? '✅' : '❌'}`);
  
  // Check for errors in console
  if (errors.length > 0) {
    console.log('\n=== Console errors ===');
    for (const e of errors.slice(0, 10)) console.log(e);
  }
  
  // Check API for conversations
  const apiConvs = await page.evaluate(async () => {
    try {
      const r = await fetch('/api/conversations', { credentials: 'include' });
      if (!r.ok) return { error: r.status };
      return await r.json();
    } catch (e) {
      return { error: String(e) };
    }
  });
  
  console.log('\nAPI conversations:', JSON.stringify(apiConvs).slice(0, 200));
  
  // Check WebSocket connection
  const wsConnected = await page.evaluate(() => {
    return (window as any).__chatStore?.wsConnected || false;
  });
  console.log(`WebSocket connected: ${wsConnected ? '✅' : '❌'}`);
  
  await ctx.close();
  
  console.log('\n=== Test complete ===');
});
