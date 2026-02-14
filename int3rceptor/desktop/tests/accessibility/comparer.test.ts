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
 * Accessibility tests for Comparer view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Comparer - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'comparer', navSelectors.comparer, viewSelectors.comparer);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.comparer);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.comparer);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.comparer);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.comparer);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible left panel', async ({ page }) => {
    const leftPanel = page.locator('[data-testid*="left-panel"], [data-testid*="panel-left"], [role="region"][aria-label*="left"]');
    const hasAccessibleName = await leftPanel.evaluate((el) => {
      return el.hasAttribute('aria-label') ||
             el.hasAttribute('aria-labelledby') ||
             el.querySelector('h1, h2, h3') !== null;
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible right panel', async ({ page }) => {
    const rightPanel = page.locator('[data-testid*="right-panel"], [data-testid*="panel-right"], [role="region"][aria-label*="right"]');
    const hasAccessibleName = await rightPanel.evaluate((el) => {
      return el.hasAttribute('aria-label') ||
             el.hasAttribute('aria-labelledby') ||
             el.querySelector('h1, h2, h3') !== null;
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible diff view', async ({ page }) => {
    const diffView = page.locator('[data-testid*="diff"], [role="region"][aria-label*="diff"]');
    const exists = await diffView.count() > 0;

    if (exists) {
      const hasAccessibleName = await diffView.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible input textareas', async ({ page }) => {
    const inputTextareas = page.locator('textarea[data-testid*="input"], textarea[aria-label*="input"]');
    const count = await inputTextareas.count();

    for (let i = 0; i < count; i++) {
      const textarea = inputTextareas.nth(i);
      const hasLabel = await textarea.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible compare button', async ({ page }) => {
    const compareButton = page.locator('button:has-text("Compare"), button[data-testid*="compare"]');
    const hasAccessibleName = await compareButton.evaluate((el) => {
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

  test('should have accessible swap button', async ({ page }) => {
    const swapButton = page.locator('button:has-text("Swap"), button[aria-label*="swap"]');
    const exists = await swapButton.count() > 0;

    if (exists) {
      const hasAccessibleName = await swapButton.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible diff highlighting', async ({ page }) => {
    const diffHighlights = page.locator('[data-testid*="diff-add"], [data-testid*="diff-remove"], [data-testid*="diff-change"]');
    const count = await diffHighlights.count();

    if (count > 0) {
      for (let i = 0; i < Math.min(count, 5); i++) {
        const highlight = diffHighlights.nth(i);
        const hasAccessibleName = await highlight.evaluate((el) => {
          return el.hasAttribute('aria-label') ||
                 el.textContent?.trim().length > 0;
        });

        expect(hasAccessibleName).toBeTruthy();
      }
    }
  });

  test('should have accessible export controls', async ({ page }) => {
    const exportButtons = page.locator('button:has-text("Export"), button[aria-label*="export"]');
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

  test('should have accessible line numbers', async ({ page }) => {
    const lineNumbers = page.locator('[data-testid*="line-number"], [aria-label*="line"]');
    const count = await lineNumbers.count();

    if (count > 0) {
      const hasAccessibleName = await lineNumbers.first().evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
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

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.comparer,
      'comparer',
      testInfo
    );
  });
});
