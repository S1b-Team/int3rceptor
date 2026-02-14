import { test, expect } from '@playwright/test';
import {
  runComprehensiveA11yTest,
  viewSelectors,
  navSelectors,
  navigateToView,
  WCAG_2_1_AA_TAGS,
  runAxeScan,
  testKeyboardNavigation,
  testFocusIndicators,
  testARIALabels,
  testContrastRatios
} from './utils';

/**
 * Accessibility tests for Scanner view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Scanner - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'scanner', navSelectors.scanner, viewSelectors.scanner);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.scanner);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.scanner);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.scanner);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.scanner);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible scan options', async ({ page }) => {
    const scanOptions = page.locator('[data-testid*="scan-option"], [role="radiogroup"]');
    const count = await scanOptions.count();

    for (let i = 0; i < count; i++) {
      const option = scanOptions.nth(i);
      const hasAccessibleName = await option.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('legend') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible target input', async ({ page }) => {
    const targetInput = page.locator('input[data-testid*="target"], input[aria-label*="target"]');
    const hasLabel = await targetInput.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible scan controls', async ({ page }) => {
    const scanButtons = page.locator('button[data-testid*="scan"], button:has-text("Start Scan"), button:has-text("Stop Scan")');
    const count = await scanButtons.count();

    for (let i = 0; i < count; i++) {
      const button = scanButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible settings panels', async ({ page }) => {
    const settingsPanels = page.locator('[data-testid*="settings"], [role="region"]');
    const count = await settingsPanels.count();

    for (let i = 0; i < count; i++) {
      const panel = settingsPanels.nth(i);
      const hasAccessibleName = await panel.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible results table', async ({ page }) => {
    const resultsTable = page.locator('[data-testid*="results"] table');
    const exists = await resultsTable.count() > 0;

    if (exists) {
      // Check for table headers
      const hasHeaders = await resultsTable.locator('th').count() > 0;
      expect(hasHeaders).toBeTruthy();

      // Check for aria-label or caption
      const hasAccessibleName = await resultsTable.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('caption') !== null;
      });
      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible progress indicators', async ({ page }) => {
    const progressBars = page.locator('[role="progressbar"], progress');
    const count = await progressBars.count();

    for (let i = 0; i < count; i++) {
      const progress = progressBars.nth(i);
      const hasAccessibleName = await progress.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.hasAttribute('aria-valuenow');
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check aria attributes
      const ariaValueNow = await progress.getAttribute('aria-valuenow');
      const ariaValueMin = await progress.getAttribute('aria-valuemin');
      const ariaValueMax = await progress.getAttribute('aria-valuemax');

      expect(ariaValueNow).toBeTruthy();
      expect(ariaValueMin).toBeTruthy();
      expect(ariaValueMax).toBeTruthy();
    }
  });

  test('should have accessible severity indicators', async ({ page }) => {
    const severityIndicators = page.locator('[data-testid*="severity"], [role="alert"]');
    const count = await severityIndicators.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const indicator = severityIndicators.nth(i);
      const hasAccessibleName = await indicator.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible vulnerability details', async ({ page }) => {
    const vulnerabilityDetails = page.locator('[data-testid*="vulnerability"], [role="article"]');
    const count = await vulnerabilityDetails.count();

    for (let i = 0; i < Math.min(count, 3); i++) {
      const detail = vulnerabilityDetails.nth(i);
      const hasAccessibleName = await detail.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible export controls', async ({ page }) => {
    const exportButtons = page.locator('button[aria-label*="export"], button:has-text("Export")');
    const count = await exportButtons.count();

    for (let i = 0; i < count; i++) {
      const button = exportButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.scanner,
      'scanner',
      testInfo
    );
  });
});
