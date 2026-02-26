import { test, expect } from '@playwright/test';

test('Login → Chat → Envoi message', async ({ page }) => {
  test.setTimeout(60_000);

  console.log('🚀 Début du test E2E...');

  // 1. Login
  await page.goto('/login');
  await page.fill('#username', 'e2e_ci');
  await page.fill('#password', 'E2eTest123!');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // 2. Redirection vers /chat
  await expect(page).toHaveURL(/\/chat/, { timeout: 15_000 });
  console.log('✅ Redirection vers /chat OK');

  // 3. Attendre que le spinner de chargement du layout disparaisse
  await expect(page.locator('.loading-screen')).not.toBeVisible({ timeout: 15_000 });
  console.log('✅ Loading screen disparu');

  // 4. Attendre que la sidebar soit visible
  await expect(
    page.locator('.conversation-item').first()
  ).toBeVisible({ timeout: 10_000 });
  console.log('✅ Sidebar conversations visible');

  await expect(
    page.locator('.conversation-info .name').first()
  ).toHaveText('Groupe Global', { timeout: 5_000 });
  console.log('✅ Groupe Global trouvé');

  // 5. Le champ de saisie
  const input = page.locator('input.message-input');
  await expect(input).toBeVisible({ timeout: 10_000 });
  console.log('✅ Input de chat visible');

  // 6. Envoyer le message
  await input.fill('Hello from E2E CI test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();
  console.log('📤 Message envoyé');

  // 7. Attendre que le message apparaisse (loadMessages est async après envoi)
  await expect(
    page.locator('.message-content').filter({ hasText: 'Hello from E2E CI test !' })
  ).toBeVisible({ timeout: 12_000 });

  console.log('🎉 E2E Test réussi : Login → Chat → Message envoyé !');
});
