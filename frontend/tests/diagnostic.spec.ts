import { test } from '@playwright/test';

const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';

test.use({ ignoreHTTPSErrors: true });

test('Diagnostic complet — login, conversations, messages, appels', async ({ browser }) => {
  const ctx1 = await browser.newContext();
  const page1 = await ctx1.newPage();
  
  const logs: string[] = [];
  const errors: string[] = [];
  
  page1.on('console', msg => {
    const text = msg.text();
    logs.push(text);
    if (msg.type() === 'error') errors.push(text);
  });
  
  // ── 1. Login ──────────────────────────────────────────────────────
  console.log('=== 1. LOGIN ===');
  await page1.goto(`${BASE_URL}/login`);
  await page1.waitForTimeout(2000);
  await page1.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page1.fill('input[type="password"]', 'Hermes2026!');
  await page1.click('button[type="submit"]');
  await page1.waitForTimeout(3000);
  
  const url = page1.url();
  console.log('Login URL:', url);
  console.log('Login:', url.includes('/login') ? '❌ FAILED' : '✅ OK');
  
  if (url.includes('/login')) {
    console.log('Cannot continue without login');
    return;
  }
  
  // ── 2. Conversations ──────────────────────────────────────────────
  console.log('\n=== 2. CONVERSATIONS ===');
  await page1.goto(`${BASE_URL}/chat`);
  await page1.waitForTimeout(3000);
  
  const convCount = await page1.locator('.conversation-item').count();
  console.log(`Conversations: ${convCount}`);
  console.log('Conversations:', convCount > 0 ? '✅ OK' : '❌ EMPTY');
  
  // ── 3. Service Worker ─────────────────────────────────────────────
  console.log('\n=== 3. SERVICE WORKER ===');
  const swInfo = await page1.evaluate(async () => {
    if (!('serviceWorker' in navigator)) return { supported: false };
    const reg = await navigator.serviceWorker.getRegistration();
    return {
      supported: true,
      registered: !!reg,
      active: !!reg?.active,
      scope: reg?.scope
    };
  });
  console.log('SW supported:', swInfo.supported ? '✅' : '❌');
  console.log('SW registered:', swInfo.registered ? '✅' : '❌');
  console.log('SW active:', swInfo.active ? '✅' : '❌');
  if (swInfo.scope) console.log('SW scope:', swInfo.scope);
  
  // ── 4. Push notification state ────────────────────────────────────
  console.log('\n=== 4. PUSH NOTIFICATIONS ===');
  const pushState = await page1.evaluate(async () => {
    if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
      return { supported: false };
    }
    try {
      const reg = await Promise.race([
        navigator.serviceWorker.ready,
        new Promise<null>((_, reject) => setTimeout(() => reject('timeout'), 5000))
      ]);
      if (!reg) return { supported: true, error: 'SW not ready' };
      const sub = await reg.pushManager.getSubscription();
      return {
        supported: true,
        subscribed: !!sub,
        endpoint: sub?.endpoint?.slice(0, 50) + '...'
      };
    } catch (e) {
      return { supported: true, error: String(e) };
    }
  });
  console.log('Push supported:', pushState.supported ? '✅' : '❌');
  console.log('Push subscribed:', pushState.subscribed ? '✅' : '❌');
  if (pushState.error) console.log('Push error:', pushState.error);
  if (pushState.endpoint) console.log('Push endpoint:', pushState.endpoint);
  
  // ── 5. VAPID key ──────────────────────────────────────────────────
  console.log('\n=== 5. VAPID KEY ===');
  const vapidResult = await page1.evaluate(async () => {
    try {
      const r = await fetch('/api/vapid-public-key');
      const key = await r.text();
      return { ok: r.ok, key: key.slice(0, 40) + '...', length: key.length };
    } catch (e) {
      return { ok: false, error: String(e) };
    }
  });
  console.log('VAPID endpoint:', vapidResult.ok ? '✅ OK' : '❌ FAILED');
  if (vapidResult.key) console.log('VAPID key:', vapidResult.key, `(${vapidResult.length} chars)`);
  if (vapidResult.error) console.log('VAPID error:', vapidResult.error);
  
  // ── 6. Messages ───────────────────────────────────────────────────
  console.log('\n=== 6. MESSAGES ===');
  const inputCount = await page1.locator('.message-input, textarea, [contenteditable]').count();
  if (inputCount > 0) {
    const testMsg = `Diagnostic ${Date.now()}`;
    await page1.locator('.message-input, textarea, [contenteditable]').first().click();
    await page1.waitForTimeout(300);
    await page1.keyboard.type(testMsg);
    await page1.keyboard.press('Enter');
    await page1.waitForTimeout(2000);
    
    const msgVisible = await page1.locator(`text=${testMsg}`).count();
    console.log('Send message:', msgVisible > 0 ? '✅ OK' : '❌ FAILED');
  } else {
    console.log('No message input found');
  }
  
  // ── Summary ───────────────────────────────────────────────────────
  console.log('\n=== CONSOLE LOGS (SW) ===');
  const swLogs = logs.filter(l => l.includes('[SW]'));
  for (const l of swLogs.slice(0, 10)) console.log(`  ${l}`);
  
  if (errors.length > 0) {
    console.log('\n=== ERRORS ===');
    for (const e of errors.slice(0, 10)) console.log(`  ${e}`);
  }
  
  await ctx1.close();
  console.log('\n=== DIAGNOSTIC COMPLETE ===');
});
