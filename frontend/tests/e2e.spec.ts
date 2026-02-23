import { test, expect } from '@playwright/test';

// ─────────────────────────────────────────────────────────────────────────────
// Le user e2e_ci est créé et approuvé par le step "🔧 Setup user E2E"
// dans test-nook.yml AVANT que Playwright soit lancé → pas de setup ici
// ─────────────────────────────────────────────────────────────────────────────

test('Login → Chat → Envoi message', async ({ page }) => {

  // 1. Login via l'UI — inputs ont id= (pas name=) dans le template Svelte
  await page.goto('/login');
  await page.fill('#username', 'e2e_ci');
  await page.fill('#password', 'E2eTest123!');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // 2. Vérifier l'arrivée sur /chat (e2e_ci n'a pas needs_password_change)
  await expect(page).toHaveURL(/\/chat/, { timeout: 10_000 });

  // 3. Envoyer un message dans le Groupe Global
  const textarea = page.locator('textarea[placeholder="Envoyer un message..."]');
  await expect(textarea).toBeVisible({ timeout: 5_000 });
  await textarea.fill('Hello from E2E CI test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();

  // 4. Vérifier que le message apparaît
  await expect(page.getByText('Hello from E2E CI test !')).toBeVisible({ timeout: 5_000 });

  console.log('✅ E2E complet : Login → Chat → Message envoyé !');
});
