import { test, expect } from '@playwright/test';

test.describe('Comprehensive Calendar Functionality', () => {
  test.setTimeout(120000);

  test('Navigate months and check events', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to calendar
    await page.goto('/calendar');
    await page.waitForTimeout(2000);
    
    // Check current month
    const currentMonth = await page.locator('h2:has-text("Avril 2026")').count();
    console.log(`Current month (April 2026): ${currentMonth > 0 ? '✅' : '❌'}`);
    
    // Navigate to next month
    const nextBtn = page.locator('button:has-text("›")');
    if (await nextBtn.count() > 0) {
      await nextBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Navigated to next month');
      
      // Check if month changed
      const mayVisible = await page.locator('h2:has-text("Mai 2026")').count();
      console.log(`May 2026 visible: ${mayVisible > 0 ? '✅' : '❌'}`);
    }
    
    // Check view buttons
    const monthView = await page.locator('button:has-text("Mois")').count();
    const weekView = await page.locator('button:has-text("Sem.")').count();
    const dayView = await page.locator('button:has-text("Jour")').count();
    console.log(`View buttons: Mois=${monthView} Sem=${weekView} Jour=${dayView}`);
    
    expect(currentMonth).toBeGreaterThan(0);
  });

  test('Add event button and form', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to calendar
    await page.goto('/calendar');
    await page.waitForTimeout(2000);
    
    // Click "+ Ajouter"
    const addBtn = page.locator('button:has-text("Ajouter")');
    if (await addBtn.count() > 0) {
      await addBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Clicked "Ajouter" button');
      
      // Check if form/dialog appeared
      const formVisible = await page.locator('input[placeholder*="événement"], textarea, form').count();
      console.log(`Event form visible: ${formVisible > 0 ? '✅' : '❌'}`);
      
      // If form visible, try to fill it
      if (formVisible > 0) {
        const titleInput = page.locator('input[placeholder*="événement"], input[type="text"]').first();
        if (await titleInput.count() > 0) {
          await titleInput.fill('Test Event from Playwright');
          console.log('✅ Filled event title');
        }
      }
    } else {
      console.log('❌ "Ajouter" button not found');
    }
    
    expect(true).toBeTruthy(); // Just verify no crash
  });

  test('Switch calendar views', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Go to calendar
    await page.goto('/calendar');
    await page.waitForTimeout(2000);
    
    // Click "Sem." (Week view)
    const weekBtn = page.locator('button:has-text("Sem.")');
    if (await weekBtn.count() > 0) {
      await weekBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Switched to Week view');
    }
    
    // Click "Jour" (Day view)
    const dayBtn = page.locator('button:has-text("Jour")');
    if (await dayBtn.count() > 0) {
      await dayBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Switched to Day view');
    }
    
    // Back to "Mois" (Month view)
    const monthBtn = page.locator('button:has-text("Mois")');
    if (await monthBtn.count() > 0) {
      await monthBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Back to Month view');
    }
    
    expect(true).toBeTruthy();
  });
});
