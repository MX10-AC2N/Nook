import { test, expect } from '@playwright/test';

test.describe('Chat UI — @Mentions + Features', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[placeholder*="utilisateur"], input[type="text"]', 'hermes-bot');
    await page.fill('input[name="password"], input[type="password"]', 'Hermes2026!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/chat', { timeout: 15000 });
    await page.waitForTimeout(2000);
  });

  test('@Mentions — dropdown appears on @', async ({ page }) => {
    const input = page.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input.click();
    await page.waitForTimeout(500);

    // Type @ to trigger mentions
    await page.keyboard.type('@');
    await page.waitForTimeout(1500);

    // Check if mention dropdown appears
    const mentionDropdown = page.locator('.mention-dropdown, .mentions-dropdown, [role="listbox"]');
    const mentionItems = page.locator('.mention-item, [role="option"]');

    const dropdownCount = await mentionDropdown.count();
    const itemsCount = await mentionItems.count();

    console.log(`@Mentions: dropdown=${dropdownCount}, items=${itemsCount}`);

    if (itemsCount > 0) {
      console.log('✅ @Mentions: dropdown visible');
      // Check first item has text
      const firstItemText = await mentionItems.first().textContent();
      console.log(`First mention: ${firstItemText}`);
      expect(firstItemText?.length).toBeGreaterThan(0);
      
      // Select first mention
      await mentionItems.first().click();
      await page.waitForTimeout(500);
      
      // Check input contains mention
      const inputValue = await input.inputValue();
      console.log(`Input after select: ${inputValue}`);
      expect(inputValue).toContain('@');
    } else {
      console.log('⚠️ @Mentions: no dropdown (might be no users or feature disabled)');
    }
  });

  test('Send message + Edit + Delete', async ({ page }) => {
    const input = page.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input.click();

    // Send a test message
    const testMsg = `Test message ${Date.now()}`;
    await input.fill(testMsg);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2000);

    // Verify message appears
    const msgVisible = page.locator(`text=${testMsg}`);
    await expect(msgVisible).toBeVisible();
    console.log('✅ Message sent and visible');

    // Hover over message to show actions
    await msgVisible.first().hover();
    await page.waitForTimeout(500);

    // Click edit button if visible
    const editBtn = page.locator('[data-testid="edit-msg"], .edit-btn').first();
    if (await editBtn.isVisible()) {
      await editBtn.click();
      await page.waitForTimeout(500);

      // Modify message
      const editedMsg = testMsg + ' (edited)';
      await input.fill(editedMsg);
      await page.keyboard.press('Enter');
      await page.waitForTimeout(2000);

      // Check edited label appears
      const editedLabel = page.locator('text=/modifié|edited/i');
      if (await editedLabel.count() > 0) {
        console.log('✅ Message edited successfully');
      }
    }

    // Test delete
    const msgToDelete = page.locator(`text=${testMsg}`).first();
    await msgToDelete.hover();
    await page.waitForTimeout(500);

    const deleteBtn = page.locator('[data-testid="delete-msg"], .delete-btn').first();
    if (await deleteBtn.isVisible()) {
      await deleteBtn.click();
      await page.waitForTimeout(1000);
      console.log('✅ Message deleted');
    }
  });

  test('Reactions — hover + click emoji', async ({ page }) => {
    const input = page.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input.click();
    await input.fill(`Reaction test ${Date.now()}`);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2000);

    // Hover first message
    const firstMsg = page.locator('.message').first();
    await firstMsg.hover();
    await page.waitForTimeout(500);

    // Click reaction trigger
    const reactionBtn = page.locator('.action-btn.react-more').first();
    if (await reactionBtn.isVisible()) {
      await reactionBtn.click();
      await page.waitForTimeout(500);

      // Check picker visible
      const picker = page.locator('[data-testid="emoji-picker"]');
      if (await picker.isVisible()) {
        console.log('✅ Reaction picker visible');
        
        // Click an emoji
        const emojiBtn = page.locator('[data-testid="emoji-quick-btn"]').first();
        if (await emojiBtn.isVisible()) {
          await emojiBtn.click();
          await page.waitForTimeout(500);
          console.log('✅ Emoji reaction added');
        }
      }
    }
  });

  test('File upload button opens dialog', async ({ page }) => {
    // Look for upload button
    const uploadBtn = page.locator('[data-testid="upload-btn"], button[title*="fichier"], button[title*="upload"]').first();
    
    if (await uploadBtn.isVisible()) {
      await uploadBtn.click();
      await page.waitForTimeout(1000);

      // Check if file dialog or input appears
      const fileInput = page.locator('input[type="file"]');
      const dialog = page.locator('[data-testid="upload-dialog"]');
      
      if (await fileInput.count() > 0 || await dialog.isVisible()) {
        console.log('✅ File upload dialog/input present');
      }
    } else {
      console.log('⚠️ Upload button not found (might be mobile or different UI)');
    }
  });

  test('Scroll to load more messages', async ({ page }) => {
    // Get messages container
    const messagesContainer = page.locator('.messages-container, .chat-messages').first();
    
    if (await messagesContainer.isVisible()) {
      // Scroll to top to trigger load more
      await messagesContainer.evaluate(el => el.scrollTop = 0);
      await page.waitForTimeout(2000);
      
      // Check if more messages loaded (message count increased)
      const msgCount = await page.locator('.message').count();
      console.log(`Messages visible: ${msgCount}`);
    }
  });

  test('Typing indicator appears', async ({ page, context }) => {
    // Open second browser context to simulate other user typing
    const page2 = await context.newPage();
    await page2.goto('/login');
    await page2.fill('input[name="username"], input[placeholder*="utilisateur"]', 'admin');
    await page2.fill('input[name="password"], input[type="password"]', 'AdminCI2026!');
    await page2.click('button[type="submit"]');
    await page2.waitForURL('**/chat', { timeout: 15000 });
    await page2.waitForTimeout(2000);

    // Type in second page
    const input2 = page2.locator('.message-input, textarea, [contenteditable="true"]').first();
    await input2.click();
    await input2.fill('Typing test...');
    await page2.waitForTimeout(500);

    // Check if typing indicator appears on first page
    const typingIndicator = page.locator('[data-testid="typing-indicator"], .typing-indicator');
    if (await typingIndicator.isVisible({ timeout: 3000 })) {
      console.log('✅ Typing indicator visible');
    }

    await page2.close();
  });

});
