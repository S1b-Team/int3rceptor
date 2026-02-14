import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for INT3RCEPTOR UI testing
 * Supports visual regression testing, cross-browser testing, and accessibility testing
 * Configured for Vue 3 + Vite
 */
export default defineConfig({
  testDir: process.env.PLAYWRIGHT_TEST_DIR || './tests/visual',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list']
  ],
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 10 * 1000,
    navigationTimeout: 30 * 1000,
  },

  projects: [
    // Visual regression tests - Chrome only
    {
      name: 'visual-chrome',
      testDir: './tests/visual',
      use: {
        ...devices['Desktop Chrome'],
        screenshot: 'only-on-failure',
      },
    },

    // Cross-browser testing - Chrome
    {
      name: 'cross-browser-chrome',
      testDir: './tests/cross-browser',
      use: {
        ...devices['Desktop Chrome'],
        channel: 'chrome',
      },
    },

    // Cross-browser testing - Firefox
    {
      name: 'cross-browser-firefox',
      testDir: './tests/cross-browser',
      use: {
        ...devices['Desktop Firefox'],
        channel: 'firefox',
      },
    },

    // Cross-browser testing - Edge
    {
      name: 'cross-browser-edge',
      testDir: './tests/cross-browser',
      use: {
        ...devices['Desktop Edge'],
        channel: 'msedge',
      },
    },

    // Cross-browser testing - Safari (WebKit)
    {
      name: 'cross-browser-safari',
      testDir: './tests/cross-browser',
      use: {
        ...devices['Desktop Safari'],
        channel: 'webkit',
      },
    },

    // Accessibility testing - Chrome with axe-core
    {
      name: 'accessibility-chrome',
      testDir: './tests/accessibility',
      use: {
        ...devices['Desktop Chrome'],
        channel: 'chrome',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',
      },
      dependencies: [],
    },
  ],

  // Performance testing - Chrome with performance metrics
  {
    name: 'performance-chrome',
    testDir: './tests/performance',
    use: {
      ...devices['Desktop Chrome'],
      channel: 'chrome',
      screenshot: 'only-on-failure',
      video: 'retain-on-failure',
    },
  },

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
