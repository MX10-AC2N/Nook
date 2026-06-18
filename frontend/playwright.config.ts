// frontend/playwright.config.ts
// Session 38 — refonte tests : inclut TOUS les fichiers .spec.ts

import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60000,
  expect: { timeout: 8000 },
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  
  use: {
    baseURL: process.env.NOOK_BASE_URL || 'http://localhost:6300',
    trace: 'on-first-retry',
  },

  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: '/tmp/playwright-results.json' }],
  ],

  projects: [
    // 1. API Sanity (401 checks, no login needed)
    {
      name: 'api-sanity',
      testMatch: '**/api-sanity.spec.ts',
      use: { ...devices['Desktop Chrome'] },
    },

    // 2. Admin Flow (needs admin login)
    {
      name: 'admin-flow',
      testMatch: ['**/admin.spec.ts', '**/admin-ui.spec.ts'],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },

    // 3. User Flow (needs user login)
    {
      name: 'user-flow',
      testMatch: [
        '**/user.spec.ts',
        '**/chat-ui.spec.ts',
        '**/chat-ui-advanced.spec.ts',
        '**/e2ee-chat.spec.ts',
        '**/polls-ui.spec.ts',
        '**/events-ui.spec.ts',
        '**/push-test.spec.ts',
        '**/push-notifications.spec.ts',
      ],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },

    // 4. Call UI (needs user login)
    {
      name: 'call-ui',
      testMatch: ['**/call-ui.spec.ts', '**/p2p-file-transfer.spec.ts'],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
  ],
});
