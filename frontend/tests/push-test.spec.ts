import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('Test Service Worker et Push Notifications', async ({ page }) => {
  const logs: string[] = [];
  const errors: string[] = [];
  
  page.on('console', msg => {
    logs.push(msg.text());
    if (msg.type() === 'error') errors.push(msg.text());
  });
  
  // Login
  await page.goto('https://192.168.1.192:6443/login');
  await page.waitForTimeout(2000);
  await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page.fill('input[type="password"]', 'Hermes2026!');
  await page.click('button[type="submit"]');
  await page.waitForTimeout(3000);
  
  console.log('URL après login:', page.url());
  
  // Aller aux paramètres
  await page.goto('https://192.168.1.192:6443/settings');
  await page.waitForTimeout(3000);
  
  // Vérifier Service Worker
  const swInfo = await page.evaluate(async () => {
    if (!('serviceWorker' in navigator)) return { supported: false };
    try {
      const reg = await Promise.race([
        navigator.serviceWorker.ready,
        new Promise<null>((_, reject) => setTimeout(() => reject('timeout'), 10000))
      ]);
      return {
        supported: true,
        registered: !!reg,
        active: !!reg?.active,
        scope: reg?.scope
      };
    } catch (e) {
      return { supported: true, error: String(e) };
    }
  });
  
  console.log('\n=== SERVICE WORKER ===');
  console.log('Supported:', swInfo.supported ? '✅' : '❌');
  console.log('Registered:', swInfo.registered ? '✅' : '❌');
  console.log('Active:', swInfo.active ? '✅' : '❌');
  if (swInfo.scope) console.log('Scope:', swInfo.scope);
  if (swInfo.error) console.log('Error:', swInfo.error);
  
  // Vérifier Push Manager
  const pushInfo = await page.evaluate(async () => {
    if (!('PushManager' in window)) return { supported: false };
    try {
      const reg = await navigator.serviceWorker.ready;
      const sub = await reg.pushManager.getSubscription();
      return {
        supported: true,
        subscribed: !!sub
      };
    } catch (e) {
      return { supported: true, error: String(e) };
    }
  });
  
  console.log('\n=== PUSH MANAGER ===');
  console.log('Supported:', pushInfo.supported ? '✅' : '❌');
  console.log('Subscribed:', pushInfo.subscribed ? '✅' : '❌');
  if (pushInfo.error) console.log('Error:', pushInfo.error);
  
  // Vérifier VAPID
  const vapidInfo = await page.evaluate(async () => {
    try {
      const r = await fetch('/api/vapid-public-key');
      const key = await r.text();
      return { ok: r.ok, key, length: key.length };
    } catch (e) {
      return { ok: false, error: String(e) };
    }
  });
  
  console.log('\n=== VAPID KEY ===');
  console.log('Endpoint:', vapidInfo.ok ? '✅ OK' : '❌ FAILED');
  if (vapidInfo.key) {
    console.log('Key (first 40):', vapidInfo.key.slice(0, 40) + '...');
    console.log('Length:', vapidInfo.length, 'chars');
    const isBase64url = /^[A-Za-z0-9_-]+$/.test(vapidInfo.key);
    console.log('Format base64url:', isBase64url ? '✅' : '❌');
  }
  
  // Logs SW
  console.log('\n=== SW LOGS ===');
  const swLogs = logs.filter(l => l.includes('[SW]'));
  for (const l of swLogs.slice(0, 10)) console.log('  ' + l);
  
  // Erreurs
  if (errors.length > 0) {
    console.log('\n=== ERRORS ===');
    for (const e of errors.slice(0, 10)) console.log('  ' + e);
  }
});
