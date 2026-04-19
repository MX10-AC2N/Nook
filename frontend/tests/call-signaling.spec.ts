import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('Call signaling test — verify signal routing', async ({ browser }) => {
  // Create two browser contexts
  const ctx1 = await browser.newContext();
  const ctx2 = await browser.newContext();
  
  const page1 = await ctx1.newPage(); // Appelant
  const page2 = await ctx2.newPage(); // Appelé
  
  // Track console logs
  const logs1: string[] = [];
  const logs2: string[] = [];
  
  page1.on('console', msg => logs1.push(msg.text()));
  page2.on('console', msg => logs2.push(msg.text()));
  
  // Login as hermes-bot (appelant)
  await page1.goto('https://192.168.1.192:6443/login');
  await page1.waitForTimeout(2000);
  await page1.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page1.fill('input[type="password"]', 'Hermes2026!');
  await page1.click('button[type="submit"]');
  await page1.waitForTimeout(3000);
  console.log('✅ hermes-bot logged in');
  
  // Login as admin (appelé)
  await page2.goto('https://192.168.1.192:6443/login');
  await page2.waitForTimeout(2000);
  await page2.fill('input[type="text"], input[name="username"]', 'admin');
  await page2.fill('input[type="password"]', 'admin123');
  await page2.click('button[type="submit"]');
  await page2.waitForTimeout(3000);
  console.log('✅ admin logged in');
  
  // Both navigate to chat to establish WebSocket connections
  await page1.goto('https://192.168.1.192:6443/chat');
  await page1.waitForTimeout(2000);
  await page2.goto('https://192.168.1.192:6443/chat');
  await page2.waitForTimeout(2000);
  
  // Get conversation ID
  const convId = await page1.evaluate(async () => {
    const r = await fetch('/api/conversations', { credentials: 'include' });
    const convs = await r.json();
    const dm = convs.find((c: any) => !c.is_group && c.id !== 'default_global');
    return dm?.id;
  });
  
  console.log('Conversation ID:', convId);
  
  if (!convId) {
    console.log('❌ No DM conversation found');
    return;
  }
  
  // Check WebSocket connections
  const ws1Connected = await page1.evaluate(() => {
    return (window as any).__chatStore?.wsConnected || false;
  });
  const ws2Connected = await page2.evaluate(() => {
    return (window as any).__chatStore?.wsConnected || false;
  });
  
  console.log('Page1 WS connected:', ws1Connected);
  console.log('Page2 WS connected:', ws2Connected);
  
  // Appelant navigue vers la page d'appel
  await page1.goto(`https://192.168.1.192:6443/call/${convId}?type=audio`);
  await page1.waitForTimeout(3000);
  
  // Vérifier si la page d'appel se charge
  const callPageLoaded = await page1.locator('.call-page').count();
  console.log('Call page loaded:', callPageLoaded > 0 ? '✅' : '❌');
  
  // Vérifier les logs pour les signaux d'appel
  const callRequestLogs = logs1.filter(log => log.includes('call_request'));
  console.log('Call request logs:', callRequestLogs.length);
  
  // Vérifier si l'appelé reçoit le signal
  await page2.waitForTimeout(2000);
  const incomingCallBanner = await page2.locator('.call-banner').count();
  console.log('Incoming call banner on page2:', incomingCallBanner > 0 ? '✅' : '❌');
  
  // Vérifier les logs de l'appelé
  const wsLogs = logs2.filter(log => log.includes('[WS] Call signal received'));
  console.log('WS logs on page2:', wsLogs.length);
  
  // Vérifier les logs du navigateur pour les erreurs
  const errors1 = logs1.filter(log => log.includes('error') || log.includes('Error'));
  const errors2 = logs2.filter(log => log.includes('error') || log.includes('Error'));
  
  if (errors1.length > 0) {
    console.log('\n=== Errors on page1 ===');
    for (const e of errors1.slice(0, 5)) console.log(`  ${e}`);
  }
  
  if (errors2.length > 0) {
    console.log('\n=== Errors on page2 ===');
    for (const e of errors2.slice(0, 5)) console.log(`  ${e}`);
  }
  
  await ctx1.close();
  await ctx2.close();
  
  console.log('\n=== Test complete ===');
});
