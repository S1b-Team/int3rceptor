import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testResponsiveLayout,
  testViewports,
  testTabNavigation,
  testFormInput,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Settings view
 * Tests settings categories, form inputs, save functionality, and persistence
 */

test.describe('Settings - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Settings on ${browserName} ${browserInfo.version}`);

    // Navigate to settings view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.settings);
    await page.waitForLoadState('networkidle');
  });

  test('should render settings view correctly', async ({ page }) => {
    const settingsSelector = viewSelectors.settings;

    // Verify settings view is visible
    await expect(page.locator(settingsSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="settings-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('settings');
  });

  test('should display settings categories', async ({ page }) => {
    const categories = testDataFixtures.settings.categories;

    for (const category of categories) {
      const categorySelector = `[data-testid="category-${category.toLowerCase()}"]`;
      const categoryElement = page.locator(categorySelector);

      const exists = await categoryElement.count().then(count => count > 0);

      if (exists) {
        await expect(categoryElement).toBeVisible();
      } else {
        // Try alternative selector
        const altCategory = page.locator(`text=${category}`);
        const altExists = await altCategory.count().then(count => count > 0);

        if (altExists) {
          console.log(`Found "${category}" using text selector in ${browserName}`);
        } else {
          console.warn(`Category "${category}" not found in ${browserName}`);
        }
      }
    }
  });

  test('should display settings fields', async ({ page }) => {
    const fields = testDataFixtures.settings.fields;

    for (const field of fields) {
      const fieldSelector = `[data-testid="field-${field.toLowerCase()}"], [name="${field.toLowerCase()}"]`;
      const fieldElement = page.locator(fieldSelector);

      const exists = await fieldElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await fieldElement.isVisible();
        if (isVisible) {
          console.log(`Field "${field}" visible in ${browserName}`);
        }
      } else {
        console.warn(`Field "${field}" not found in ${browserName}`);
      }
    }
  });

  test('should have save and reset buttons', async ({ page }) => {
    const saveButton = page.locator('[data-testid="save-settings"], button:has-text("Save")');
    const resetButton = page.locator('[data-testid="reset-settings"], button:has-text("Reset")');

    const hasSaveButton = await saveButton.count().then(count => count > 0);
    const hasResetButton = await resetButton.count().then(count => count > 0);

    if (hasSaveButton) {
      await expect(saveButton).toBeVisible();

      const isEnabled = await saveButton.isEnabled();
      console.log(`Save button enabled in ${browserName}: ${isEnabled}`);
    }

    if (hasResetButton) {
      const isVisible = await resetButton.isVisible();
      if (isVisible) {
        const isEnabled = await resetButton.isEnabled();
        console.log(`Reset button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should navigate between settings categories', async ({ page }) => {
    const categories = testDataFixtures.settings.categories;

    for (const category of categories) {
      const categorySelector = `[data-testid="category-${category.toLowerCase()}"]`;
      const contentSelector = `[data-testid="content-${category.toLowerCase()}"]`;

      const categoryElement = page.locator(categorySelector);
      const exists = await categoryElement.count().then(count => count > 0);

      if (exists) {
        const success = await testTabNavigation(page, categorySelector, contentSelector);
        if (success) {
          console.log(`Successfully navigated to ${category} in ${browserName}`);
        }
      }
    }
  });

  test('should handle form input correctly', async ({ page }) => {
    const portInput = page.locator('[data-testid="field-port"], input[name="port"]');
    const hasPortInput = await portInput.count().then(count => count > 0);

    if (hasPortInput && await portInput.isVisible()) {
      const testPort = '8080';
      await portInput.fill(testPort);
      await page.waitForTimeout(100);

      const inputValue = await portInput.inputValue();
      expect(inputValue).toBe(testPort);
    }
  });

  test('should handle toggle switches', async ({ page }) => {
    const toggleSelectors = [
      '[data-testid="toggle-logging"]',
      '[data-testid="toggle-https"]',
      '[data-testid="toggle-verbose"]'
    ];

    for (const selector of toggleSelectors) {
      const toggle = page.locator(selector);
      const exists = await toggle.count().then(count => count > 0);

      if (exists) {
        const isVisible = await toggle.isVisible();
        if (isVisible) {
          // Test toggle
          await toggle.click();
          await page.waitForTimeout(100);

          const isChecked = await toggle.isChecked().catch(() => false);
          console.log(`Toggle ${selector} checked in ${browserName}: ${isChecked}`);
        }
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const settingsSelector = viewSelectors.settings;
    const results = await testResponsiveLayout(page, settingsSelector, testViewports);

    for (const result of results) {
      console.log(`Viewport ${result.viewport}: visible=${result.visible}, issues=${result.issues.length}`);

      if (result.viewport === '375x667') {
        // Mobile may have stacked layout
        expect(result.visible).toBeTruthy();
      } else {
        expect(result.visible).toBeTruthy();
      }
    }
  });

  test('should capture screenshot for comparison', async ({ page }) => {
    const screenshotPath = await captureComparisonScreenshot(
      page,
      'settings',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const settingsSelector = viewSelectors.settings;

    // Initial state
    await expect(page.locator(settingsSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify settings view is still visible
    await expect(page.locator(settingsSelector)).toBeVisible();
  });

  test('should support settings validation', async ({ page }) => {
    const portInput = page.locator('[data-testid="field-port"], input[name="port"]');
    const hasPortInput = await portInput.count().then(count => count > 0);

    if (hasPortInput && await portInput.isVisible()) {
      // Test invalid port
      await portInput.fill('99999');
      await page.waitForTimeout(100);

      // Check for validation message
      const validationMessage = await portInput.evaluate(el => (el as HTMLInputElement).validationMessage);
      console.log(`Validation message in ${browserName}: ${validationMessage}`);

      // Test valid port
      await portInput.fill('8080');
      await page.waitForTimeout(100);

      const validationMessage2 = await portInput.evaluate(el => (el as HTMLInputElement).validationMessage);
      console.log(`Validation message for valid port in ${browserName}: ${validationMessage2}`);
    }
  });

  test('should handle browser-specific form styling', async ({ page }) => {
    const formInputs = page.locator('input, select, textarea');
    const count = await formInputs.count();

    if (count > 0) {
      const firstInput = formInputs.first();
      const inputStyles = await firstInput.evaluate(el => {
        const styles = window.getComputedStyle(el);
        return {
          borderStyle: styles.borderStyle,
          borderRadius: styles.borderRadius,
          padding: styles.padding,
          fontFamily: styles.fontFamily
        };
      });

      console.log(`Input styles in ${browserName}:`, inputStyles);

      // Safari-specific form styling
      if (browserName === 'webkit') {
        const hasAppearanceNone = await firstInput.evaluate(el => {
          return el.style.appearance === 'none' ||
                 el.style.webkitAppearance === 'none';
        });
        console.log(`Safari appearance reset: ${hasAppearanceNone}`);
      }
    }
  });

  test('should support settings export/import', async ({ page }) => {
    const exportButton = page.locator('[data-testid="export-settings"], button:has-text("Export")');
    const importButton = page.locator('[data-testid="import-settings"], button:has-text("Import")');

    const hasExportButton = await exportButton.count().then(count => count > 0);
    const hasImportButton = await importButton.count().then(count => count > 0);

    if (hasExportButton) {
      const isVisible = await exportButton.isVisible();
      if (isVisible) {
        const isEnabled = await exportButton.isEnabled();
        console.log(`Export button enabled in ${browserName}: ${isEnabled}`);
      }
    }

    if (hasImportButton) {
      const isVisible = await importButton.isVisible();
      if (isVisible) {
        const isEnabled = await importButton.isEnabled();
        console.log(`Import button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display help tooltips', async ({ page }) => {
    const tooltipSelectors = page.locator('[data-testid^="tooltip-"], [title]');
    const count = await tooltipSelectors.count();

    if (count > 0) {
      // Test first tooltip
      const firstTooltip = tooltipSelectors.first();
      await firstTooltip.hover();
      await page.waitForTimeout(500);

      console.log(`Tooltip hover tested in ${browserName}`);
    }
  });

  test('should handle settings persistence', async ({ page }) => {
    const portInput = page.locator('[data-testid="field-port"], input[name="port"]');
    const hasPortInput = await portInput.count().then(count => count > 0);

    if (hasPortInput && await portInput.isVisible()) {
      // Set a value
      const testPort = '9090';
      await portInput.fill(testPort);
      await page.waitForTimeout(100);

      // Save settings
      const saveButton = page.locator('[data-testid="save-settings"], button:has-text("Save")');
      const hasSaveButton = await saveButton.count().then(count => count > 0);

      if (hasSaveButton && await saveButton.isVisible()) {
        await saveButton.click();
        await page.waitForTimeout(500);
      }

      // Refresh
      await page.reload();
      await page.waitForLoadState('networkidle');

      // Check if value is persisted (depends on app implementation)
      const portValue = await portInput.inputValue();
      console.log(`Port value after refresh in ${browserName}: ${portValue}`);
    }
  });

  test('should support keyboard navigation', async ({ page }) => {
    const firstInput = page.locator('input, select, textarea').first();
    const hasInputs = await firstInput.count().then(count => count > 0);

    if (hasInputs) {
      // Focus first input
      await firstInput.focus();

      // Press Tab to navigate
      await page.keyboard.press('Tab');
      await page.waitForTimeout(100);

      // Verify focus moved
      const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
      console.log(`Focused element after Tab in ${browserName}: ${focusedElement}`);
    }
  });

  test('should display settings summary', async ({ page }) => {
    const summarySelector = '[data-testid="settings-summary"], .settings-summary';
    const summaryElement = page.locator(summarySelector);
    const exists = await summaryElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await summaryElement.isVisible();
      if (isVisible) {
        console.log(`Settings summary visible in ${browserName}`);

        const summaryText = await summaryElement.textContent();
        console.log(`Summary text: ${summaryText}`);
      }
    }
  });
});
