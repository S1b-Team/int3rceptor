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
 * Accessibility tests for Intruder view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Intruder - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'intruder', navSelectors.intruder, viewSelectors.intruder);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.intruder);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.intruder);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.intruder);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.intruder);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible tabs', async ({ page }) => {
    const tabs = page.locator('[role="tablist"] [role="tab"]');
    const count = await tabs.count();
    expect(count).toBeGreaterThan(0);

    for (let i = 0; i < count; i++) {
      const tab = tabs.nth(i);
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

  test('should navigate tabs with keyboard', async ({ page }) => {
    const tabs = page.locator('[role="tablist"] [role="tab"]');
    const count = await tabs.count();

    if (count > 0) {
      await tabs.first().focus();

      // Test arrow key navigation
      await page.keyboard.press('ArrowRight');
      await page.waitForTimeout(100);

      const focusedElement = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement;
        return {
          tagName: active?.tagName,
          role: active?.getAttribute('role')
        };
      });

      expect(focusedElement.role).toBe('tab');
    }
  });

  test('should have accessible attack controls', async ({ page }) => {
    const attackButtons = page.locator('button[data-testid*="attack"], button:has-text("Start Attack"), button:has-text("Stop Attack")');
    const count = await attackButtons.count();

    for (let i = 0; i < count; i++) {
      const button = attackButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible target inputs', async ({ page }) => {
    const targetInputs = page.locator('input[data-testid*="target"], input[aria-label*="target"]');
    const count = await targetInputs.count();

    for (let i = 0; i < count; i++) {
      const input = targetInputs.nth(i);
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

  test('should have accessible payload editors', async ({ page }) => {
    const payloadEditors = page.locator('textarea[data-testid*="payload"], [data-testid*="payload"] textarea');
    const count = await payloadEditors.count();

    for (let i = 0; i < count; i++) {
      const editor = payloadEditors.nth(i);
      const hasLabel = await editor.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible position tables', async ({ page }) => {
    const tables = page.locator('[data-testid*="position"] table');
    const count = await tables.count();

    for (let i = 0; i < count; i++) {
      const table = tables.nth(i);

      // Check for table headers
      const hasHeaders = await table.locator('th').count() > 0;
      expect(hasHeaders).toBeTruthy();

      // Check for aria-label or caption
      const hasAccessibleName = await table.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('caption') !== null;
      });
      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible result displays', async ({ page }) => {
    const results = page.locator('[data-testid*="result"], [role="status"]');
    const count = await results.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const result = results.nth(i);
      const hasAccessibleName = await result.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible configuration panels', async ({ page }) => {
    const panels = page.locator('[data-testid*="config"], [role="region"]');
    const count = await panels.count();

    for (let i = 0; i < count; i++) {
      const panel = panels.nth(i);
      const hasAccessibleName = await panel.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.hasAttribute('aria-describedby');
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

  test('should have accessible dropdowns', async ({ page }) => {
    const dropdowns = page.locator('select, [role="combobox"]');
    const count = await dropdowns.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const dropdown = dropdowns.nth(i);
      const hasLabel = await dropdown.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.intruder,
      'intruder',
      testInfo
    );
  });
});
