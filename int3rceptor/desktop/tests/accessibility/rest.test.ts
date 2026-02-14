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
 * Accessibility tests for REST view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('REST - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'rest', navSelectors.rest, viewSelectors.rest);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.rest);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.rest);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.rest);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.rest);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible method selector', async ({ page }) => {
    const methodSelector = page.locator('[data-testid*="method"], select[aria-label*="method"]');
    const exists = await methodSelector.count() > 0;

    if (exists) {
      const hasLabel = await methodSelector.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible URL input', async ({ page }) => {
    const urlInput = page.locator('input[type="url"], input[aria-label*="url"]');
    const hasLabel = await urlInput.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible headers editor', async ({ page }) => {
    const headersEditor = page.locator('[data-testid*="headers"], textarea[aria-label*="headers"]');
    const exists = await headersEditor.count() > 0;

    if (exists) {
      const hasLabel = await headersEditor.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible body editor', async ({ page }) => {
    const bodyEditor = page.locator('[data-testid*="body"], textarea[aria-label*="body"]');
    const exists = await bodyEditor.count() > 0;

    if (exists) {
      const hasLabel = await bodyEditor.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible send button', async ({ page }) => {
    const sendButton = page.locator('button:has-text("Send"), button[data-testid*="send"]');
    const hasAccessibleName = await sendButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible response display', async ({ page }) => {
    const responseDisplay = page.locator('[data-testid*="response"], [role="region"][aria-label*="response"]');
    const exists = await responseDisplay.count() > 0;

    if (exists) {
      const hasAccessibleName = await responseDisplay.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.hasAttribute('aria-live');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible status code display', async ({ page }) => {
    const statusCode = page.locator('[data-testid*="status"], [role="status"]');
    const exists = await statusCode.count() > 0;

    if (exists) {
      const hasAccessibleName = await statusCode.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible save controls', async ({ page }) => {
    const saveButtons = page.locator('button:has-text("Save"), button[aria-label*="save"]');
    const count = await saveButtons.count();

    for (let i = 0; i < count; i++) {
      const button = saveButtons.nth(i);
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
      viewSelectors.rest,
      'rest',
      testInfo
    );
  });
});
