import { test, expect } from '@playwright/test';

test('Login → Chat → Envoi message', async ({ page }) => {
  test.setTimeout(40_000); // Timeout global plus large pour CI

  console.log('🚀 Début du test E2E...');

  // 1. Login
  await page.goto('/login');
  await page.fill('#username', 'e2e_ci');
  await page.fill('#password', 'E2eTest123!');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // 2. Redirection vers /chat
  await expect(page).toHaveURL(/\/chat/, { timeout: 15_000 });
  console.log('✅ Redirection vers /chat OK');

  // 3. Chargement complet de la page (très important avec Svelte 5 runes)
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await page.waitForTimeout(1500); // Temps pour que les runes + state se stabilisent

  // 4. Attendre qu'au moins une conversation soit chargée (Groupe Global, etc.)
  await expect(
    page.locator('text=Groupe Global, .conversation, [data-conversation], .chat-room').first()
  ).toBeVisible({ timeout: 10_000 });

  // 5. Le champ de saisie
  const textarea = page.getByPlaceholder('Envoyer un message...');
  await expect(textarea).toBeVisible({ timeout: 12_000 });
  console.log('✅ Textarea de chat visible');

  // 6. Envoyer le message
  await textarea.fill('Hello from E2E CI test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();
  console.log('📤 Message envoyé');

  // 7. Vérifier que le message apparaît
  await expect(page.getByText('Hello from E2E CI test !')).toBeVisible({ timeout: 8_000 });

  console.log('🎉 E2E Test réussi : Login → Chat → Message envoyé !');
});