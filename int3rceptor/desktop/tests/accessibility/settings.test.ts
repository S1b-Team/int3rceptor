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
 * Accessibility tests for Settings view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Settings - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'settings', navSelectors.settings, viewSelectors.settings);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.settings);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.settings);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.settings);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.settings);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible settings categories', async ({ page }) => {
    const categories = page.locator('[data-testid*="category"], [role="tab"]');
    const count = await categories.count();

    for (let i = 0; i < count; i++) {
      const category = categories.nth(i);
      const hasAccessibleName = await category.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible form inputs', async ({ page }) => {
    const inputs = page.locator('input, select, textarea');
    const count = await inputs.count();

    for (let i = 0; i < Math.min(count, 10); i++) {
      const input = inputs.nth(i);
      const hasLabel = await input.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible toggle switches', async ({ page }) => {
    const toggles = page.locator('[role="switch"], input[type="checkbox"]');
    const count = await toggles.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const toggle = toggles.nth(i);
      const hasAccessibleName = await toggle.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check aria-checked for switches
      const role = await toggle.getAttribute('role');
      if (role === 'switch') {
        const ariaChecked = await toggle.getAttribute('aria-checked');
        expect(['true', 'false'].includes(ariaChecked || '')).toBeTruthy();
      }
    }
  });

  test('should have accessible sliders', async ({ page }) => {
    const sliders = page.locator('input[type="range"], [role="slider"]');
    const count = await sliders.count();

    for (let i = 0; i < count; i++) {
      const slider = sliders.nth(i);
      const hasLabel = await slider.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();

      // Check aria attributes
      const ariaValueNow = await slider.getAttribute('aria-valuenow');
      const ariaValueMin = await slider.getAttribute('aria-valuemin');
      const ariaValueMax = await slider.getAttribute('aria-valuemax');

      expect(ariaValueNow).toBeTruthy();
      expect(ariaValueMin).toBeTruthy();
      expect(ariaValueMax).toBeTruthy();
    }
  });

  test('should have accessible save buttons', async ({ page }) => {
    const saveButtons = page.locator('button:has-text("Save"), button[data-testid*="save"]');
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

  test('should have accessible cancel buttons', async ({ page }) => {
    const cancelButtons = page.locator('button:has-text("Cancel"), button[data-testid*="cancel"]');
    const count = await cancelButtons.count();

    for (let i = 0; i < count; i++) {
      const button = cancelButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible reset buttons', async ({ page }) => {
    const resetButtons = page.locator('button:has-text("Reset"), button[data-testid*="reset"]');
    const count = await resetButtons.count();

    for (let i = 0; i < count; i++) {
      const button = resetButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible fieldsets', async ({ page }) => {
    const fieldsets = page.locator('fieldset');
    const count = await fieldsets.count();

    for (let i = 0; i < count; i++) {
      const fieldset = fieldsets.nth(i);
      const hasLegend = await fieldset.evaluate((el) => {
        return el.querySelector('legend') !== null;
      });

      expect(hasLegend).toBeTruthy();
    }
  });

  test('should have accessible help text', async ({ page }) => {
    const helpTexts = page.locator('[data-testid*="help"], [aria-describedby]');
    const count = await helpTexts.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const help = helpTexts.nth(i);
      const ariaDescribedby = await help.getAttribute('aria-describedby');

      if (ariaDescribedby) {
        const describedElement = page.locator(`#${ariaDescribedby}`);
        const isVisible = await describedElement.isVisible();
        expect(isVisible).toBeTruthy();
      }
    }
  });

  test('should have accessible error messages', async ({ page }) => {
    const errorMessages = page.locator('[role="alert"], [data-testid*="error"]');
    const count = await errorMessages.count();

    for (let i = 0; i < count; i++) {
      const error = errorMessages.nth(i);
      const hasAccessibleName = await error.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.settings,
      'settings',
      testInfo
    );
  });
});
