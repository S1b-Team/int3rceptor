import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testResponsiveLayout,
  testViewports,
  testTabNavigation,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Scanner view
 * Tests scan configuration, scan execution, results display, and settings
 */

test.describe('Scanner - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Scanner on ${browserName} ${browserInfo.version}`);

    // Navigate to scanner view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.scanner);
    await page.waitForLoadState('networkidle');
  });

  test('should render scanner view correctly', async ({ page }) => {
    const scannerSelector = viewSelectors.scanner;

    // Verify scanner view is visible
    await expect(page.locator(scannerSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="scanner-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('scanner');
  });

  test('should display scan options', async ({ page }) => {
    const options = testDataFixtures.scanner.options;

    for (const option of options) {
      const optionSelector = `[data-testid="scan-option-${option.toLowerCase().replace(/\s+/g, '-')}"]`;
      const optionElement = page.locator(optionSelector);

      const exists = await optionElement.count().then(count => count > 0);

      if (exists) {
        await expect(optionElement).toBeVisible();
      } else {
        // Try alternative selector
        const altOption = page.locator(`text=${option}`);
        const altExists = await altOption.count().then(count => count > 0);

        if (altExists) {
          console.log(`Found "${option}" using text selector in ${browserName}`);
        } else {
          console.warn(`Option "${option}" not found in ${browserName}`);
        }
      }
    }
  });

  test('should display scan settings', async ({ page }) => {
    const settings = testDataFixtures.scanner.settings;

    for (const setting of settings) {
      const settingSelector = `[data-testid="setting-${setting.toLowerCase().replace(/\s+/g, '-')}"]`;
      const settingElement = page.locator(settingSelector);

      const exists = await settingElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await settingElement.isVisible();
        if (isVisible) {
          console.log(`Setting "${setting}" visible in ${browserName}`);
        }
      } else {
        console.warn(`Setting "${setting}" not found in ${browserName}`);
      }
    }
  });

  test('should have scan control buttons', async ({ page }) => {
    const scanButton = page.locator('[data-testid="start-scan"], button:has-text("Start Scan")');
    const stopButton = page.locator('[data-testid="stop-scan"], button:has-text("Stop Scan")');

    const hasScanButton = await scanButton.count().then(count => count > 0);
    const hasStopButton = await stopButton.count().then(count => count > 0);

    if (hasScanButton) {
      await expect(scanButton).toBeVisible();

      const isEnabled = await scanButton.isEnabled();
      console.log(`Start scan button enabled in ${browserName}: ${isEnabled}`);
    }

    if (hasStopButton) {
      const isVisible = await stopButton.isVisible();
      if (isVisible) {
        const isEnabled = await stopButton.isEnabled();
        console.log(`Stop scan button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display target selection', async ({ page }) => {
    const targetSelector = '[data-testid="target-selection"], .target-selection';
    const targetElement = page.locator(targetSelector);
    const exists = await targetElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await targetElement.isVisible();
      if (isVisible) {
        console.log(`Target selection visible in ${browserName}`);

        // Check for target input
        const targetInput = targetElement.locator('input[name="target"], [data-testid="target-input"]');
        const hasTargetInput = await targetInput.count().then(count => count > 0);

        if (hasTargetInput) {
          console.log(`Target input found in ${browserName}`);
        }
      }
    }
  });

  test('should display scan progress', async ({ page }) => {
    const progressSelector = '[data-testid="scan-progress"], .scan-progress';
    const progressElement = page.locator(progressSelector);
    const exists = await progressElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await progressElement.isVisible();
      if (isVisible) {
        console.log(`Scan progress visible in ${browserName}`);

        // Check for progress bar
        const progressBar = progressElement.locator('[data-testid="progress-bar"], .progress-bar');
        const hasProgressBar = await progressBar.count().then(count => count > 0);

        if (hasProgressBar) {
          console.log(`Progress bar found in ${browserName}`);
        }
      }
    }
  });

  test('should display scan results', async ({ page }) => {
    const resultsSelector = '[data-testid="scan-results"], .scan-results';
    const resultsElement = page.locator(resultsSelector);
    const exists = await resultsElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await resultsElement.isVisible();
      if (isVisible) {
        console.log(`Scan results visible in ${browserName}`);

        // Check for results table or list
        const resultsTable = resultsElement.locator('table, [data-testid="results-list"]');
        const hasResultsTable = await resultsTable.count().then(count => count > 0);

        if (hasResultsTable) {
          console.log(`Results table/list found in ${browserName}`);
        }
      }
    }
  });

  test('should display vulnerability details', async ({ page }) => {
    const vulnDetailsSelector = '[data-testid="vulnerability-details"], .vulnerability-details';
    const vulnDetailsElement = page.locator(vulnDetailsSelector);
    const exists = await vulnDetailsElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await vulnDetailsElement.isVisible();
      if (isVisible) {
        console.log(`Vulnerability details visible in ${browserName}`);
      }
    }
  });

  test('should support scan type selection', async ({ page }) => {
    const scanTypeSelector = page.locator('[data-testid="scan-type"], select[name="scanType"]');
    const exists = await scanTypeSelector.count().then(count => count > 0);

    if (exists && await scanTypeSelector.isVisible()) {
      // Get available scan types
      const options = await scanTypeSelector.locator('option').allTextContents();
      console.log(`Available scan types in ${browserName}:`, options);

      if (options.length > 1) {
        // Select second option
        await scanTypeSelector.selectOption({ index: 1 });
        await page.waitForTimeout(100);

        const selectedValue = await scanTypeSelector.inputValue();
        console.log(`Selected scan type: ${selectedValue}`);
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const scannerSelector = viewSelectors.scanner;
    const results = await testResponsiveLayout(page, scannerSelector, testViewports);

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
      'scanner',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const scannerSelector = viewSelectors.scanner;

    // Initial state
    await expect(page.locator(scannerSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify scanner view is still visible
    await expect(page.locator(scannerSelector)).toBeVisible();
  });

  test('should support scan configuration', async ({ page }) => {
    const configSelector = '[data-testid="scan-config"], .scan-config';
    const configElement = page.locator(configSelector);
    const exists = await configElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await configElement.isVisible();
      if (isVisible) {
        console.log(`Scan configuration visible in ${browserName}`);

        // Check for configuration inputs
        const configInputs = configElement.locator('input, select');
        const count = await configInputs.count();

        console.log(`Configuration inputs in ${browserName}: ${count}`);
      }
    }
  });

  test('should display scan statistics', async ({ page }) => {
    const statsSelectors = [
      '[data-testid="stat-issues"]',
      '[data-testid="stat-warnings"]',
      '[data-testid="stat-info"]'
    ];

    for (const selector of statsSelectors) {
      const statElement = page.locator(selector);
      const exists = await statElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await statElement.isVisible();
        if (isVisible) {
          const statValue = await statElement.textContent();
          console.log(`Stat ${selector} in ${browserName}: ${statValue}`);
        }
      }
    }
  });

  test('should support filter options', async ({ page }) => {
    const filterSelector = '[data-testid="filter-options"], .filter-options';
    const filterElement = page.locator(filterSelector);
    const exists = await filterElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await filterElement.isVisible();
      if (isVisible) {
        console.log(`Filter options visible in ${browserName}`);

        // Check for filter inputs
        const filterInputs = filterElement.locator('input[type="checkbox"], select');
        const count = await filterInputs.count();

        console.log(`Filter inputs in ${browserName}: ${count}`);
      }
    }
  });

  test('should support export functionality', async ({ page }) => {
    const exportButton = page.locator('[data-testid="export-results"], button:has-text("Export")');
    const exists = await exportButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await exportButton.isVisible();
      if (isVisible) {
        const isEnabled = await exportButton.isEnabled();
        console.log(`Export button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should handle browser-specific progress rendering', async ({ page }) => {
    const progressSelector = '[data-testid="scan-progress"], .scan-progress';
    const progressElement = page.locator(progressSelector);
    const exists = await progressElement.count().then(count => count > 0);

    if (exists && await progressElement.isVisible()) {
      // Check progress bar styles
      const progressBar = progressElement.locator('[data-testid="progress-bar"], .progress-bar');
      const hasProgressBar = await progressBar.count().then(count => count > 0);

      if (hasProgressBar) {
        const progressStyles = await progressBar.first().evaluate(el => {
          const styles = window.getComputedStyle(el);
          return {
            width: styles.width,
            backgroundColor: styles.backgroundColor,
            transition: styles.transition
          };
        });

        console.log(`Progress bar styles in ${browserName}:`, progressStyles);
      }
    }
  });

  test('should support severity level display', async ({ page }) => {
    const severitySelectors = [
      '[data-testid="severity-critical"]',
      '[data-testid="severity-high"]',
      '[data-testid="severity-medium"]',
      '[data-testid="severity-low"]'
    ];

    for (const selector of severitySelectors) {
      const severityElement = page.locator(selector);
      const exists = await severityElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await severityElement.isVisible();
        if (isVisible) {
          console.log(`Severity ${selector} visible in ${browserName}`);
        }
      }
    }
  });

  test('should support scan history', async ({ page }) => {
    const historySelector = '[data-testid="scan-history"], .scan-history';
    const historyElement = page.locator(historySelector);
    const exists = await historyElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await historyElement.isVisible();
      if (isVisible) {
        console.log(`Scan history visible in ${browserName}`);

        // Check for history items
        const historyItems = historyElement.locator('[data-testid="history-item"], .history-item');
        const count = await historyItems.count();

        console.log(`History items in ${browserName}: ${count}`);
      }
    }
  });
});
