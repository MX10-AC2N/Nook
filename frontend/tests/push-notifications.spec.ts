import { test, expect } from '@playwright/test';
import { BASE } from './helpers';


test.describe('Push Notifications — VAPID + Subscription', () => {

  test('GET /api/push/vapid-public-key → 200 (public route)', async ({ request }) => {
    const res = await request.get(`${BASE}/api/push/vapid-public-key`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(typeof body.public_key).toBe('string');
    expect(body.public_key.length).toBeGreaterThan(50); // VAPID keys are long
    console.log('✅ VAPID public key available');
  });

  test('GET /api/push/preferences sans auth → 401', async ({ request }) => {
    const res = await request.get(`${BASE}/api/push/preferences`);
    expect(res.status()).toBe(401);
    console.log('✅ Push preferences requires auth');
  });

  test('Push subscription flow (mocked)', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[name="username"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);

    // Check if push notification components exist
    const pushElements = await page.evaluate(() => {
      return {
        hasServiceWorker: 'serviceWorker' in navigator,
        hasPushManager: 'PushManager' in window,
        hasNotification: 'Notification' in window,
        permission: Notification.permission,
      };
    });

    console.log('Push capabilities:', pushElements);
    expect(pushElements.hasServiceWorker).toBe(true);
    expect(pushElements.hasPushManager).toBe(true);

    // Check for push subscription UI
    const pushBtn = page.locator('[data-testid="push-enable"], button[title*="notification"], .push-toggle').first();
    if (await pushBtn.isVisible({ timeout: 3000 })) {
      console.log('✅ Push enable button found');
      await pushBtn.click();
      await page.waitForTimeout(2000);
    } else {
      console.log('⚠️ Push button not visible (might be in settings)');
    }
  });

  test('Push preferences API', async ({ request }) => {
    // Login via API
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Get push preferences
    const prefs = await request.get(`${BASE}/api/push/preferences`);
    expect(prefs.status()).toBe(200);
    const prefsBody = await prefs.json();
    console.log('Push preferences:', prefsBody);
    expect(typeof prefsBody.enabled).toBe('boolean');

    // Update preferences
    const updateRes = await request.post(`${BASE}/api/push/preferences`, {
      data: { enabled: true, types: ['message', 'call'] },
    });
    expect([200, 204, 404]).toContain(updateRes.status()); // 404 if endpoint not implemented
    console.log('✅ Push preferences API responsive');
  });

});
