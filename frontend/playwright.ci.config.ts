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
    baseURL: 'http://192.168.1.192:6300',
    trace: 'on-first-retry',
  },

  reporter: [
    ['list'],
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
      testMatch: ['**/admin.spec.ts', '**/admin-ui.spec.ts'],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
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
    {
      name: 'call-ui',
      testMatch: ['**/call-ui.spec.ts', '**/call.spec.ts', '**/call-debug.spec.ts', '**/call-signaling.spec.ts', '**/call-test.spec.ts'],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['api-sanity'],
    },
  ],
});
