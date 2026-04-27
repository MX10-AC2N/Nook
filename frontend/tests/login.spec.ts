import { test, expect } from '@playwright/test';

test.describe('Login Functionality', () => {
  test.setTimeout(60000);
  
  test('Login with valid credentials', async ({ page }) => {
    // Navigate to login page
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    
    // Verify login page is displayed
    const loginForm = await page.locator('input[type="text"], input[name="username"]').count();
    expect(loginForm).toBeGreaterThan(0);
    console.log('✅ Login page loaded');
    
    // Fill credentials
    await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    
    // Click login button
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Verify redirect away from login page
    const url = page.url();
    const loginSuccess = !url.includes('/login');
    console.log(`Login result: ${loginSuccess ? '✅' : '❌'} (URL: ${url})`);
    
    expect(loginSuccess).toBeTruthy();
  });

  test('Login with invalid credentials', async ({ page }) => {
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    
    // Fill invalid credentials
    await page.fill('input[type="text"]', 'invalid_user');
    await page.fill('input[type="password"]', 'wrong_password');
    
    // Click login button
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Should still be on login page
    const url = page.url();
    const stillOnLogin = url.includes('/login');
    console.log(`Invalid login result: ${stillOnLogin ? '✅' : '❌'} (correctly rejected)`);
    
    expect(stillOnLogin).toBeTruthy();
  });

  test('Login and navigate to chat', async ({ page }) => {
    await page.goto('https://192.168.1.192:6443/login');
    await page.waitForTimeout(2000);
    
    await page.fill('input[type="text"]', 'hermes-bot');
    await page.fill('input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // Should be redirected to chat
    const url = page.url();
    const onChat = url.includes('/chat/');
    console.log(`After login, on chat page: ${onChat ? '✅' : '❌'} (${url})`);
    
    if (onChat) {
      // Check if messages are visible
      const messages = await page.locator('[data-testid="message"], .message-item').count();
      console.log(`Messages visible: ${messages}`);
    }
    
    expect(onChat).toBeTruthy();
  });
});
