// frontend/playwright.config.ts
// Session 38 — refonte tests : 3 fichiers + helpers partagés
//
// Ordre d'exécution des projets :
//   1. api-sanity  → tests 401 sans login (rapides, ~30s)
//   2. admin-flow  → flux admin complet (beforeAll = 1 login)
//   3. user-flow   → flux user complet (beforeAll = 1 login)
//
// fullyParallel:false + workers:1 en CI :
//   Les 3 projets s'exécutent en séquence pour partager la même DB Docker.
//   Pas de race condition, pas de pollution entre suites.
//   user-flow dépend de api-sanity (serveur up) mais PAS de admin-flow :
//   une erreur admin ne doit pas skippper tous les tests user.
//

import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60_000,
  expect: { timeout: 8_000 },
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  use: {
    baseURL: 'http://localhost:6300',
    trace: 'on-first-retry',
  },
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: '/tmp/playwright-results.json' }],
  ],
  projects: [
    {
      name: 'api-sanity',
      testMatch: '**/api-sanity.spec.ts',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'admin-flow',
      testMatch: '**/admin.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
    {
      name: 'user-flow',
      testMatch: '**/user.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
    {
      name: 'chat-ui',
      testMatch: '**/chat-ui.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
  ],
});
