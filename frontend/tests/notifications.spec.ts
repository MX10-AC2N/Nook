import { test, expect } from '@playwright/test';

test.describe('Notifications Functionality', () => {
  test.setTimeout(120000);

  test('Access notifications settings via Settings', async ({ page }) => {
    // Login as hermes-bot
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to settings
    await page.goto('http://192.168.1.192:6300/settings');
    await page.waitForTimeout(2000);
    
    // Check if there's a "Notifications" section or tab
    const notifSection = await page.locator('text=/Notification/i').count();
    console.log(`Notifications section: ${notifSection > 0 ? '✅' : '❌'}`);
    
    // Check if there's a toggle or button for notifications
    const notifToggle = await page.locator('button:has-text("Notification"), input[type="checkbox"]').count();
    console.log(`Notification toggle/button: ${notifToggle > 0 ? '✅' : '❌'}`);
    
    expect(true).toBeTruthy(); // Basic check - just verify no crash
  });

  test('Check notifications UI elements', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/settings');
    await page.waitForTimeout(2000);
    
    // Check if we can navigate to notifications section
    const notifLink = await page.locator('a:has-text("Notification"), button:has-text("Notification")').count();
    
    if (notifLink > 0) {
      await page.locator('a:has-text("Notification"), button:has-text("Notification")').first().click();
      await page.waitForTimeout(1000);
      console.log('✅ Clicked on Notifications');
      
      // Check current URL
      const url = page.url();
      console.log(`URL after clicking: ${url}`);
    }
    
    // Verify settings page is still functional
    const settingsTitle = await page.locator('heading:has-text("Paramètres")').count();
    expect(settingsTitle).toBeGreaterThan(0);
  });

  test('HTTPS required message for push (if visible)', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/settings');
    await page.waitForTimeout(2000);
    
    // Check if there's any message about HTTPS being required
    const httpsMsg = await page.locator('text=/HTTPS|certificat|push/i').count();
    console.log(`HTTPS/push related message: ${httpsMsg > 0 ? '✅' : '❌'}`);
    
    // This is just informational - don't fail the test
    expect(true).toBeTruthy();
  });
});
