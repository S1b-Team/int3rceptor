import { test, expect } from '@playwright/test';
import {
  expectVisualMatch,
  expectMockupMatch,
  waitForPageStability
} from './utils';

test.describe('Repeater/REST Tab Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/repeater');
    await waitForPageStability(page);
  });

  test('should match baseline screenshot', async ({ page }, testInfo) => {
    await expectVisualMatch(page, testInfo, 'repeater-baseline.png');
  });

  test('should match approved mockup', async ({ page }, testInfo) => {
    await expectMockupMatch(page, testInfo, 'repeater');
  });

  test('should capture full page screenshot', async ({ page }, testInfo) => {
    const screenshot = await page.screenshot({ fullPage: true });
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });

  test('should have correct layout structure', async ({ page }) => {
    // Check for main repeater elements
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();

    // Check for repeater-specific elements
    const repeaterContent = page.locator('[data-testid="repeater"], .repeater, #repeater');
    if (await repeaterContent.count() > 0) {
      await expect(repeaterContent.first()).toBeVisible();
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

      const filename = `repeater-${viewport.width}x${viewport.height}.png`;
      await expectVisualMatch(page, testInfo, filename);
    }
  });
});
