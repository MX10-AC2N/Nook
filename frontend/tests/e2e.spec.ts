import { test, expect, request as apiRequest } from '@playwright/test';

// ─────────────────────────────────────────────────────────────────────────────
// Helpers API (hors browser)
// ─────────────────────────────────────────────────────────────────────────────
const BASE = 'http://localhost:6300';

async function apiLogin(username: string, password: string) {
  const ctx = await apiRequest.newContext({ baseURL: BASE });
  const res = await ctx.post('/api/auth/login', {
    data: { username, password },
  });
  const cookies = res.headers()['set-cookie'] ?? '';
  const match = cookies.match(/auth_token=([^;]+)/);
  await ctx.dispose();
  if (!res.ok()) throw new Error(`Login ${username} failed: ${res.status()}`);
  return match?.[1] ?? '';
}

async function approveUser(adminCookie: string, userId: string) {
  const ctx = await apiRequest.newContext({
    baseURL: BASE,
    extraHTTPHeaders: { Cookie: `auth_token=${adminCookie}` },
  });
  await ctx.post('/api/approve', { data: { user_id: userId } });
  await ctx.dispose();
}

async function getPendingUserId(adminCookie: string, username: string): Promise<string> {
  const ctx = await apiRequest.newContext({
    baseURL: BASE,
    extraHTTPHeaders: { Cookie: `auth_token=${adminCookie}` },
  });
  const res = await ctx.get('/api/pending-users-json');
  const body = await res.json();
  await ctx.dispose();
  const user = body.users?.find((u: any) => u.username === username);
  if (!user) throw new Error(`User ${username} not found in pending list`);
  return user.id;
}

// ─────────────────────────────────────────────────────────────────────────────
// Setup : créer + approuver un user de test via API
// (sans passer par le flow admin UI ni needs_password_change)
// ─────────────────────────────────────────────────────────────────────────────
test.beforeAll(async () => {
  const ctx = await apiRequest.newContext({ baseURL: BASE });

  // Register le user de test (idempotent : ignore le conflit si déjà créé)
  await ctx.post('/api/auth/register', {
    data: {
      username: 'e2e_ci',
      password: 'E2eTest123!',
      email: 'e2e@ci.nook',
      name: 'E2E CI',
    },
  });
  await ctx.dispose();

  // Login admin (approved=1, require_auth passe même avec needs_password_change)
  const adminCookie = await apiLogin('admin', 'changeme2026');

  // Récupérer l'id du user e2e_ci dans la liste des pending
  const userId = await getPendingUserId(adminCookie, 'e2e_ci');

  // Approuver le user
  await approveUser(adminCookie, userId);
});

// ─────────────────────────────────────────────────────────────────────────────
// Test principal
// ─────────────────────────────────────────────────────────────────────────────
test('Login → Chat → Envoi message', async ({ page }) => {

  // 1. Login via l'UI
  await page.goto('/login');

  // Les inputs ont id= (pas name=) dans le template Svelte
  await page.fill('#username', 'e2e_ci');
  await page.fill('#password', 'E2eTest123!');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // 2. Vérifier qu'on arrive sur /chat (e2e_ci n'a pas needs_password_change)
  await expect(page).toHaveURL(/\/chat/, { timeout: 10_000 });

  // 3. Envoyer un message dans le Groupe Global
  const textarea = page.locator('textarea[placeholder="Envoyer un message..."]');
  await expect(textarea).toBeVisible({ timeout: 5000 });
  await textarea.fill('Hello from E2E CI test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();

  // 4. Vérifier que le message apparaît dans le chat
  await expect(page.getByText('Hello from E2E CI test !')).toBeVisible({ timeout: 5000 });

  console.log('✅ E2E complet : Login → Chat → Message envoyé !');
});
