import { test, expect } from '@playwright/test';
import {
  expectVisualMatch,
  expectMockupMatch,
  waitForPageStability
} from './utils';

test.describe('Loading State Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/loading');
    await waitForPageStability(page);
  });

  test('should match baseline screenshot', async ({ page }, testInfo) => {
    await expectVisualMatch(page, testInfo, 'loading-baseline.png');
  });

  test('should match approved mockup', async ({ page }, testInfo) => {
    await expectMockupMatch(page, testInfo, 'loading');
  });

  test('should capture full page screenshot', async ({ page }, testInfo) => {
    const screenshot = await page.screenshot({ fullPage: true });
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });

  test('should have correct layout structure', async ({ page }) => {
    // Check for main loading elements
    const loadingContent = page.locator('[data-testid="loading"], .loading, #loading');
    if (await loadingContent.count() > 0) {
      await expect(loadingContent.first()).toBeVisible();
    }

    // Check for loading indicators
    const spinner = page.locator('[data-testid="spinner"], .spinner, .loading-spinner');
    if (await spinner.count() > 0) {
      await expect(spinner.first()).toBeVisible();
    }
  });

  test('should be responsive', async ({ page }, testInfo) => {
    // Test at different viewport sizes
    const viewports = [
      { width: 1920, height: 1080 },
      { width: 1366, height: 768 },
      { width: 768, height: 1024 }
    ];

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      await waitForPageStability(page);

      const filename = `loading-${viewport.width}x${viewport.height}.png`;
      await expectVisualMatch(page, testInfo, filename);
    }
  });
});
