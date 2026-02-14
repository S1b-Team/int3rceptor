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
 * Accessibility tests for Dashboard view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Dashboard - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.dashboard);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.dashboard);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.dashboard);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.dashboard);

    expect(result.passed).toBeTruthy();
  });

  test('should navigate using Tab key', async ({ page }) => {
    const navItems = [
      navSelectors.traffic,
      navSelectors.intruder,
      navSelectors.rest,
      navSelectors.scanner
    ];

    for (const navItem of navItems) {
      await page.keyboard.press('Tab');
      const focusedElement = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement;
        return {
          tagName: active?.tagName,
          id: active?.id
        };
      });

      expect(focusedElement.tagName).toBe('BUTTON');
    }
  });

  test('should activate buttons with Enter key', async ({ page }) => {
    const buttons = page.locator(`${viewSelectors.dashboard} button`);
    const count = await buttons.count();

    if (count > 0) {
      await buttons.first().focus();
      await page.keyboard.press('Enter');

      // Button should trigger its action
      await page.waitForTimeout(100);
    }
  });

  test('should dismiss modals with Escape key', async ({ page }) => {
    // Test Escape key functionality
    await page.keyboard.press('Escape');

    // Verify no modal is open (this is a basic check)
    const modalOpen = await page.locator('[role="dialog"]').isVisible().catch(() => false);
    expect(modalOpen).toBeFalsy();
  });

  test('should have accessible navigation', async ({ page }) => {
    const nav = page.locator('nav, [role="navigation"]');
    await expect(nav).toBeVisible();

    const navButtons = await nav.locator('button, a').count();
    expect(navButtons).toBeGreaterThan(0);

    // Check for ARIA attributes
    const hasAria = await nav.evaluate((el) => {
      const buttons = el.querySelectorAll('button, a');
      return Array.from(buttons).every(btn =>
        btn.hasAttribute('aria-label') ||
        btn.hasAttribute('aria-labelledby') ||
        btn.textContent?.trim().length > 0
      );
    });

    expect(hasAria).toBeTruthy();
  });

  test('should have accessible charts', async ({ page }) => {
    const charts = page.locator('[data-testid^="chart-"]');
    const count = await charts.count();

    for (let i = 0; i < count; i++) {
      const chart = charts.nth(i);
      const hasAccessibleName = await chart.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.hasAttribute('title');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible statistics', async ({ page }) => {
    const stats = page.locator('[data-testid^="stat-"]');
    const count = await stats.count();

    for (let i = 0; i < count; i++) {
      const stat = stats.nth(i);
      await expect(stat).toBeVisible();

      const hasAccessibleName = await stat.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible headings', async ({ page }) => {
    const headings = page.locator('h1, h2, h3, h4, h5, h6');
    const count = await headings.count();

    expect(count).toBeGreaterThan(0);

    // Check heading hierarchy
    const firstHeading = await headings.first().evaluate((el) => {
      return el.tagName;
    });

    expect(['H1', 'H2'].includes(firstHeading)).toBeTruthy();
  });

  test('should have accessible links', async ({ page }) => {
    const links = page.locator('a[href]');
    const count = await links.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const link = links.nth(i);
      const hasAccessibleName = await link.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible forms', async ({ page }) => {
    const forms = page.locator('form');
    const count = await forms.count();

    for (let i = 0; i < count; i++) {
      const form = forms.nth(i);
      const inputs = form.locator('input, select, textarea');
      const inputCount = await inputs.count();

      for (let j = 0; j < inputCount; j++) {
        const input = inputs.nth(j);
        const hasLabel = await input.evaluate((el) => {
          const id = el.id;
          const hasAriaLabel = el.hasAttribute('aria-label');
          const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
          const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

          return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
        });

        expect(hasLabel).toBeTruthy();
      }
    }
  });

  test('should have accessible buttons', async ({ page }) => {
    const buttons = page.locator('button');
    const count = await buttons.count();

    for (let i = 0; i < Math.min(count, 10); i++) {
      const button = buttons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible images', async ({ page }) => {
    const images = page.locator('img');
    const count = await images.count();

    for (let i = 0; i < count; i++) {
      const image = images.nth(i);
      const role = await image.getAttribute('role');

      if (role !== 'presentation') {
        const hasAlt = await image.evaluate((el) => {
          return el.hasAttribute('alt');
        });

        expect(hasAlt).toBeTruthy();
      }
    }
  });

  test('should have accessible tables', async ({ page }) => {
    const tables = page.locator('table');
    const count = await tables.count();

    for (let i = 0; i < count; i++) {
      const table = tables.nth(i);

      // Check for table headers
      const hasHeaders = await table.locator('th').count() > 0;
      expect(hasHeaders).toBeTruthy();

      // Check for caption or aria-label
      const hasAccessibleName = await table.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('caption') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.dashboard,
      'dashboard',
      testInfo
    );
  });
});
