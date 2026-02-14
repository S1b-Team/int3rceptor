import { test, expect } from '@playwright/test';
import {
  expectVisualMatch,
  waitForPageStability
} from './utils';

test.describe('Decoder Tab Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/decoder');
    await waitForPageStability(page);
  });

  test('should match baseline screenshot', async ({ page }, testInfo) => {
    await expectVisualMatch(page, testInfo, 'decoder-baseline.png');
  });

  test('should capture full page screenshot', async ({ page }, testInfo) => {
    const screenshot = await page.screenshot({ fullPage: true });
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });

  test('should have correct layout structure', async ({ page }) => {
    // Check for main decoder elements
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();

    // Check for decoder-specific elements
    const decoderContent = page.locator('[data-testid="decoder"], .decoder, #decoder');
    if (await decoderContent.count() > 0) {
      await expect(decoderContent.first()).toBeVisible();
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

      const filename = `decoder-${viewport.width}x${viewport.height}.png`;
      await expectVisualMatch(page, testInfo, filename);
    }
  });
});
