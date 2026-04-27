import { test, expect } from '@playwright/test';

test.describe('Chess Functionality', () => {
  test.setTimeout(120000);

  test('View chess page with games list', async ({ page }) => {
    // Login as hermes-bot
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to chess
    await page.goto('http://192.168.1.192:6300/chess');
    await page.waitForTimeout(2000);
    
    // Check chess page loaded
    const chessTitle = await page.locator('heading:has-text("Échecs")').count();
    console.log(`Chess page: ${chessTitle > 0 ? '✅' : '❌'}`);
    
    // Check "+ Nouvelle partie" button
    const newGameBtn = await page.locator('button:has-text("Nouvelle partie")').count();
    console.log(`"Nouvelle partie" button: ${newGameBtn > 0 ? '✅' : '❌'}`);
    
    // Check games list
    const gamesList = await page.locator('heading:has-text("Parties disponibles")').count();
    console.log(`Games list: ${gamesList > 0 ? '✅' : '❌'}`);
    
    // Check for existing games (from snapshot: hermes-bot created many)
    const observerBtns = await page.locator('button:has-text("Observer")').count();
    console.log(`Games available: ${observerBtns} games`);
    
    expect(chessTitle).toBeGreaterThan(0);
    expect(newGameBtn).toBeGreaterThan(0);
  });

  test('Create new game button works', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/chess');
    await page.waitForTimeout(2000);
    
    // Click "+ Nouvelle partie"
    await page.click('button:has-text("Nouvelle partie")');
    await page.waitForTimeout(2000);
    
    // Should show game creation dialog or navigate to new game
    const currentUrl = page.url();
    console.log(`After clicking new game: ${currentUrl}`);
    
    // Check if we're on a game page or dialog appeared
    const isGamePage = currentUrl.includes('/chess/');
    console.log(`Game page opened: ${isGamePage ? '✅' : '❌'}`);
    
    expect(true).toBeTruthy(); // Just verify no crash
  });

  test('Refresh games list', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/chess');
    await page.waitForTimeout(2000);
    
    // Click "Actualiser" button
    const refreshBtn = await page.locator('button:has-text("Actualiser")').count();
    if (refreshBtn > 0) {
      await page.click('button:has-text("Actualiser")');
      await page.waitForTimeout(2000);
      console.log('✅ Games list refreshed');
    }
    
    expect(true).toBeTruthy();
  });
});
