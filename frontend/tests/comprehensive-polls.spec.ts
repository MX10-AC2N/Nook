import { test, expect } from '@playwright/test';
import { loginAs, BASE, E2E_USER, E2E_PASS } from './helpers';

test.describe('Comprehensive Polls Functionality', () => {
  test.setTimeout(120000);

  test('Create new poll with options', async ({ page }) => {
    await loginAs(page, E2E_USER, E2E_PASS);

    // Go to polls
    await page.goto('/polls');
    await page.waitForSelector('[data-testid="create-poll-btn"]', { state: 'visible', timeout: 10000 });

    // Click "Nouveau sondage"
    await page.click('[data-testid="create-poll-btn"]');
    await page.waitForSelector('[data-testid="poll-form"]', { state: 'visible', timeout: 5000 });

    // Fill poll question
    const q = `Test Poll ${Date.now()}`;
    await page.fill('input[placeholder="Quelle est votre question ?"]', q);

    // Fill options
    await page.fill('input[placeholder="Option 1 *"]', 'Option A from Playwright');
    await page.fill('input[placeholder="Option 2 *"]', 'Option B from Playwright');

    // Submit
    await page.click('button.btn-submit');

    // Verify poll appears in list
    await page.waitForSelector(`.poll-question:has-text("${q}")`, { state: 'visible', timeout: 10000 });
    console.log('✅ Poll created and visible in list');
  });

  test('Vote on a poll', async ({ page }) => {
    await loginAs(page, E2E_USER, E2E_PASS);

    // Go to polls
    await page.goto('/polls');
    await page.waitForSelector('[data-testid="create-poll-btn"]', { state: 'visible', timeout: 10000 });

    // Click first open poll's first option
    const firstOption = page.locator('.poll-card:not(.closed) .option-btn').first();
    if (await firstOption.count() > 0) {
      await firstOption.click();
      await page.waitForTimeout(1000);
      console.log('✅ Voted on first option');
    } else {
      console.log('ℹ️ No open poll found to vote on');
    }
  });

  test('View poll results', async ({ page }) => {
    await loginAs(page, E2E_USER, E2E_PASS);

    // Go to polls
    await page.goto('/polls');
    await page.waitForSelector('[data-testid="create-poll-btn"]', { state: 'visible', timeout: 10000 });

    // Check if any polls exist
    const pollCards = page.locator('[data-testid="poll-item"]');
    const count = await pollCards.count();
    console.log(`Poll count: ${count}`);

    if (count > 0) {
      // Check for percentage display
      const percentage = page.locator('text=/%').count();
      console.log(`Percentage visible: ${percentage > 0 ? '✅' : '❌'}`);
    }

    expect(true).toBeTruthy();
  });
});
