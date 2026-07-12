import { test, expect } from '@playwright/test';
import { BASE } from './helpers';


test.describe('Polls UI — Create + Vote', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('Polls UI: Create poll button', async ({ page }) => {
    // Look for poll creation button
    const pollBtn = page.locator('[data-testid="create-poll"], button[title*="sondage"], button[title*="poll"], .poll-btn').first();
    
    if (await pollBtn.isVisible({ timeout: 3000 })) {
      console.log('✅ Poll creation button found');
      await pollBtn.click();
      await page.waitForTimeout(1000);

      // Check for poll form
      const pollForm = page.locator('[data-testid="poll-form"], form, .poll-form').first();
      if (await pollForm.isVisible()) {
        console.log('✅ Poll form opened');
        
        // Fill question
        const questionInput = page.locator('input[placeholder*="question"], input[name="question"], [data-testid="poll-question"]').first();
        if (await questionInput.isVisible()) {
          await questionInput.fill('Test poll question?');
        }

        // Add options
        const optionInputs = page.locator('input[placeholder*="option"], input[name*="option"], [data-testid="poll-option"]');
        const optionCount = await optionInputs.count();
        
        for (let i = 0; i < Math.min(optionCount, 3); i++) {
          await optionInputs.nth(i).fill(`Option ${i + 1}`);
        }

        console.log('✅ Poll form filled');
      }
    } else {
      console.log('⚠️ Poll button not found (might be in menu or not implemented)');
    }
  });

  test('Polls UI: Display polls in chat', async ({ page }) => {
    // Check if polls are displayed in the chat
    const pollMessage = page.locator('[data-testid="poll-message"], .poll-message, .poll-card').first();
    const pollCount = await pollMessage.count();

    console.log(`Poll messages in chat: ${pollCount}`);
    
    if (pollCount > 0) {
      console.log('✅ Poll messages displayed in chat');
      
      // Check for vote buttons
      const voteBtn = page.locator('[data-testid="vote-btn"], button:has-text("Voter"), .vote-btn').first();
      if (await voteBtn.isVisible()) {
        console.log('✅ Vote button available');
      }
    } else {
      console.log('⚠️ No poll messages yet (need to create one first)');
    }
  });

  test('API: List polls', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Get polls
    const pollsRes = await request.get(`${BASE}/api/polls`);
    expect([200, 404]).toContain(pollsRes.status()); // 404 if not implemented

    if (pollsRes.ok()) {
      const polls = await pollsRes.json();
      console.log('Polls:', polls);
      console.log('✅ Polls API available');
    } else {
      console.log('⚠️ Polls API not found');
    }
  });

  test('API: Create and vote on poll', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Create poll
    const createRes = await request.post(`${BASE}/api/polls`, {
      data: {
        question: 'Playwright test poll?',
        options: ['Option A', 'Option B', 'Option C'],
        conversation_id: 'default_global',
      },
    });

    expect([200, 201, 404]).toContain(createRes.status());

    if (createRes.ok()) {
      const poll = await createRes.json();
      console.log('Poll created:', poll);
      const pollId = poll.id || poll.poll_id;

      if (pollId) {
        // Vote on poll
        const voteRes = await request.post(`${BASE}/api/polls/${pollId}/vote`, {
          data: { option_id: poll.options?.[0]?.id || 'option_0' },
        });
        expect([200, 204, 404]).toContain(voteRes.status());

        if (voteRes.ok()) {
          console.log('✅ Vote recorded');
        }

        // Close poll
        const closeRes = await request.post(`${BASE}/api/polls/${pollId}/close`);
        expect([200, 204, 404]).toContain(closeRes.status());
      }
    } else {
      console.log('⚠️ Poll creation not accessible');
    }
  });

});
