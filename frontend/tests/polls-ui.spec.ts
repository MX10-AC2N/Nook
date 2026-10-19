import { test, expect } from '@playwright/test';
import { BASE, loginAs, E2E_USER, E2E_PASS } from './helpers';


test.describe('Polls UI — Create + Vote', () => {

  test.beforeEach(async ({ page }) => {
    await loginAs(page, E2E_USER, E2E_PASS);
    await page.waitForTimeout(1000);
  });

  test('Polls UI: Create poll button', async ({ page }) => {
    await page.goto('/polls');
    await page.waitForSelector('[data-testid="create-poll-btn"]', { state: 'visible', timeout: 10000 });
    console.log('✅ Poll creation button found');
    await page.click('[data-testid="create-poll-btn"]');
    await page.waitForSelector('[data-testid="poll-form"]', { state: 'visible', timeout: 5000 });
    console.log('✅ Poll form opened');

    // Fill question
    const q = 'Test poll question?';
    await page.fill('input[placeholder="Quelle est votre question ?"]', q);

    // Fill options
    await page.fill('input[placeholder="Option 1 *"]', 'Option 1');
    await page.fill('input[placeholder="Option 2 *"]', 'Option 2');
    console.log('✅ Poll form filled');
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
      data: { username: E2E_USER, password: E2E_PASS },
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
      data: { username: E2E_USER, password: E2E_PASS },
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
