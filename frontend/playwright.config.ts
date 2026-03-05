import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  expect: { timeout: 5000 },
  // ⚠️  fullyParallel:false obligatoire avec workers:1 pour éviter que les
  // tests partagent le même browser context (et donc le même localStorage).
  // En mode parallel=true + workers=1, Playwright réutilise le context par
  // défaut → localStorage pollué entre tests → #username timeout.
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:6300',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:6300',
    // En CI : le serveur tourne déjà dans Docker → on le réutilise
    // En local : pas de serveur → Playwright démarre npm run dev
    reuseExistingServer: !!process.env.CI,
  },
});
