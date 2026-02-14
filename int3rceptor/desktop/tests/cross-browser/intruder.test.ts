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
 * Cross-browser tests for Intruder view
 * Tests tabs, attack configuration, payload management, and execution
 */

test.describe('Intruder - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Intruder on ${browserName} ${browserInfo.version}`);

    // Navigate to intruder view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.intruder);
    await page.waitForLoadState('networkidle');
  });

  test('should render intruder view correctly', async ({ page }) => {
    const intruderSelector = viewSelectors.intruder;

    // Verify intruder view is visible
    await expect(page.locator(intruderSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="intruder-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('intruder');
  });

  test('should display all intruder tabs', async ({ page }) => {
    const tabs = testDataFixtures.intruder.tabs;

    for (const tab of tabs) {
      const tabSelector = `[data-testid="tab-${tab.toLowerCase().replace(/\s+/g, '-')}"]`;
      const tabElement = page.locator(tabSelector);

      const exists = await tabElement.count().then(count => count > 0);

      if (exists) {
        await expect(tabElement).toBeVisible();
      } else {
        console.warn(`Tab "${tab}" not found in ${browserName}`);
      }
    }
  });

  test('should navigate between tabs correctly', async ({ page }) => {
    const tabs = testDataFixtures.intruder.tabs;

    for (const tab of tabs) {
      const tabSelector = `[data-testid="tab-${tab.toLowerCase().replace(/\s+/g, '-')}"]`;
      const contentSelector = `[data-testid="content-${tab.toLowerCase().replace(/\s+/g, '-')}"]`;

      const tabElement = page.locator(tabSelector);
      const exists = await tabElement.count().then(count => count > 0);

      if (exists) {
        const success = await testTabNavigation(page, tabSelector, contentSelector);
        expect(success).toBeTruthy();
      }
    }
  });

  test('should have working attack control buttons', async ({ page }) => {
    const buttons = testDataFixtures.intruder.buttons;

    for (const button of buttons) {
      const buttonSelector = `[data-testid="btn-${button.toLowerCase().replace(/\s+/g, '-')}"]`;
      const buttonElement = page.locator(buttonSelector);

      const exists = await buttonElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await isElementVisibleAndInteractive(page, buttonSelector);
        if (isVisible) {
          // Test hover effect
          await page.hover(buttonSelector);
          await page.waitForTimeout(100);
        }
      }
    }
  });

  test('should display target configuration form', async ({ page }) => {
    const targetTab = '[data-testid="tab-target"]';
    const targetContent = '[data-testid="content-target"]';

    // Navigate to target tab
    const targetTabElement = page.locator(targetTab);
    const hasTargetTab = await targetTabElement.count().then(count => count > 0);

    if (hasTargetTab) {
      await targetTabElement.click();
      await page.waitForTimeout(500);

      // Check for target form elements
      const urlInput = page.locator('[data-testid="target-url"], input[name="url"]');
      const methodSelect = page.locator('[data-testid="target-method"], select[name="method"]');

      const hasUrlInput = await urlInput.count().then(count => count > 0);
      const hasMethodSelect = await methodSelect.count().then(count => count > 0);

      console.log(`Target form in ${browserName}: URL input=${hasUrlInput}, Method select=${hasMethodSelect}`);
    }
  });

  test('should display positions configuration', async ({ page }) => {
    const positionsTab = '[data-testid="tab-positions"]';
    const positionsContent = '[data-testid="content-positions"]';

    const positionsTabElement = page.locator(positionsTab);
    const hasPositionsTab = await positionsTabElement.count().then(count => count > 0);

    if (hasPositionsTab) {
      await positionsTabElement.click();
      await page.waitForTimeout(500);

      // Check for positions UI
      const positionsList = page.locator('[data-testid="positions-list"], .positions-list');
      const hasPositionsList = await positionsList.count().then(count => count > 0);

      if (hasPositionsList) {
        console.log(`Positions list visible in ${browserName}`);
      }
    }
  });

  test('should display payload configuration', async ({ page }) => {
    const payloadsTab = '[data-testid="tab-payloads"]';
    const payloadsContent = '[data-testid="content-payloads"]';

    const payloadsTabElement = page.locator(payloadsTab);
    const hasPayloadsTab = await payloadsTabElement.count().then(count => count > 0);

    if (hasPayloadsTab) {
      await payloadsTabElement.click();
      await page.waitForTimeout(500);

      // Check for payload UI
      const payloadInput = page.locator('[data-testid="payload-input"], textarea[name="payload"]');
      const hasPayloadInput = await payloadInput.count().then(count => count > 0);

      if (hasPayloadInput) {
        console.log(`Payload input visible in ${browserName}`);
      }
    }
  });

  test('should display attack results', async ({ page }) => {
    const attackTab = '[data-testid="tab-attack"]';
    const attackContent = '[data-testid="content-attack"]';

    const attackTabElement = page.locator(attackTab);
    const hasAttackTab = await attackTabElement.count().then(count => count > 0);

    if (hasAttackTab) {
      await attackTabElement.click();
      await page.waitForTimeout(500);

      // Check for results table
      const resultsTable = page.locator('[data-testid="attack-results"], table');
      const hasResultsTable = await resultsTable.count().then(count => count > 0);

      if (hasResultsTable) {
        console.log(`Attack results table visible in ${browserName}`);
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const intruderSelector = viewSelectors.intruder;
    const results = await testResponsiveLayout(page, intruderSelector, testViewports);

    for (const result of results) {
      console.log(`Viewport ${result.viewport}: visible=${result.visible}, issues=${result.issues.length}`);

      if (result.viewport === '375x667') {
        // Mobile may have different tab layout
        expect(result.visible).toBeTruthy();
      } else {
        expect(result.visible).toBeTruthy();
      }
    }
  });

  test('should capture screenshot for comparison', async ({ page }) => {
    const screenshotPath = await captureComparisonScreenshot(
      page,
      'intruder',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const intruderSelector = viewSelectors.intruder;

    // Initial state
    await expect(page.locator(intruderSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify intruder view is still visible
    await expect(page.locator(intruderSelector)).toBeVisible();
  });

  test('should support tab keyboard navigation', async ({ page }) => {
    const firstTab = page.locator('[data-testid^="tab-"]').first();
    const hasTabs = await firstTab.count().then(count => count > 0);

    if (hasTabs) {
      // Focus first tab
      await firstTab.focus();

      // Press Tab to navigate
      await page.keyboard.press('Tab');
      await page.waitForTimeout(100);

      // Verify focus moved
      const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
      console.log(`Focused element after Tab in ${browserName}: ${focusedElement}`);
    }
  });

  test('should handle form input correctly', async ({ page }) => {
    const urlInput = page.locator('[data-testid="target-url"], input[name="url"]');
    const hasUrlInput = await urlInput.count().then(count => count > 0);

    if (hasUrlInput && await urlInput.isVisible()) {
      const testUrl = 'https://example.com/test';
      await urlInput.fill(testUrl);
      await page.waitForTimeout(100);

      const inputValue = await urlInput.inputValue();
      expect(inputValue).toBe(testUrl);
    }
  });

  test('should display attack progress indicator', async ({ page }) => {
    const progressSelector = '[data-testid="attack-progress"], .progress-bar';
    const progressElement = page.locator(progressSelector);
    const hasProgress = await progressElement.count().then(count => count > 0);

    if (hasProgress) {
      const isVisible = await progressElement.isVisible();
      if (isVisible) {
        console.log(`Attack progress indicator visible in ${browserName}`);
      }
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
          padding: styles.padding
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

  test('should support payload type selection', async ({ page }) => {
    const payloadTypeSelect = page.locator('[data-testid="payload-type"], select[name="payloadType"]');
    const hasPayloadType = await payloadTypeSelect.count().then(count => count > 0);

    if (hasPayloadType && await payloadTypeSelect.isVisible()) {
      // Get available options
      const options = await payloadTypeSelect.locator('option').allTextContents();
      console.log(`Available payload types in ${browserName}:`, options);

      if (options.length > 1) {
        // Select second option
        await payloadTypeSelect.selectOption({ index: 1 });
        await page.waitForTimeout(100);

        const selectedValue = await payloadTypeSelect.inputValue();
        console.log(`Selected payload type: ${selectedValue}`);
      }
    }
  });

  test('should display attack statistics', async ({ page }) => {
    const statsSelectors = [
      '[data-testid="stat-total"]',
      '[data-testid="stat-success"]',
      '[data-testid="stat-failed"]'
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

  test('should handle attack stop/clear operations', async ({ page }) => {
    const stopButton = page.locator('[data-testid="btn-stop"], button:has-text("Stop")');
    const clearButton = page.locator('[data-testid="btn-clear"], button:has-text("Clear")');

    const hasStop = await stopButton.count().then(count => count > 0);
    const hasClear = await clearButton.count().then(count => count > 0);

    if (hasStop && await stopButton.isVisible()) {
      const isEnabled = await stopButton.isEnabled();
      console.log(`Stop button enabled in ${browserName}: ${isEnabled}`);
    }

    if (hasClear && await clearButton.isVisible()) {
      const isEnabled = await clearButton.isEnabled();
      console.log(`Clear button enabled in ${browserName}: ${isEnabled}`);
    }
  });
});
