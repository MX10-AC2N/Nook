import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('Debug conversations loading', async ({ page }) => {
  // Login
  await page.goto('https://192.168.1.192:6443/login');
  await page.waitForTimeout(2000);
  await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page.fill('input[type="password"]', 'Hermes2026!');
  await page.click('button[type="submit"]');
  await page.waitForTimeout(3000);
  
  // Navigate to chat
  await page.goto('https://192.168.1.192:6443/chat');
  await page.waitForTimeout(3000);
  
  // Check API response
  const apiResult = await page.evaluate(async () => {
    try {
      const r = await fetch('/api/conversations', { credentials: 'include' });
      if (!r.ok) return { error: r.status, statusText: r.statusText };
      const data = await r.json();
      return { 
        conversations: data, 
        count: Array.isArray(data) ? data.length : (data.conversations?.length || 0),
        isArray: Array.isArray(data)
      };
    } catch (e) {
      return { error: String(e) };
    }
  });
  
  console.log('API result:', JSON.stringify(apiResult, null, 2));
  
  // Check sidebar content
  const sidebarContent = await page.evaluate(() => {
    const sidebar = document.querySelector('.conversations-sidebar');
    const items = document.querySelectorAll('.conversation-item');
    const emptyState = document.querySelector('.sidebar-empty');
    const loading = document.querySelector('.sidebar-loading');
    
    return {
      sidebarExists: !!sidebar,
      itemCount: items.length,
      emptyStateExists: !!emptyState,
      loadingExists: !!loading,
      sidebarHTML: sidebar?.innerHTML?.slice(0, 1000) || 'no sidebar'
    };
  });
  
  console.log('Sidebar:', JSON.stringify(sidebarContent, null, 2));
  
  // Check for console errors
  const errors: string[] = [];
  page.on('console', msg => {
    if (msg.type() === 'error') errors.push(msg.text());
  });
  
  await page.waitForTimeout(1000);
  
  if (errors.length > 0) {
    console.log('\nConsole errors:');
    for (const e of errors.slice(0, 10)) console.log(`  ${e}`);
  }
});
