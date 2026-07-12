import { test } from '@playwright/test';

const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';

test.use({ ignoreHTTPSErrors: true });

test('Diagnostic Service Worker', async ({ page }) => {
  // Collecter les logs de la console
  const logs: string[] = [];
  page.on('console', msg => logs.push(msg.text()));
  
  // Aller sur la page
  await page.goto(`${BASE_URL}/settings`);
  await page.waitForTimeout(3000);
  
  // Vérifier le Service Worker
  const swInfo = await page.evaluate(async () => {
    if (!('serviceWorker' in navigator)) return { supported: false };
    
    try {
      // Essayer d'enregistrer le SW
      const reg = await navigator.serviceWorker.register('/service-worker.js', { scope: '/' });
      return {
        supported: true,
        registered: true,
        scope: reg.scope,
        active: !!reg.active,
        installing: !!reg.installing,
        waiting: !!reg.waiting
      };
    } catch (error) {
      return {
        supported: true,
        registered: false,
        error: error.toString()
      };
    }
  });
  
  console.log('=== Service Worker ===');
  console.log('Supported:', swInfo.supported ? '✅' : '❌');
  console.log('Registered:', swInfo.registered ? '✅' : '❌');
  console.log('Active:', swInfo.active ? '✅' : '❌');
  console.log('Scope:', swInfo.scope);
  if (swInfo.error) console.log('Error:', swInfo.error);
  
  // Vérifier le contenu du SW
  const swContent = await page.evaluate(async () => {
    try {
      const response = await fetch('/service-worker.js');
      const text = await response.text();
      return {
        status: response.status,
        length: text.length,
        snippet: text.slice(0, 200)
      };
    } catch (error) {
      return { error: error.toString() };
    }
  });
  
  console.log('\n=== Service Worker Content ===');
  console.log('Status:', swContent.status);
  console.log('Length:', swContent.length);
  console.log('Snippet:', swContent.snippet);
  
  // Afficher les logs de la console
  console.log('\n=== Console Logs ===');
  for (const log of logs.slice(0, 20)) {
    console.log(log);
  }
});
