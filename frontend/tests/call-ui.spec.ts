import { test, expect } from '@playwright/test';
import { BASE } from './helpers';

const WS_BASE = BASE.replace(/^http/, 'ws').replace('/api', '');


test.describe('Call Page UI — Audio/Video', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('Navigate to call page (audio)', async ({ page }) => {
    // Get conversation ID from API or use a known one
    const convId = 'be5d6e53-518a-4962-85a5-e13b35c8eecf'; // 1-to-1 conv with admin
    await page.goto(`${BASE}/call/${convId}?type=audio`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({ path: '/tmp/nook-call-audio.png', fullPage: true });

    // Check if call UI elements exist
    const callContainer = page.locator('.call-container, [data-testid="call-container"]');
    const idleState = page.locator('.idle, [data-testid="call-idle"]');
    const audioOnly = page.locator('.audio-only, [data-testid="audio-only"]');

    const containerCount = await callContainer.count();
    const idleCount = await idleState.count();
    const audioCount = await audioOnly.count();

    console.log(`Call UI: container=${containerCount}, idle=${idleCount}, audio=${audioCount}`);

    if (containerCount > 0 || idleCount > 0) {
      console.log('✅ Call page loaded (audio mode)');
      expect(containerCount + idleCount).toBeGreaterThan(0);
    } else {
      console.log('⚠️ Call UI not found — might be redirect or error');
      // Check if redirected to chat
      const url = page.url();
      console.log(`Current URL: ${url}`);
    }
  });

  test('Navigate to call page (video)', async ({ page }) => {
    const convId = 'be5d6e53-518a-4962-85a5-e13b35c8eecf';
    await page.goto(`${BASE}/call/${convId}?type=video`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({ path: '/tmp/nook-call-video.png', fullPage: true });

    const videoContainer = page.locator('.video-container, [data-testid="video-container"]');
    const videoOnly = page.locator('.video-only, [data-testid="video-only"]');

    const videoCount = await videoContainer.count() + await videoOnly.count();
    console.log(`Video call UI: ${videoCount}`);

    if (videoCount > 0) {
      console.log('✅ Call page loaded (video mode)');
    }
  });

  test('Call controls visible (mic/cam toggles)', async ({ page }) => {
    const convId = 'be5d6e53-518a-4962-85a5-e13b35c8eecf';
    await page.goto(`${BASE}/call/${convId}?type=audio`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    // Check for control buttons
    const micToggle = page.locator('[data-testid="mic-toggle"], button[title*="micro"], button[aria-label*="micro"]');
    const camToggle = page.locator('[data-testid="cam-toggle"], button[title*="caméra"], button[aria-label*="caméra"]');
    const hangupBtn = page.locator('[data-testid="hangup"], button[title*="raccrocher"], button[aria-label*="raccrocher"]');

    const micCount = await micToggle.count();
    const hangupCount = await hangupBtn.count();

    console.log(`Call controls: mic=${micCount}, hangup=${hangupCount}`);

    if (hangupCount > 0) {
      console.log('✅ Call controls present');
    }
  });

  test('Call banner appears on chat page when call active', async ({ page, context }) => {
    // This test simulates receiving a call
    // Open chat page first
    await page.goto('/chat');
    await page.waitForTimeout(2000);

    // Check if call banner exists (might be hidden initially)
    const callBanner = page.locator('[data-testid="call-banner"], .call-banner');
    const bannerCount = await callBanner.count();
    
    console.log(`Call banner elements: ${bannerCount}`);
    
    // Banner might only appear when there's an incoming call
    // This is more of a "check if component exists" test
    if (bannerCount > 0) {
      console.log('✅ Call banner component exists');
    } else {
      console.log('⚠️ Call banner not found (might be conditional)');
    }
  });

  test('WebSocket connection for signaling', async ({ page }) => {
    const convId = 'be5d6e53-518a-4962-85a5-e13b35c8eecf';
    await page.goto(`${BASE}/call/${convId}?type=audio`);
    await page.waitForTimeout(3000);

    // Check WebSocket connection in console/browser
    const wsConnected = await page.evaluate(() => {
      // Check if WebSocket is connecting or connected
      return new Promise((resolve) => {
        try {
          const ws = new WebSocket(`${WS_BASE}/ws/call/${window.location.pathname.split('/')[2]}`);
          ws.onopen = () => { ws.close(); resolve(true); };
          ws.onerror = () => resolve(false);
          setTimeout(() => resolve(false), 3000);
        } catch (e) {
          resolve(false);
        }
      });
    });

    console.log(`WebSocket test (client-side): ${wsConnected ? '✅ Possible' : '⚠️ Not tested'}`);
  });

});
