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
 * Accessibility tests for Decoder view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Decoder - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'decoder', navSelectors.decoder, viewSelectors.decoder);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.decoder);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.decoder);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.decoder);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.decoder);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible format selector', async ({ page }) => {
    const formatSelector = page.locator('select[aria-label*="format"], [data-testid*="format"]');
    const hasLabel = await formatSelector.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible input textarea', async ({ page }) => {
    const inputTextarea = page.locator('textarea[data-testid*="input"], textarea[aria-label*="input"]');
    const hasLabel = await inputTextarea.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible output textarea', async ({ page }) => {
    const outputTextarea = page.locator('textarea[data-testid*="output"], textarea[aria-label*="output"], [role="log"]');
    const hasLabel = await outputTextarea.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();

    // Check if output is readonly
    const isReadonly = await outputTextarea.getAttribute('readonly');
    expect(isReadonly).toBeTruthy();
  });

  test('should have accessible encode button', async ({ page }) => {
    const encodeButton = page.locator('button:has-text("Encode"), button[data-testid*="encode"]');
    const hasAccessibleName = await encodeButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible decode button', async ({ page }) => {
    const decodeButton = page.locator('button:has-text("Decode"), button[data-testid*="decode"]');
    const hasAccessibleName = await decodeButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible format button', async ({ page }) => {
    const formatButton = page.locator('button:has-text("Format"), button[data-testid*="format"]');
    const hasAccessibleName = await formatButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible clear buttons', async ({ page }) => {
    const clearButtons = page.locator('button:has-text("Clear"), button[aria-label*="clear"]');
    const count = await clearButtons.count();

    for (let i = 0; i < count; i++) {
      const button = clearButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible copy buttons', async ({ page }) => {
    const copyButtons = page.locator('button:has-text("Copy"), button[aria-label*="copy"]');
    const count = await copyButtons.count();

    for (let i = 0; i < count; i++) {
      const button = copyButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible format tabs', async ({ page }) => {
    const formatTabs = page.locator('[role="tablist"] [role="tab"]');
    const count = await formatTabs.count();

    for (let i = 0; i < count; i++) {
      const tab = formatTabs.nth(i);
      const hasAccessibleName = await tab.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check aria-selected attribute
      const ariaSelected = await tab.getAttribute('aria-selected');
      expect(['true', 'false'].includes(ariaSelected || '')).toBeTruthy();
    }
  });

  test('should have accessible status messages', async ({ page }) => {
    const statusMessages = page.locator('[role="status"], [data-testid*="status"]');
    const count = await statusMessages.count();

    for (let i = 0; i < count; i++) {
      const status = statusMessages.nth(i);
      const hasAccessibleName = await status.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible error messages', async ({ page }) => {
    const errorMessages = page.locator('[role="alert"], [data-testid*="error"]');
    const count = await errorMessages.count();

    for (let i = 0; i < count; i++) {
      const error = errorMessages.nth(i);
      const hasAccessibleName = await error.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.decoder,
      'decoder',
      testInfo
    );
  });
});
