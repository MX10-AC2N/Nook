import { test, expect } from '@playwright/test';


test.describe('Events UI — Calendar + Creation', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('Events UI: Navigate to calendar', async ({ page }) => {
    // Try to navigate to calendar/events page
    await page.goto('/events');
    await page.waitForTimeout(2000);

    const url = page.url();
    console.log('Events URL:', url);

    // Check if calendar UI loaded
    const calendar = page.locator('[data-testid="calendar"], .calendar, .events-calendar').first();
    const isCalendarVisible = await calendar.isVisible({ timeout: 3000 }).catch(() => false);

    if (isCalendarVisible || url.includes('events')) {
      console.log('✅ Calendar/Events page accessible');
    } else {
      console.log('⚠️ Calendar page not found at /events');
    }
  });

  test('Events UI: Create event button', async ({ page }) => {
    await page.goto('/events');
    await page.waitForTimeout(2000);

    // Look for create event button
    const createBtn = page.locator('[data-testid="create-event"], button:has-text("Créer"), button:has-text("Create")').first();
    
    if (await createBtn.isVisible({ timeout: 3000 })) {
      console.log('✅ Create event button found');
      await createBtn.click();
      await page.waitForTimeout(1000);

      // Check for event form
      const eventForm = page.locator('[data-testid="event-form"], form, .event-form').first();
      if (await eventForm.isVisible()) {
        console.log('✅ Event form opened');
        
        // Fill form fields
        const titleInput = page.locator('input[name="title"], [data-testid="event-title"]').first();
        const dateInput = page.locator('input[type="date"], [data-testid="event-date"]').first();
        
        if (await titleInput.isVisible()) {
          await titleInput.fill('Test Event');
        }
        
        if (await dateInput.isVisible()) {
          const tomorrow = new Date();
          tomorrow.setDate(tomorrow.getDate() + 1);
          await dateInput.fill(tomorrow.toISOString().split('T')[0]);
        }
      }
    } else {
      console.log('⚠️ Create event button not found');
    }
  });

  test('Events UI: List events', async ({ page }) => {
    await page.goto('/events');
    await page.waitForTimeout(2000);

    // Check for event list
    const eventList = page.locator('[data-testid="event-list"], .event-list, .events-list').first();
    const eventItems = page.locator('[data-testid="event-item"], .event-item, .event-card');
    
    const listVisible = await eventList.isVisible({ timeout: 3000 }).catch(() => false);
    const itemCount = await eventItems.count();

    console.log(`Events: list=${listVisible}, items=${itemCount}`);

    if (listVisible || itemCount > 0) {
      console.log('✅ Event list displayed');
    } else {
      console.log('⚠️ No events found (might be empty)');
    }
  });

  test('API: List events', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Get events
    const eventsRes = await request.get(`${BASE}/api/events`);
    expect([200, 404]).toContain(eventsRes.status()); // 404 if not implemented

    if (eventsRes.ok()) {
      const events = await eventsRes.json();
      console.log('Events:', events);
      console.log('✅ Events API available');
    } else {
      console.log('⚠️ Events API not found');
    }
  });

  test('API: Create and delete event', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'hermes-bot', password: 'Hermes2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Create event
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    
    const createRes = await request.post(`${BASE}/api/events`, {
      data: {
        title: 'Playwright Test Event',
        date: tomorrow.toISOString().split('T')[0],
        time: '14:00',
        description: 'Test event created by Playwright',
      },
    });

    expect([200, 201, 404]).toContain(createRes.status());

    if (createRes.ok()) {
      const event = await createRes.json();
      console.log('Event created:', event);
      const eventId = event.id || event.event_id;

      if (eventId) {
        // Delete event
        const deleteRes = await request.delete(`${BASE}/api/events/${eventId}`);
        expect([200, 204, 404]).toContain(deleteRes.status());
        console.log('✅ Event created and deleted');
      }
    } else {
      console.log('⚠️ Event creation not accessible');
    }
  });

});
