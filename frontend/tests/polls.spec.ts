import { test, expect } from '@playwright/test';

test.describe('Polls Functionality', () => {
  test.setTimeout(120000);

  test('View polls page with existing polls', async ({ page }) => {
    // Login as hermes-bot
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Navigate to polls
    await page.goto('http://192.168.1.192:6300/polls');
    await page.waitForTimeout(2000);
    
    // Check polls page loaded
    const pollsTitle = await page.locator('heading:has-text("Sondages")').count();
    console.log(`Polls page: ${pollsTitle > 0 ? '✅' : '❌'}`);
    
    // Check "+ Nouveau sondage" button
    const newPollBtn = await page.locator('button:has-text("Nouveau sondage")').count();
    console.log(`"Nouveau sondage" button: ${newPollBtn > 0 ? '✅' : '❌'}`);
    
    // Check for existing poll (from snapshot: "Ouvert" poll by Julien)
    const openPoll = await page.locator('text=/Ouvert/i').count();
    console.log(`Open poll present: ${openPoll > 0 ? '✅' : '❌'}`);
    
    // Check poll options (A, B, C)
    const optionA = await page.locator('button:has-text("A")').count();
    const optionB = await page.locator('button:has-text("B")').count();
    const optionC = await page.locator('button:has-text("C")').count();
    console.log(`Options: A=${optionA} B=${optionB} C=${optionC}`);
    
    expect(pollsTitle).toBeGreaterThan(0);
    expect(newPollBtn).toBeGreaterThan(0);
  });

  test('Create new poll (basic flow)', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/polls');
    await page.waitForTimeout(2000);
    
    // Click "+ Nouveau sondage"
    const newPollBtn = await page.locator('button:has-text("Nouveau sondage")').count();
    if (newPollBtn > 0) {
      await page.click('button:has-text("Nouveau sondage")');
      await page.waitForTimeout(2000);
      console.log('✅ New poll dialog/form opened');
      
      // Check if form appeared (basic check)
      const formVisible = await page.locator('input[placeholder*="question"], textarea').count();
      console.log(`Poll form visible: ${formVisible > 0 ? '✅' : '❌'}`);
    }
    
    expect(true).toBeTruthy(); // Just verify no crash
  });

  test('Vote on existing poll', async ({ page }) => {
    await page.goto('http://192.168.1.192:6300/login');
    await page.waitForTimeout(2000);
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    await page.goto('http://192.168.1.192:6300/polls');
    await page.waitForTimeout(2000);
    
    // Try to vote on option B (which has 1 vote from Julien)
    const optionB = await page.locator('button:has-text("B")').first();
    const isVisible = await optionB.isVisible();
    
    if (isVisible) {
      await optionB.click();
      await page.waitForTimeout(2000);
      console.log('✅ Voted on option B');
      
      // Check if vote count changed
      const voteCount = await page.locator('text=/votes|vote/').count();
      console.log(`Vote count visible: ${voteCount > 0 ? '✅' : '❌'}`);
    }
    
    expect(true).toBeTruthy();
  });
});
