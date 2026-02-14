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
 * Accessibility tests for Traffic view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Traffic - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'traffic', navSelectors.traffic, viewSelectors.traffic);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.traffic);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.traffic);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.traffic);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.traffic);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible traffic table', async ({ page }) => {
    const table = page.locator('table');
    await expect(table).toBeVisible();

    // Check for table headers
    const headers = table.locator('thead th');
    const headerCount = await headers.count();
    expect(headerCount).toBeGreaterThan(0);

    // Check headers have accessible names
    for (let i = 0; i < headerCount; i++) {
      const header = headers.nth(i);
      const hasText = await header.evaluate((el) => {
        return el.textContent?.trim().length > 0;
      });
      expect(hasText).toBeTruthy();
    }

    // Check table has aria-label or caption
    const hasAccessibleName = await table.evaluate((el) => {
      return el.hasAttribute('aria-label') ||
             el.hasAttribute('aria-labelledby') ||
             el.querySelector('caption') !== null;
    });
    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible table rows', async ({ page }) => {
    const table = page.locator('table');
    const rows = table.locator('tbody tr');
    const rowCount = await rows.count();

    if (rowCount > 0) {
      // Check first row has proper structure
      const firstRow = rows.first();
      const cells = firstRow.locator('td');
      const cellCount = await cells.count();
      expect(cellCount).toBeGreaterThan(0);
    }
  });

  test('should have accessible filter controls', async ({ page }) => {
    const filters = page.locator('[data-testid*="filter"], input[type="search"], input[type="text"]');
    const count = await filters.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const filter = filters.nth(i);
      const hasLabel = await filter.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible action buttons', async ({ page }) => {
    const actionButtons = page.locator('button[data-testid*="action"], button[aria-label]');
    const count = await actionButtons.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const button = actionButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible status indicators', async ({ page }) => {
    const statusIndicators = page.locator('[data-testid*="status"], [role="status"]');
    const count = await statusIndicators.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const indicator = statusIndicators.nth(i);
      const hasAccessibleName = await indicator.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible pagination controls', async ({ page }) => {
    const pagination = page.locator('[data-testid*="pagination"], nav[aria-label*="pagination"]');
    const exists = await pagination.count() > 0;

    if (exists) {
      const buttons = pagination.locator('button, a');
      const buttonCount = await buttons.count();

      for (let i = 0; i < buttonCount; i++) {
        const button = buttons.nth(i);
        const hasAccessibleName = await button.evaluate((el) => {
          return el.textContent?.trim().length > 0 ||
                 el.hasAttribute('aria-label') ||
                 el.hasAttribute('aria-labelledby');
        });

        expect(hasAccessibleName).toBeTruthy();
      }
    }
  });

  test('should navigate table rows with keyboard', async ({ page }) => {
    const table = page.locator('table');
    const rows = table.locator('tbody tr');
    const rowCount = await rows.count();

    if (rowCount > 0) {
      // Focus first row
      await rows.first().focus();

      // Test arrow key navigation
      await page.keyboard.press('ArrowDown');
      await page.waitForTimeout(100);

      const focusedElement = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement;
        return {
          tagName: active?.tagName,
          className: active?.className
        };
      });

      expect(focusedElement.tagName).not.toBe('BODY');
    }
  });

  test('should have accessible sort controls', async ({ page }) => {
    const sortButtons = page.locator('button[aria-sort], th[aria-sort]');
    const count = await sortButtons.count();

    for (let i = 0; i < count; i++) {
      const button = sortButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check aria-sort attribute
      const ariaSort = await button.getAttribute('aria-sort');
      expect(['ascending', 'descending', 'none'].includes(ariaSort || '')).toBeTruthy();
    }
  });

  test('should have accessible search functionality', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[aria-label*="search"]');
    const exists = await searchInput.count() > 0;

    if (exists) {
      const hasLabel = await searchInput.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();

      // Test keyboard interaction
      await searchInput.focus();
      await page.keyboard.type('test');
      await page.keyboard.press('Enter');
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
      viewSelectors.traffic,
      'traffic',
      testInfo
    );
  });
});
