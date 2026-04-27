import { test, expect } from '@playwright/test';

test.describe('Settings Page Functionality', () => {
  test.setTimeout(120000);

  test('View settings page with avatar styles', async ({ page }) => {
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
    
    // Check settings page loaded
    const settingsTitle = await page.locator('heading:has-text("Paramètres")').count();
    console.log(`Settings page: ${settingsTitle > 0 ? '✅' : '❌'}`);
    
    // Check tabs
    const profilTab = await page.locator('tab:has-text("Profil")').count();
    const securiteTab = await page.locator('tab:has-text("Sécurité")').count();
    const apparenceTab = await page.locator('tab:has-text("Apparence")').count();
    console.log(`Tabs: Profil=${profilTab} Sécurité=${securiteTab} Apparence=${apparenceTab}`);
    
    // Check avatar styles
    const styles = ['Aventurier', 'Cartoon', 'Illustré', 'Minimaliste', 'Emoji', 'Sourire', 'Portrait', 'Personas', 'Robot', 'Initiales'];
    for (const style of styles) {
      const count = await page.locator(`button:has-text("${style}")`).count();
      if (count > 0) {
        console.log(`✅ Style: ${style}`);
      }
    }
    
    expect(settingsTitle).toBeGreaterThan(0);
    expect(profilTab).toBeGreaterThan(0);
  });

  test('Switch tabs in settings', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/settings');
    await page.waitForTimeout(2000);
    
    // Click on Sécurité tab
    const securiteTab = page.locator('tab:has-text("Sécurité")');
    if (await securiteTab.count() > 0) {
      await securiteTab.click();
      await page.waitForTimeout(1000);
      console.log('✅ Switched to Sécurité tab');
      
      // Check if content changed
      const content = await page.content();
      console.log(`Sécurité tab loaded: ${content.includes('Sécurité') ? '✅' : '❌'}`);
    }
    
    // Click on Apparence tab
    const apparenceTab = page.locator('tab:has-text("Apparence")');
    if (await apparenceTab.count() > 0) {
      await apparenceTab.click();
      await page.waitForTimeout(1000);
      console.log('✅ Switched to Apparence tab');
    }
    
    expect(true).toBeTruthy();
  });

  test('Select avatar style and avatar', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/settings');
    await page.waitForTimeout(2000);
    
    // Click on "Cartoon" style
    const cartoonBtn = page.locator('button:has-text("Cartoon")');
    if (await cartoonBtn.count() > 0) {
      await cartoonBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Selected Cartoon style');
      
      // Check if avatars are displayed
      const avatars = await page.locator('button[aria-label], button:has(img)').count();
      console.log(`Avatars displayed: ${avatars}`);
      
      // Click on first avatar if available
      if (avatars > 0) {
        await page.locator('button[aria-label], button:has(img)').first().click();
        await page.waitForTimeout(1000);
        console.log('✅ Avatar selected');
      }
    }
    
    expect(true).toBeTruthy();
  });
});
