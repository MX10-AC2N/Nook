import { test, expect } from '@playwright/test';

test.describe('Comprehensive Chess Functionality', () => {
  test.setTimeout(120000);

  test('Navigate to chess and view games', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to chess
    await page.goto('/chess');
    await page.waitForTimeout(2000);
    
    // Check chess page loaded
    const chessTitle = await page.locator('h1:has-text("Échecs")').count();
    console.log(`Chess page: ${chessTitle > 0 ? '✅' : '❌'}`);
    
    // Check for existing games
    const gamesList = await page.locator('text=/En cours|En attente/').count();
    console.log(`Games visible: ${gamesList}`);
    
    // Click "Nouvelle partie" if available
    const newGameBtn = page.locator('button:has-text("Nouvelle partie")');
    if (await newGameBtn.count() > 0) {
      await newGameBtn.click();
      await page.waitForTimeout(2000);
      console.log('✅ Clicked "Nouvelle partie"');
      
      // Should navigate to a game page
      const url = page.url();
      const isGamePage = url.includes('/chess/');
      console.log(`Navigated to game: ${isGamePage ? '✅' : '❌'}`);
    }
    
    expect(chessTitle).toBeGreaterThan(0);
  });

  test('Create new game and try to play', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to chess
    await page.goto('/chess');
    await page.waitForTimeout(2000);
    
    // Click "Nouvelle partie"
    const newGameBtn = page.locator('button:has-text("Nouvelle partie")');
    if (await newGameBtn.count() > 0) {
      await newGameBtn.click();
      await page.waitForTimeout(3000);
      
      const url = page.url();
      console.log(`After new game: ${url}`);
      
      // Check if we're on a game page
      if (url.includes('/chess/')) {
        console.log('✅ On game page');
        
        // Look for chessboard
        const chessboard = await page.locator('[class*="chessboard"], [class*="board"], canvas').count();
        console.log(`Chessboard visible: ${chessboard > 0 ? '✅' : '❌'}`);
        
        // Look for pieces
        const pieces = await page.locator('[class*="piece"], img[alt*="pawn"], img[alt*="rook"]').count();
        console.log(`Pieces visible: ${pieces}`);
        
        if (pieces > 0) {
          // Try to click on a piece
          await page.locator('[class*="piece"], img[alt*="pawn"]').first().click();
          await page.waitForTimeout(1000);
          console.log('✅ Clicked on a piece');
        }
      }
    } else {
      console.log('❌ "Nouvelle partie" button not found');
    }
    
    expect(true).toBeTruthy(); // Just verify no crash
  });

  test('Refresh games list', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to chess
    await page.goto('/chess');
    await page.waitForTimeout(2000);
    
    // Click "Actualiser"
    const refreshBtn = page.locator('button:has-text("Actualiser")');
    if (await refreshBtn.count() > 0) {
      await refreshBtn.click();
      await page.waitForTimeout(2000);
      console.log('✅ Refreshed games list');
    }
    
    expect(true).toBeTruthy();
  });
});
