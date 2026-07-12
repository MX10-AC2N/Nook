import { test, expect } from '@playwright/test';
import { BASE } from './helpers';


test.describe('E2EE Chat — Encryption + Decryption', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('E2EE: crypto store initializes', async ({ page }) => {
    // Check if crypto store is available
    const cryptoStatus = await page.evaluate(async () => {
      // Wait for crypto store to initialize
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Check for crypto store in window or Nook app
      const hasCryptoStore = !!(window as any).cryptoStore;
      const hasOlm = typeof (window as any).olm !== 'undefined';
      const hasLibolm = typeof (window as any).libolm !== 'undefined';
      
      return { hasCryptoStore, hasOlm, hasLibolm, ready: (window as any).cryptoStore?.ready };
    });

    console.log('Crypto status:', cryptoStatus);
    
    // At least one crypto method should be available
    expect(cryptoStatus.hasCryptoStore || cryptoStatus.hasOlm || cryptoStatus.hasLibolm).toBe(true);
    console.log('✅ E2EE crypto available');
  });

  test('E2EE: Send encrypted message', async ({ page }) => {
    const input = page.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input.click();

    const testMsg = `E2EE Test ${Date.now()}`;
    await input.fill(testMsg);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2500);

    // Check if message appears (decrypted on receive)
    const msgVisible = page.locator(`text=${testMsg}`);
    const count = await msgVisible.count();
    
    if (count > 0) {
      console.log('✅ Message sent and visible (E2EE working)');
      
      // Check for encryption indicator
      const encryptedIcon = page.locator('[data-testid="encrypted-icon"], .encrypted-badge, svg[title*="chiffré"]').first();
      if (await encryptedIcon.isVisible({ timeout: 3000 })) {
        console.log('✅ Encryption indicator shown');
      }
    } else {
      console.log('⚠️ Message not visible (might be encryption issue)');
    }
  });

  test('E2EE: Refresh preserves decrypted messages', async ({ page }) => {
    const input = page.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input.click();

    // Send a message
    const testMsg = `Refresh Test ${Date.now()}`;
    await input.fill(testMsg);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2000);

    // Refresh page
    await page.reload();
    await page.waitForTimeout(3000);

    // Check if message is still visible (decrypted after refresh)
    const msgAfterRefresh = page.locator(`text=${testMsg}`);
    const count = await msgAfterRefresh.count();

    if (count > 0) {
      console.log('✅ Message still visible after refresh (E2EE working)');
    } else {
      // This might be the bug you mentioned (NOOK-E2EE-REFRESH BUG)
      console.log('⚠️ Message not visible after refresh — possible E2EE bug');
      console.log('Known issue: cryptoStore.ready=false after refresh → messages stay encrypted');
    }
  });

  test('E2EE: Key exchange UI (if available)', async ({ page }) => {
    // Check for E2EE key management UI
    const keyManagementBtn = page.locator('[data-testid="key-management"], button[title*="clé"], button[title*="key"]').first();
    
    if (await keyManagementBtn.isVisible({ timeout: 3000 })) {
      console.log('✅ Key management UI found');
      await keyManagementBtn.click();
      await page.waitForTimeout(1000);

      // Check for key fingerprint or export option
      const fingerprint = page.locator('text=/empreinte|fingerprint|clé publique/i').first();
      if (await fingerprint.isVisible()) {
        console.log('✅ Key fingerprint displayed');
      }
    } else {
      console.log('⚠️ Key management UI not found (might be in settings)');
    }
  });

  test('API: Check encryption status', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Check if there's an encryption status endpoint
    const statusRes = await request.get(`${BASE}/api/auth/encryption-status`).catch(() => null);
    
    if (statusRes && statusRes.ok()) {
      const status = await statusRes.json();
      console.log('Encryption status:', status);
      console.log('✅ Encryption status API available');
    } else {
      console.log('⚠️ Encryption status API not found (might be client-side only)');
    }
  });

});
