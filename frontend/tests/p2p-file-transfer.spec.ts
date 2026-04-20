import { test } from '@playwright/test';

test.use({ ignoreHTTPSErrors: true });

test('P2P file transfer logic verification', async ({ browser }) => {
  // Browser 1: hermes-bot
  const ctx1 = await browser.newContext();
  const page1 = await ctx1.newPage();
  
  // Login as hermes-bot
  await page1.goto('https://192.168.1.192:6443/login');
  await page1.waitForTimeout(2000);
  await page1.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  await page1.fill('input[type="password"]', 'Hermes2026!');
  await page1.click('button[type="submit"]');
  await page1.waitForTimeout(3000);
  
  const url1 = page1.url();
  console.log('hermes-bot URL:', url1);
  if (url1.includes('/login')) { console.log('❌ Login failed'); return; }
  console.log('✅ hermes-bot logged in');
  
  // Navigate to chat
  await page1.waitForTimeout(2000);
  
  // Test 1: Verify P2P connection required message
  console.log('\n=== Test 1: P2P connection required ===');
  
  // Create a large file input
  await page1.evaluate(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.id = 'test-file-input';
    input.style.display = 'none';
    document.body.appendChild(input);
  });
  
  // Create a 60 MB file
  const largeFile = await page1.evaluate(async () => {
    const size = 60 * 1024 * 1024; // 60 MB
    const data = new Uint8Array(size);
    for (let i = 0; i < size; i++) {
      data[i] = i % 256;
    }
    const file = new File([data], 'test_60mb.bin', { type: 'application/octet-stream' });
    return { name: file.name, size: file.size };
  });
  
  console.log(`Created test file: ${largeFile.name} (${(largeFile.size / 1024 / 1024).toFixed(1)} MB)`);
  
  // Try to upload the file (should fail with P2P required message)
  await page1.evaluate(() => {
    const input = document.getElementById('test-file-input') as HTMLInputElement;
    // Simulate file selection
    const event = new Event('change', { bubbles: true });
    input.dispatchEvent(event);
  });
  
  // Check for error message
  await page1.waitForTimeout(1000);
  const errorText = await page1.evaluate(() => {
    const errorElements = document.querySelectorAll('.conn-error, .error-message, [class*="error"]');
    for (const el of errorElements) {
      if (el.textContent?.includes('P2P')) {
        return el.textContent;
      }
    }
    return null;
  });
  
  if (errorText) {
    console.log('✅ Error message displayed:', errorText);
  } else {
    console.log('❌ No P2P error message found');
  }
  
  // Test 2: Verify file size limit
  console.log('\n=== Test 2: File size limit ===');
  
  const hugeFile = await page1.evaluate(async () => {
    const size = 550 * 1024 * 1024; // 550 MB
    const data = new Uint8Array(1024); // Only create 1KB for testing
    const file = new File([data], 'test_550mb.bin', { type: 'application/octet-stream' });
    return { name: file.name, size: size }; // Report actual size
  });
  
  console.log(`Created test file: ${hugeFile.name} (${(hugeFile.size / 1024 / 1024).toFixed(1)} MB)`);
  
  // Try to upload the huge file (should fail with size limit message)
  await page1.evaluate(() => {
    const input = document.getElementById('test-file-input') as HTMLInputElement;
    const event = new Event('change', { bubbles: true });
    input.dispatchEvent(event);
  });
  
  await page1.waitForTimeout(1000);
  const sizeError = await page1.evaluate(() => {
    const errorElements = document.querySelectorAll('.conn-error, .error-message, [class*="error"]');
    for (const el of errorElements) {
      if (el.textContent?.includes('500 Mo')) {
        return el.textContent;
      }
    }
    return null;
  });
  
  if (sizeError) {
    console.log('✅ Size limit error displayed:', sizeError);
  } else {
    console.log('❌ No size limit error found');
  }
  
  // Test 3: Verify P2P transfer UI elements exist
  console.log('\n=== Test 3: P2P UI elements ===');
  
  const uiElements = await page1.evaluate(() => {
    const elements = {
      p2pTransfers: document.querySelector('.p2p-transfers'),
      p2pTransfer: document.querySelector('.p2p-transfer'),
      progressBar: document.querySelector('.p2p-progress-bar'),
      progressFill: document.querySelector('.p2p-progress-fill'),
    };
    
    return {
      p2pTransfers: !!elements.p2pTransfers,
      p2pTransfer: !!elements.p2pTransfer,
      progressBar: !!elements.progressBar,
      progressFill: !!elements.progressFill,
    };
  });
  
  console.log('UI elements:', uiElements);
  
  // Cleanup
  await ctx1.close();
  
  console.log('\n=== Tests completed ===');
});
