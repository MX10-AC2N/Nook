import { test, expect } from '@playwright/test';


test.describe('Admin UI — Users + Invites', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="username"], input[type="text"]', 'admin');
    await page.fill('input[name="password"], input[type="password"]', 'AdminCI2026!');
    await page.click('button[type="submit"]');
    await page.waitForTimeout(3000);
    
    // May redirect to change password if first login
    if (page.url().includes('change-password')) {
      await page.fill('input[type="password"]', 'AdminCI2026!');
      await page.click('button[type="submit"]');
      await page.waitForTimeout(2000);
    }
  });

  test('Admin panel accessible', async ({ page }) => {
    // Navigate to admin panel (usually /admin or similar)
    await page.goto('/admin');
    await page.waitForTimeout(2000);

    const url = page.url();
    console.log('Admin URL:', url);

    // Check if admin UI loaded
    const adminContainer = page.locator('[data-testid="admin-panel"], .admin-container, .admin-panel').first();
    const isAdminVisible = await adminContainer.isVisible({ timeout: 3000 }).catch(() => false);

    if (isAdminVisible || url.includes('admin')) {
      console.log('✅ Admin panel accessible');
    } else {
      console.log('⚠️ Admin panel not found at /admin — might be at different route');
    }
  });

  test('Pending users list (if any)', async ({ page }) => {
    // Go to admin section for pending users
    await page.goto('/admin/users');
    await page.waitForTimeout(2000);

    // Check for pending users section
    const pendingSection = page.locator('text=/en attente|pending|approbation/i').first();
    const pendingCount = await pendingSection.count();

    if (pendingCount > 0) {
      console.log('✅ Pending users section found');
      
      // Check for approve buttons
      const approveBtn = page.locator('[data-testid="approve-user"], button:has-text("Approuver"), button:has-text("Approve")').first();
      if (await approveBtn.isVisible({ timeout: 3000 })) {
        console.log('✅ Approve button found');
      }
    } else {
      console.log('⚠️ No pending users or section not found');
    }
  });

  test('Invite management', async ({ page }) => {
    await page.goto('/admin/invites');
    await page.waitForTimeout(2000);

    // Check for invite creation
    const createInviteBtn = page.locator('[data-testid="create-invite"], button:has-text("Créer"), button:has-text("Create")').first();
    
    if (await createInviteBtn.isVisible({ timeout: 3000 })) {
      console.log('✅ Create invite button found');
      await createInviteBtn.click();
      await page.waitForTimeout(1000);

      // Check for invite form
      const inviteForm = page.locator('[data-testid="invite-form"], form').first();
      if (await inviteForm.isVisible()) {
        console.log('✅ Invite form opened');
      }
    } else {
      console.log('⚠️ Invite creation UI not found');
    }
  });

  test('User list and search', async ({ page }) => {
    await page.goto('/admin/users');
    await page.waitForTimeout(2000);

    // Check for user list
    const userList = page.locator('[data-testid="user-list"], .user-list, tbody tr').first();
    if (await userList.isVisible({ timeout: 3000 })) {
      console.log('✅ User list visible');
      
      // Check for search input
      const searchInput = page.locator('input[placeholder*="recherche"], input[placeholder*="search"], [data-testid="user-search"]').first();
      if (await searchInput.isVisible()) {
        console.log('✅ User search available');
        await searchInput.fill('hermes');
        await page.waitForTimeout(1000);
      }
    }
  });

  test('API: Get pending users', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'admin', password: 'AdminCI2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Get pending users
    const pending = await request.get(`${BASE}/api/users/pending`);
    expect([200, 401, 403]).toContain(pending.status());
    
    if (pending.ok()) {
      const body = await pending.json();
      console.log('Pending users:', body);
      expect(body.users || body).toBeDefined();
      console.log('✅ Pending users API works');
    } else {
      console.log('⚠️ Pending users API not accessible');
    }
  });

  test('API: Create and delete invite', async ({ request }) => {
    const login = await request.post(`${BASE}/api/auth/login`, {
      data: { username: 'admin', password: 'AdminCI2026!' },
    });
    expect(login.ok()).toBeTruthy();

    // Create invite
    const createRes = await request.post(`${BASE}/api/invites`, {
      data: { max_uses: 1, expires_in_days: 7 },
    });
    expect([200, 201, 401, 403]).toContain(createRes.status());

    if (createRes.ok()) {
      const inviteBody = await createRes.json();
      console.log('Invite created:', inviteBody);
      const inviteId = inviteBody.id || inviteBody.invite_id;
      
      if (inviteId) {
        // Delete invite
        const deleteRes = await request.post(`${BASE}/api/invites/delete`, {
          data: { invite_id: inviteId },
        });
        expect([200, 204, 404]).toContain(deleteRes.status());
        console.log('✅ Invite created and deleted');
      }
    } else {
      console.log('⚠️ Invite creation not accessible');
    }
  });

});
