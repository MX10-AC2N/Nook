import { test } from '@playwright/test';

const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';

test.use({ ignoreHTTPSErrors: true });

test('Push notification diagnostic', async ({ page }) => {
  const logs: string[] = [];
  const errors: string[] = [];
  
  page.on('console', msg => {
    logs.push(msg.text());
    if (msg.type() === 'error') errors.push(msg.text());
  });
  
  // Login
  await page.goto(`${BASE_URL}/login`);
  await page.waitForTimeout(2000);
  await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page.fill('input[type="password"]', 'Hermes2026!');
  await page.click('button[type="submit"]');
  await page.waitForTimeout(3000);
  
  // Navigate to settings
  await page.goto(`${BASE_URL}/settings`);
  await page.waitForTimeout(3000);
  
  // Check service worker
  const swInfo = await page.evaluate(async () => {
    if (!('serviceWorker' in navigator)) return { supported: false };
    
    // Attendre max 10s pour le SW
    try {
      const reg = await Promise.race([
        navigator.serviceWorker.ready,
        new Promise<null>((_, reject) => setTimeout(() => reject('timeout'), 10000))
      ]);
      
      return {
        supported: true,
        registered: !!reg,
        active: !!reg?.active,
        installing: !!reg?.installing,
        waiting: !!reg?.waiting,
        scope: reg?.scope
      };
    } catch (e) {
      return { supported: true, error: String(e) };
    }
  });
  
  console.log('=== Service Worker ===');
  console.log('Supported:', swInfo.supported ? '✅' : '❌');
  console.log('Registered:', swInfo.registered ? '✅' : '❌');
  console.log('Active:', swInfo.active ? '✅' : '❌');
  if (swInfo.installing) console.log('Installing: ⏳');
  if (swInfo.waiting) console.log('Waiting: ⏳');
  if (swInfo.error) console.log('Error:', swInfo.error);
  if (swInfo.scope) console.log('Scope:', swInfo.scope);
  
  // Check push subscription
  const pushInfo = await page.evaluate(async () => {
    if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
      return { supported: false };
    }
    try {
      const reg = await navigator.serviceWorker.ready;
      const sub = await reg.pushManager.getSubscription();
      return {
        supported: true,
        subscribed: !!sub,
        endpoint: sub?.endpoint
      };
    } catch (e) {
      return { supported: true, error: String(e) };
    }
  });
  
  console.log('\n=== Push Subscription ===');
  console.log('Supported:', pushInfo.supported ? '✅' : '❌');
  console.log('Subscribed:', pushInfo.subscribed ? '✅' : '❌');
  if (pushInfo.endpoint) console.log('Endpoint:', pushInfo.endpoint.slice(0, 50) + '...');
  if (pushInfo.error) console.log('Error:', pushInfo.error);
  
  // Check VAPID key
  const vapidInfo = await page.evaluate(async () => {
    try {
      const r = await fetch('/api/vapid-public-key');
      const key = await r.text();
      return { ok: r.ok, key, length: key.length };
    } catch (e) {
      return { ok: false, error: String(e) };
    }
  });
  
  console.log('\n=== VAPID Key ===');
  console.log('Endpoint:', vapidInfo.ok ? '✅ OK' : '❌ FAILED');
  if (vapidInfo.key) {
    console.log('Key:', vapidInfo.key.slice(0, 40) + '...');
    console.log('Length:', vapidInfo.length, 'chars');
    // Vérifier le format (doit être base64url)
    const isBase64url = /^[A-Za-z0-9_-]+$/.test(vapidInfo.key);
    console.log('Format base64url:', isBase64url ? '✅' : '❌');
  }
  
  // Check SW logs
  console.log('\n=== SW Logs ===');
  const swLogs = logs.filter(l => l.includes('[SW]'));
  if (swLogs.length > 0) {
    for (const l of swLogs.slice(0, 10)) console.log(`  ${l}`);
  } else {
    console.log('  No SW logs found');
  }
  
  // Check for errors
  if (errors.length > 0) {
    console.log('\n=== Errors ===');
    for (const e of errors.slice(0, 10)) console.log(`  ${e}`);
  }
});
