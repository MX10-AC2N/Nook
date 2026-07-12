import { test } from '@playwright/test';

const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';

test.use({ ignoreHTTPSErrors: true });

test('Call signaling test — check WebSocket and call notifications', async ({ browser }) => {
  // Create two browser contexts for two users
  const ctx1 = await browser.newContext();
  const ctx2 = await browser.newContext();
  
  const page1 = await ctx1.newPage(); // julien
  const page2 = await ctx2.newPage(); // geraldine
  
  const errors1: string[] = [];
  const errors2: string[] = [];
  
  page1.on('pageerror', err => errors1.push(`PAGE_ERROR: ${err.message}`));
  page1.on('console', msg => { if (msg.type() === 'error') errors1.push(`CONSOLE_ERROR: ${msg.text()}`); });
  
  page2.on('pageerror', err => errors2.push(`PAGE_ERROR: ${err.message}`));
  page2.on('console', msg => { if (msg.type() === 'error') errors2.push(`CONSOLE_ERROR: ${msg.text()}`); });
  
  // Login as julien
  await page1.goto(`${BASE_URL}/login`);
  await page1.waitForTimeout(2000);
  await page1.fill('input[type="text"], input[name="username"]', 'julien');
  await page1.fill('input[type="password"]', 'julien123');
  await page1.click('button[type="submit"]');
  await page1.waitForTimeout(3000);
  
  if (page1.url().includes('/login')) {
    console.log('❌ julien login failed');
    // Try with different password
    await page1.fill('input[type="password"]', 'password');
    await page1.click('button[type="submit"]');
    await page1.waitForTimeout(3000);
    if (page1.url().includes('/login')) {
      console.log('❌ julien login failed again');
      return;
    }
  }
  console.log('✅ julien logged in');
  
  // Login as geraldine
  await page2.goto(`${BASE_URL}/login`);
  await page2.waitForTimeout(2000);
  await page2.fill('input[type="text"], input[name="username"]', 'geraldine');
  await page2.fill('input[type="password"]', 'geraldine123');
  await page2.click('button[type="submit"]');
  await page2.waitForTimeout(3000);
  
  if (page2.url().includes('/login')) {
    console.log('❌ geraldine login failed');
    // Try with different password
    await page2.fill('input[type="password"]', 'password');
    await page2.click('button[type="submit"]');
    await page2.waitForTimeout(3000);
    if (page2.url().includes('/login')) {
      console.log('❌ geraldine login failed again');
      return;
    }
  }
  console.log('✅ geraldine logged in');
  
  // Get conversation ID for DM between julien and geraldine
  const convId = await page1.evaluate(async () => {
    const r = await fetch('/api/conversations', { credentials: 'include' });
    const convs = await r.json();
    // Find a DM conversation (not default_global)
    const dm = convs.find((c: any) => !c.is_group && c.id !== 'default_global');
    return dm?.id;
  });
  
  console.log('Conversation ID:', convId);
  
  if (!convId) {
    console.log('❌ No DM conversation found');
    return;
  }
  
  // Julien navigates to call page
  await page1.goto(`${BASE_URL}/call/${convId}?type=audio`);
  await page1.waitForTimeout(3000);
  
  // Check if call page loads
  const idle1 = await page1.locator('.idle').count();
  console.log('Julien call page idle:', idle1 > 0 ? '✅' : '❌');
  
  // Check WebSocket connection
  const wsStatus = await page1.evaluate(() => {
    return (window as any).__wsConnected || false;
  });
  console.log('WebSocket connected:', wsStatus ? '✅' : '❌');
  
  // Check for call button
  const callBtn = await page1.locator('button:has-text("Démarrer"), button:has-text("Appel")').count();
  console.log('Call button found:', callBtn > 0 ? '✅' : '❌');
  
  // Check Geraldine's page for incoming call notification
  await page2.waitForTimeout(2000);
  const callBanner = await page2.locator('.call-banner, [class*="call"]').count();
  console.log('Geraldine call banner:', callBanner > 0 ? '✅' : '❌');
  
  // Check for any notifications
  const notifications = await page2.locator('.notification, [class*="notification"]').count();
  console.log('Geraldine notifications:', notifications);
  
  // Check console errors
  if (errors1.length > 0) {
    console.log('\n=== Julien errors ===');
    for (const e of errors1.slice(0, 5)) console.log(e);
  }
  
  if (errors2.length > 0) {
    console.log('\n=== Geraldine errors ===');
    for (const e of errors2.slice(0, 5)) console.log(e);
  }
  
  await ctx1.close();
  await ctx2.close();
  
  console.log('\n=== Test complete ===');
});
