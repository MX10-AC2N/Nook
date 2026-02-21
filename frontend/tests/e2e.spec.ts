import { test, expect } from '@playwright/test';

test('Login → Chat → Upload fichier', async ({ page }) => {
  // 1. Register (si besoin)
  await page.goto('/');
  await page.getByRole('link', { name: 'Créer un compte' }).click();
  await page.fill('input[name="username"]', 'e2e_test');
  await page.fill('input[name="email"]', 'e2e@test.local');
  await page.fill('input[name="password"]', 'Test123!');
  await page.getByRole('button', { name: 'Créer le compte' }).click();

  // 2. Login
  await page.goto('/');
  await page.fill('input[name="username"]', 'e2e_test');
  await page.fill('input[name="password"]', 'Test123!');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  await expect(page.getByText('Bienvenue')).toBeVisible({ timeout: 10000 });

  // 3. Créer / ouvrir une conversation
  await page.getByRole('button', { name: 'Nouvelle conversation' }).click();
  await page.getByRole('button', { name: 'Créer' }).click();

  // 4. Envoyer un message texte
  await page.fill('textarea', 'Hello from E2E test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();
  await expect(page.getByText('Hello from E2E test')).toBeVisible();

  // 5. Upload fichier
  const fileInput = page.locator('input[type="file"]').first();
  await fileInput.setInputFiles({
    name: 'test-upload.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('Ceci est un fichier uploadé via Playwright E2E !')
  });

  await expect(page.getByText('test-upload.txt')).toBeVisible({ timeout: 15000 });
  await expect(page.getByText('✅')).toBeVisible(); // ou l’indicateur de succès dans ton UI

  console.log('✅ E2E complet : Login → Chat → Upload réussi !');
});