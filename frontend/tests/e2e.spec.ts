import { test, expect } from '@playwright/test';

// En CI le serveur tourne sur localhost:6300 via Docker
// Compte admin créé automatiquement au démarrage (voir check_initial_admin)
// Credentials : admin / changeme2026

test('Login → Chat → Upload fichier', async ({ page }) => {

  // 1. Login avec le compte admin (créé automatiquement au 1er démarrage)
  await page.goto('/');

  await page.fill('input[name="username"]', 'admin');
  await page.fill('input[name="password"]', 'changeme2026');
  await page.getByRole('button', { name: 'Se connecter' }).click();

  // Attendre la redirection post-login
  await expect(page).toHaveURL(/\/(home|dashboard|conversations|chat)?/, { timeout: 10000 });

  // 2. Créer une conversation
  await page.getByRole('button', { name: 'Nouvelle conversation' }).click();
  await page.getByRole('button', { name: 'Créer' }).click();

  // 3. Envoyer un message texte
  await page.fill('textarea', 'Hello from E2E test !');
  await page.getByRole('button', { name: 'Envoyer' }).click();
  await expect(page.getByText('Hello from E2E test')).toBeVisible();

  // 4. Upload fichier
  const fileInput = page.locator('input[type="file"]').first();
  await fileInput.setInputFiles({
    name: 'test-upload.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('Ceci est un fichier uploadé via Playwright E2E !')
  });

  await expect(page.getByText('test-upload.txt')).toBeVisible({ timeout: 15000 });

  console.log('✅ E2E complet : Login → Chat → Upload réussi !');
});
