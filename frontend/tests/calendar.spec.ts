import { test, expect } from '@playwright/test';

test.describe('Calendar Functionality', () => {
  test.setTimeout(120000);

  test('View calendar and navigate months', async ({ browser }) => {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    
    // Login as hermes-bot
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to calendar
    await page.goto('http://192.168.1.192:6300/calendar');
    await page.waitForTimeout(2000);
    
    // Check calendar is loaded
    const calendarTitle = await page.locator('heading:has-text("Calendrier")').count();
    console.log(`Calendar page: ${calendarTitle > 0 ? '✅' : '❌'}`);
    
    // Check current month is displayed
    const currentMonth = await page.locator('heading:has-text("Avril 2026")').count();
    console.log(`April 2026 displayed: ${currentMonth > 0 ? '✅' : '❌'}`);
    
    // Try to navigate to next month
    const nextButton = await page.locator('button:has-text("›")').count();
    if (nextButton > 0) {
      await page.click('button:has-text("›")');
      await page.waitForTimeout(1000);
      console.log('✅ Navigated to next month');
    }
    
    // Check "À venir" section
    const upcoming = await page.locator('heading:has-text("À venir")').count();
    console.log(`"À venir" section: ${upcoming > 0 ? '✅' : '❌'}`);
    
    expect(calendarTitle).toBeGreaterThan(0);
    
    await ctx.close();
  });

  test('Add event button visible', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/calendar');
    await page.waitForTimeout(2000);
    
    // Check "+ Ajouter" button
    const addButton = await page.locator('button:has-text("Ajouter")').count();
    console.log(`"Ajouter" button: ${addButton > 0 ? '✅' : '❌'}`);
    
    // Check view buttons (Mois, Sem., Jour)
    const monthView = await page.locator('button:has-text("Mois")').count();
    const weekView = await page.locator('button:has-text("Sem.")').count();
    const dayView = await page.locator('button:has-text("Jour")').count();
    
    console.log(`View buttons: Mois=${monthView} Sem=${weekView} Jour=${dayView}`);
    
    expect(addButton).toBeGreaterThan(0);
  });
});
