import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('Call signaling debug — check WebSocket messages', async ({ browser }) => {
  const ctx1 = await browser.newContext();
  const ctx2 = await browser.newContext();
  
  const page1 = await ctx1.newPage();
  const page2 = await ctx2.newPage();
  
  // Track WebSocket messages
  const wsMessages1: any[] = [];
  const wsMessages2: any[] = [];
  
  page1.on('websocket', ws => {
    ws.on('framereceived', frame => {
      if (frame.payload) {
        try {
          const data = JSON.parse(frame.payload.toString());
          wsMessages1.push(data);
          console.log(`[Page1] WS received: ${data.type}`);
        } catch {}
      }
    });
  });
  
  page2.on('websocket', ws => {
    ws.on('framereceived', frame => {
      if (frame.payload) {
        try {
          const data = JSON.parse(frame.payload.toString());
          wsMessages2.push(data);
          console.log(`[Page2] WS received: ${data.type}`);
        } catch {}
      }
    });
  });
  
  // Login as hermes-bot (page1)
  await page1.goto('https://192.168.1.192:6443/login');
  await page1.waitForTimeout(2000);
  await page1.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page1.fill('input[type="password"]', 'Hermes2026!');
  await page1.click('button[type="submit"]');
  await page1.waitForTimeout(3000);
  console.log('✅ hermes-bot logged in');
  
  // Login as admin (page2)
  await page2.goto('https://192.168.1.192:6443/login');
  await page2.waitForTimeout(2000);
  await page2.fill('input[type="text"], input[name="username"]', 'admin');
  await page2.fill('input[type="password"]', 'admin123');
  await page2.click('button[type="submit"]');
  await page2.waitForTimeout(3000);
  console.log('✅ admin logged in');
  
  // Both navigate to chat to establish WebSocket
  await page1.goto('https://192.168.1.192:6443/chat');
  await page1.waitForTimeout(2000);
  await page2.goto('https://192.168.1.192:6443/chat');
  await page2.waitForTimeout(2000);
  
  // Get conversation ID
  const convId = await page1.evaluate(async () => {
    const r = await fetch('/api/conversations', { credentials: 'include' });
    const convs = await r.json();
    const dm = convs.find((c: any) => !c.is_group && c.id !== 'default_global');
    return dm?.id;
  });
  console.log('Conversation ID:', convId);
  
  if (!convId) {
    console.log('❌ No DM conversation found');
    return;
  }
  
  // Check WebSocket connections
  const ws1Connected = await page1.evaluate(() => {
    return (window as any).__chatStore?.wsConnected || false;
  });
  const ws2Connected = await page2.evaluate(() => {
    return (window as any).__chatStore?.wsConnected || false;
  });
  console.log('Page1 WS connected:', ws1Connected);
  console.log('Page2 WS connected:', ws2Connected);
  
  // Navigate to call page (page1)
  await page1.goto(`https://192.168.1.192:6443/call/${convId}?type=audio`);
  await page1.waitForTimeout(3000);
  
  // Check for call_request messages
  const callRequests1 = wsMessages1.filter(m => m.type === 'call_request');
  const callRequests2 = wsMessages2.filter(m => m.type === 'call_request');
  console.log('Call requests on page1:', callRequests1.length);
  console.log('Call requests on page2:', callRequests2.length);
  
  // Check for any errors
  if (wsMessages1.length === 0 && wsMessages2.length === 0) {
    console.log('❌ No WebSocket messages received');
  } else {
    console.log('✅ WebSocket messages received');
  }
  
  await ctx1.close();
  await ctx2.close();
});
