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
 * Accessibility tests for Plugins view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('Plugins - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'plugins', navSelectors.plugins, viewSelectors.plugins);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.plugins);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.plugins);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.plugins);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.plugins);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible plugin list', async ({ page }) => {
    const pluginList = page.locator('[data-testid*="plugin-list"], [role="list"]');
    const count = await pluginList.count();

    if (count > 0) {
      const hasAccessibleName = await pluginList.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check list items
      const listItems = pluginList.locator('[role="listitem"], li');
      const itemCount = await listItems.count();

      for (let i = 0; i < Math.min(itemCount, 5); i++) {
        const item = listItems.nth(i);
        const hasAccessibleName = await item.evaluate((el) => {
          return el.textContent?.trim().length > 0 ||
                 el.hasAttribute('aria-label');
        });

        expect(hasAccessibleName).toBeTruthy();
      }
    }
  });

  test('should have accessible plugin cards', async ({ page }) => {
    const pluginCards = page.locator('[data-testid*="plugin-card"], article');
    const count = await pluginCards.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const card = pluginCards.nth(i);
      const hasAccessibleName = await card.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible install buttons', async ({ page }) => {
    const installButtons = page.locator('button:has-text("Install"), button[data-testid*="install"]');
    const count = await installButtons.count();

    for (let i = 0; i < count; i++) {
      const button = installButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible uninstall buttons', async ({ page }) => {
    const uninstallButtons = page.locator('button:has-text("Uninstall"), button[data-testid*="uninstall"]');
    const count = await uninstallButtons.count();

    for (let i = 0; i < count; i++) {
      const button = uninstallButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible update buttons', async ({ page }) => {
    const updateButtons = page.locator('button:has-text("Update"), button[data-testid*="update"]');
    const count = await updateButtons.count();

    for (let i = 0; i < count; i++) {
      const button = updateButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible configure buttons', async ({ page }) => {
    const configureButtons = page.locator('button:has-text("Configure"), button[data-testid*="configure"]');
    const count = await configureButtons.count();

    for (let i = 0; i < count; i++) {
      const button = configureButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
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
    }
  });

  test('should have accessible filter controls', async ({ page }) => {
    const filters = page.locator('select[aria-label*="filter"], [role="combobox"]');
    const count = await filters.count();

    for (let i = 0; i < count; i++) {
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

  test('should have accessible plugin status indicators', async ({ page }) => {
    const statusIndicators = page.locator('[data-testid*="status"], [role="status"]');
    const count = await statusIndicators.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const indicator = statusIndicators.nth(i);
      const hasAccessibleName = await indicator.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible plugin descriptions', async ({ page }) => {
    const descriptions = page.locator('[data-testid*="description"], p');
    const count = await descriptions.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const description = descriptions.nth(i);
      const hasText = await description.evaluate((el) => {
        return el.textContent?.trim().length > 0;
      });

      expect(hasText).toBeTruthy();
    }
  });

  test('should have accessible modal dialogs', async ({ page }) => {
    // Check if there are any modals
    const modals = page.locator('[role="dialog"]');
    const count = await modals.count();

    if (count > 0) {
      const modal = modals.first();
      const hasAccessibleName = await modal.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();

      // Check for focus trap
      const focusableElements = await modal.locator('button, input, select, textarea, a').count();
      expect(focusableElements).toBeGreaterThan(0);
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.plugins,
      'plugins',
      testInfo
    );
  });
});
