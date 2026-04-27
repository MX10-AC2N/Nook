import { test, expect } from '@playwright/test';

test.describe('API Endpoints', () => {
  test.setTimeout(60000);

  test('Login API endpoint', async ({ request }) => {
    // Test login API directly
    const response = await request.post('http://192.168.1.192:6300/api/auth/login', {
      data: {
        username: 'hermes-bot',
        password: 'Hermes2026!'
      }
    });
    
    console.log(`Login API status: ${response.status()}`);
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    console.log(`Login success: ${body.success ? '✅' : '❌'}`);
    expect(body.success).toBeTruthy();
  });

  test('Get conversations API', async ({ request }) => {
    // First login to get cookie
    const loginResponse = await request.post('http://192.168.1.192:6300/api/auth/login', {
      data: {
        username: 'hermes-bot',
        password: 'Hermes2026!'
      }
    });
    
    expect(loginResponse.status()).toBe(200);
    
    // Get conversations
    const response = await request.get('http://192.168.1.192:6300/api/conversations');
    
    console.log(`Conversations API status: ${response.status()}`);
    expect(response.status()).toBe(200);
    
    const conversations = await response.json();
    console.log(`Conversations count: ${conversations.length || 'N/A'}`);
    expect(Array.isArray(conversations)).toBeTruthy();
  });

  test('Get calendar events API', async ({ request }) => {
    // Login first
    await request.post('http://192.168.1.192:6300/api/auth/login', {
      data: {
        username: 'hermes-bot',
        password: 'Hermes2026!'
      }
    });
    
    // Get calendar events
    const response = await request.get('http://192.168.1.192:6300/api/calendar/events');
    
    console.log(`Calendar API status: ${response.status()}`);
    // Might return 200 with empty array or 404 if no events
    expect([200, 404]).toContain(response.status());
    
    if (response.status() === 200) {
      const events = await response.json();
      console.log(`Events: ${events.length || 0}`);
    }
  });

  test('Get polls API', async ({ request }) => {
    // Login first
    await request.post('http://192.168.1.192:6300/api/auth/login', {
      data: {
        username: 'hermes-bot',
        password: 'Hermes2026!'
      }
    });
    
    // Get polls
    const response = await request.get('http://192.168.1.192:6300/api/polls');
    
    console.log(`Polls API status: ${response.status()}`);
    expect(response.status()).toBe(200);
    
    const polls = await response.json();
    console.log(`Polls count: ${polls.length || 0}`);
    expect(Array.isArray(polls)).toBeTruthy();
  });

  test('Unauthorized access returns 401', async ({ request }) => {
    // Try to access protected endpoint without login
    const response = await request.get('http://192.168.1.192:6300/api/conversations');
    
    console.log(`Unauthorized status: ${response.status()}`);
    expect(response.status()).toBe(401);
  });

  test('Get chess games API', async ({ request }) => {
    // Login first
    await request.post('http://192.168.1.192:6300/api/auth/login', {
      data: {
        username: 'hermes-bot',
        password: 'Hermes2026!'
      }
    });
    
    // Get chess games
    const response = await request.get('http://192.168.1.192:6300/api/chess/games');
    
    console.log(`Chess API status: ${response.status()}`);
    expect(response.status()).toBe(200);
    
    const games = await response.json();
    console.log(`Chess games: ${games.length || 0}`);
    expect(Array.isArray(games)).toBeTruthy();
  });
});
