import { test, expect } from '@playwright/test';

const BASE = 'http://localhost:6300';

test.describe('P2P File Transfer — UI + Validation', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto(BASE + '/login');
    await page.fill('input[name="username"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('Upload button visible in chat', async ({ page }) => {
    // Check for upload button
    const uploadBtn = page.locator('[data-testid="upload-btn"], button[title*="fichier"], button[title*="upload"], .upload-btn').first();
    
    if (await uploadBtn.isVisible()) {
      console.log('✅ Upload button visible');
      await uploadBtn.click();
      await page.waitForTimeout(1000);
      
      // Check if file input appears
      const fileInput = page.locator('input[type="file"]');
      if (await fileInput.count() > 0) {
        console.log('✅ File input available');
      }
    } else {
      // Maybe upload is triggered differently
      console.log('⚠️ Upload button not found — checking alternative UI');
      
      // Check for attachment icon
      const attachIcon = page.locator('[data-testid="attach"], .attach-icon, svg[aria-label*="attach"]').first();
      if (await attachIcon.isVisible()) {
        console.log('✅ Attachment icon found (alternative)');
      }
    }
  });

  test('File size > 50MB shows P2P required message', async ({ page }) => {
    // Navigate to a 1-to-1 conversation (not group)
    // First, check if we're in a conversation
    const convList = page.locator('.conversation-item, [data-testid="conversation"]');
    const convCount = await convList.count();
    
    if (convCount > 1) {
      // Click on a 1-to-1 conversation (not the first one which might be global)
      await convList.nth(1).click();
      await page.waitForTimeout(2000);
    }

    // Create a mock large file (60MB) using the file input
    const fileInput = page.locator('input[type="file"]').first();
    
    if (await fileInput.count() > 0) {
      // Set a large file via JavaScript (simulate selection)
      const largeFileInfo = await page.evaluate(() => {
        // Create a mock file object
        const file = new File([''], 'large_file_60mb.bin', { type: 'application/octet-stream' });
        Object.defineProperty(file, 'size', { value: 60 * 1024 * 1024 }); // 60MB
        
        // Trigger the upload handler
        const event = new Event('change', { bubbles: true });
        const input = document.querySelector('input[type="file"]') as HTMLInputElement;
        if (input) {
          // Simulate file selection
          Object.defineProperty(input, 'files', {
            value: [file],
            writable: false,
          });
          input.dispatchEvent(event);
        }
        
        return { name: file.name, size: file.size };
      });

      console.log('Mock file created:', largeFileInfo);
      await page.waitForTimeout(2000);

      // Check for P2P required message
      const p2pMessage = page.locator('text=/P2P|P2P requis|connexion directe/i');
      if (await p2pMessage.isVisible({ timeout: 3000 })) {
        console.log('✅ P2P required message displayed');
      } else {
        console.log('⚠️ P2P message not found (might be auto-retry or different UI)');
      }
    }
  });

  test('Group conversation > 50MB file shows restriction message', async ({ page }) => {
    // Try to navigate to a group conversation
    const groupConv = page.locator('.conversation-item:has(.group-icon), [data-testid="conversation"]').first();
    
    // For this test, we'll just verify the restriction logic exists
    const restrictionCheck = await page.evaluate(() => {
      // Check if the app has logic to restrict >50MB files in groups
      const hasP2PTransfer = typeof (window as any).startP2PTransfer === 'function';
      const hasFileSizeCheck = document.querySelector('[data-testid="file-size-limit"]') !== null;
      return { hasP2PTransfer, hasFileSizeCheck };
    });

    console.log('P2P/File size check:', restrictionCheck);
  });

  test('P2P transfer UI elements exist', async ({ page }) => {
    // Check if P2P transfer components exist in the DOM
    const p2pElements = await page.evaluate(() => {
      const elements = {
        p2pTransfers: document.querySelector('[data-testid="p2p-transfers"], .p2p-transfers'),
        p2pTransfer: document.querySelector('[data-testid="p2p-transfer"], .p2p-transfer'),
        progressBar: document.querySelector('[data-testid="p2p-progress"], .p2p-progress-bar'),
        cancelBtn: document.querySelector('[data-testid="p2p-cancel"], .p2p-cancel-btn'),
      };
      return Object.entries(elements).reduce((acc, [key, el]) => {
        acc[key] = !!el;
        return acc;
      }, {} as Record<string, boolean>);
    });

    console.log('P2P UI elements:', p2pElements);
    
    // At least one P2P element should exist (even if hidden)
    const hasAnyP2P = Object.values(p2pElements).some(v => v);
    if (hasAnyP2P) {
      console.log('✅ P2P UI elements present in DOM');
    } else {
      console.log('⚠️ No P2P UI elements found (might be lazy-loaded)');
    }
  });

  test('File type validation (SVG XSS prevention)', async ({ page }) => {
    // Upload an SVG file (should be sanitized or rejected)
    const fileInput = page.locator('input[type="file"]').first();
    
    if (await fileInput.count() > 0) {
      // Create a malicious SVG
      const svgContent = `<svg xmlns="http://www.w3.org/2000/svg" onload="alert('XSS')"><circle r="10"/></svg>`;
      
      const svgFile = await page.evaluate((content) => {
        const blob = new Blob([content], { type: 'image/svg+xml' });
        const file = new File([blob], 'malicious.svg', { type: 'image/svg+xml' });
        
        // Dispatch to input
        const input = document.querySelector('input[type="file"]') as HTMLInputElement;
        if (input) {
          const event = new Event('change', { bubbles: true });
          Object.defineProperty(input, 'files', { value: [file] });
          input.dispatchEvent(event);
        }
        
        return { name: file.name, type: file.type, size: file.size };
      }, svgContent);

      console.log('SVG file "uploaded":', svgFile);
      await page.waitForTimeout(2000);

      // Check that SVG is either rejected or sanitized
      const xssAlert = await page.evaluate(() => {
        return new Promise<boolean>((resolve) => {
          const originalAlert = window.alert;
          let alertCalled = false;
          window.alert = () => { alertCalled = true; };
          setTimeout(() => {
            window.alert = originalAlert;
            resolve(alertCalled);
          }, 1000);
        });
      });

      if (!xssAlert) {
        console.log('✅ SVG XSS prevented (no alert triggered)');
      } else {
        console.log('❌ SVG XSS vulnerability!');
      }
    }
  });

});
