import { test, expect } from '@playwright/test';
import {
  expectVisualMatch,
  waitForPageStability
} from './utils';

test.describe('Plugins Tab Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/plugins');
    await waitForPageStability(page);
  });

  test('should match baseline screenshot', async ({ page }, testInfo) => {
    await expectVisualMatch(page, testInfo, 'plugins-baseline.png');
  });

  test('should capture full page screenshot', async ({ page }, testInfo) => {
    const screenshot = await page.screenshot({ fullPage: true });
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });

  test('should have correct layout structure', async ({ page }) => {
    // Check for main plugins elements
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();

    // Check for plugins-specific elements
    const pluginsContent = page.locator('[data-testid="plugins"], .plugins, #plugins');
    if (await pluginsContent.count() > 0) {
      await expect(pluginsContent.first()).toBeVisible();
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

      const filename = `plugins-${viewport.width}x${viewport.height}.png`;
      await expectVisualMatch(page, testInfo, filename);
    }
  });
});
