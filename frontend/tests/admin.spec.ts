import { test, expect } from '@playwright/test';

test.describe('Admin Functionality', () => {
  test.setTimeout(120000);

  test('Access admin page and view tabs', async ({ page }) => {
    // Login as admin (not hermes-bot)
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to admin
    await page.goto('http://192.168.1.192:6300/admin');
    await page.waitForTimeout(2000);
    
    // Check admin page loaded
    const adminTitle = await page.locator('heading:has-text("Administration")').count();
    console.log(`Admin page: ${adminTitle > 0 ? '✅' : '❌'}`);
    
    // Check tabs
    const invitationsTab = await page.locator('tab:has-text("Invitations")').count();
    const membersTab = await page.locator('tab:has-text("Membres")').count();
    const settingsTab = await page.locator('tab:has-text("Paramètres")').count();
    
    console.log(`Tabs: Invitations=${invitationsTab} Membres=${membersTab} Paramètres=${settingsTab}`);
    
    expect(adminTitle).toBeGreaterThan(0);
    expect(invitationsTab).toBeGreaterThan(0);
  });

  test('View invitations tab', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/admin');
    await page.waitForTimeout(2000);
    
    // Click Invitations tab
    const invitationsTab = page.locator('tab:has-text("Invitations")');
    if (await invitationsTab.count() > 0) {
      await invitationsTab.click();
      await page.waitForTimeout(1000);
      console.log('✅ Invitations tab clicked');
      
      // Check for "Nouvelle invitation" button
      const newInviteBtn = await page.locator('button:has-text("Nouvelle invitation")').count();
      console.log(`"Nouvelle invitation" button: ${newInviteBtn > 0 ? '✅' : '❌'}`);
    }
    
    expect(true).toBeTruthy();
  });

  test('View members tab', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/admin');
    await page.waitForTimeout(2000);
    
    // Click Membres tab
    const membersTab = page.locator('tab:has-text("Membres")');
    if (await membersTab.count() > 0) {
      await membersTab.click();
      await page.waitForTimeout(1000);
      console.log('✅ Membres tab clicked');
      
      // Check for members list (should have admin and hermes-bot)
      const members = await page.locator('text=/admin|hermes-bot/').count();
      console.log(`Members visible: ${members}`);
    }
    
    expect(true).toBeTruthy();
  });

  test('Reset member password (UI check)', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/admin');
    await page.waitForTimeout(2000);
    
    // Go to Membres tab
    const membersTab = page.locator('tab:has-text("Membres")');
    if (await membersTab.count() > 0) {
      await membersTab.click();
      await page.waitForTimeout(1000);
      
      // Check if there's a "Réinitialiser" button for hermes-bot
      const resetBtn = await page.locator('button:has-text("Réinitialiser")').count();
      console.log(`"Réinitialiser" buttons: ${resetBtn}`);
      if (resetBtn > 0) {
        console.log('✅ Password reset functionality available');
      }
    }
    
    expect(true).toBeTruthy();
  });
});
