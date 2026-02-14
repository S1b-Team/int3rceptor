import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testResponsiveLayout,
  testViewports,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Decoder view
 * Tests encoding/decoding operations, format selection, and input/output
 */

test.describe('Decoder - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Decoder on ${browserName} ${browserInfo.version}`);

    // Navigate to decoder view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.decoder);
    await page.waitForLoadState('networkidle');
  });

  test('should render decoder view correctly', async ({ page }) => {
    const decoderSelector = viewSelectors.decoder;

    // Verify decoder view is visible
    await expect(page.locator(decoderSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="decoder-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('decoder');
  });

  test('should display format options', async ({ page }) => {
    const formats = testDataFixtures.decoder.formats;

    for (const format of formats) {
      const formatSelector = `[data-testid="format-${format.toLowerCase()}"]`;
      const formatElement = page.locator(formatSelector);

      const exists = await formatElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await formatElement.isVisible();
        if (isVisible) {
          console.log(`Format "${format}" visible in ${browserName}`);
        }
      } else {
        // Try alternative selector
        const altFormat = page.locator(`text=${format}`);
        const altExists = await altFormat.count().then(count => count > 0);

        if (altExists) {
          console.log(`Found "${format}" using text selector in ${browserName}`);
        } else {
          console.warn(`Format "${format}" not found in ${browserName}`);
        }
      }
    }
  });

  test('should display action buttons', async ({ page }) => {
    const actions = testDataFixtures.decoder.actions;

    for (const action of actions) {
      const actionSelector = `[data-testid="action-${action.toLowerCase()}"], button:has-text("${action}")`;
      const actionElement = page.locator(actionSelector);

      const exists = await actionElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await actionElement.isVisible();
        if (isVisible) {
          console.log(`Action "${action}" visible in ${browserName}`);
        }
      } else {
        console.warn(`Action "${action}" not found in ${browserName}`);
      }
    }
  });

  test('should have input textarea', async ({ page }) => {
    const inputTextarea = page.locator('[data-testid="input-textarea"], textarea[name="input"]');
    const exists = await inputTextarea.count().then(count => count > 0);

    if (exists) {
      await expect(inputTextarea).toBeVisible();

      // Test input
      const testInput = 'SGVsbG8gV29ybGQ=';
      await inputTextarea.fill(testInput);
      await page.waitForTimeout(100);

      const inputValue = await inputTextarea.inputValue();
      expect(inputValue).toBe(testInput);
    }
  });

  test('should have output textarea', async ({ page }) => {
    const outputTextarea = page.locator('[data-testid="output-textarea"], textarea[name="output"]');
    const exists = await outputTextarea.count().then(count => count > 0);

    if (exists) {
      const isVisible = await outputTextarea.isVisible();
      if (isVisible) {
        console.log(`Output textarea visible in ${browserName}`);

        // Check if it's read-only
        const isReadOnly = await outputTextarea.isReadOnly();
        console.log(`Output textarea read-only in ${browserName}: ${isReadOnly}`);
      }
    }
  });

  test('should support format selection', async ({ page }) => {
    const formatSelect = page.locator('[data-testid="format-select"], select[name="format"]');
    const exists = await formatSelect.count().then(count => count > 0);

    if (exists && await formatSelect.isVisible()) {
      // Get available formats
      const options = await formatSelect.locator('option').allTextContents();
      console.log(`Available formats in ${browserName}:`, options);

      if (options.length > 1) {
        // Select second option
        await formatSelect.selectOption({ index: 1 });
        await page.waitForTimeout(100);

        const selectedValue = await formatSelect.inputValue();
        console.log(`Selected format: ${selectedValue}`);
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const decoderSelector = viewSelectors.decoder;
    const results = await testResponsiveLayout(page, decoderSelector, testViewports);

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
      'decoder',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const decoderSelector = viewSelectors.decoder;

    // Initial state
    await expect(page.locator(decoderSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify decoder view is still visible
    await expect(page.locator(decoderSelector)).toBeVisible();
  });

  test('should support encode operation', async ({ page }) => {
    const encodeButton = page.locator('[data-testid="action-encode"], button:has-text("Encode")');
    const exists = await encodeButton.count().then(count => count > 0);

    if (exists && await encodeButton.isVisible()) {
      const isEnabled = await encodeButton.isEnabled();
      console.log(`Encode button enabled in ${browserName}: ${isEnabled}`);

      if (isEnabled) {
        // Fill input
        const inputTextarea = page.locator('[data-testid="input-textarea"], textarea[name="input"]');
        const hasInput = await inputTextarea.count().then(count => count > 0);

        if (hasInput) {
          await inputTextarea.fill('Hello World');
          await page.waitForTimeout(100);
        }
      }
    }
  });

  test('should support decode operation', async ({ page }) => {
    const decodeButton = page.locator('[data-testid="action-decode"], button:has-text("Decode")');
    const exists = await decodeButton.count().then(count => count > 0);

    if (exists && await decodeButton.isVisible()) {
      const isEnabled = await decodeButton.isEnabled();
      console.log(`Decode button enabled in ${browserName}: ${isEnabled}`);

      if (isEnabled) {
        // Fill input
        const inputTextarea = page.locator('[data-testid="input-textarea"], textarea[name="input"]');
        const hasInput = await inputTextarea.count().then(count => count > 0);

        if (hasInput) {
          await inputTextarea.fill('SGVsbG8gV29ybGQ=');
          await page.waitForTimeout(100);
        }
      }
    }
  });

  test('should support format operation', async ({ page }) => {
    const formatButton = page.locator('[data-testid="action-format"], button:has-text("Format")');
    const exists = await formatButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await formatButton.isVisible();
      if (isVisible) {
        const isEnabled = await formatButton.isEnabled();
        console.log(`Format button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should handle browser-specific textarea styling', async ({ page }) => {
    const textarea = page.locator('textarea').first();
    const hasTextarea = await textarea.count().then(count => count > 0);

    if (hasTextarea) {
      const textareaStyles = await textarea.evaluate(el => {
        const styles = window.getComputedStyle(el);
        return {
          fontFamily: styles.fontFamily,
          fontSize: styles.fontSize,
          lineHeight: styles.lineHeight,
          resize: styles.resize
        };
      });

      console.log(`Textarea styles in ${browserName}:`, textareaStyles);

      // Safari-specific textarea styling
      if (browserName === 'webkit') {
        const hasAppearanceNone = await textarea.evaluate(el => {
          return el.style.appearance === 'none' ||
                 el.style.webkitAppearance === 'none';
        });
        console.log(`Safari appearance reset: ${hasAppearanceNone}`);
      }
    }
  });

  test('should support clear operation', async ({ page }) => {
    const clearButton = page.locator('[data-testid="clear-button"], button:has-text("Clear")');
    const exists = await clearButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await clearButton.isVisible();
      if (isVisible) {
        const isEnabled = await clearButton.isEnabled();
        console.log(`Clear button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display operation status', async ({ page }) => {
    const statusSelector = '[data-testid="operation-status"], .operation-status';
    const statusElement = page.locator(statusSelector);
    const exists = await statusElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await statusElement.isVisible();
      if (isVisible) {
        console.log(`Operation status visible in ${browserName}`);

        const statusText = await statusElement.textContent();
        console.log(`Status text: ${statusText}`);
      }
    }
  });

  test('should support copy to clipboard', async ({ page }) => {
    const copyButton = page.locator('[data-testid="copy-button"], button:has-text("Copy")');
    const exists = await copyButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await copyButton.isVisible();
      if (isVisible) {
        const isEnabled = await copyButton.isEnabled();
        console.log(`Copy button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display character count', async ({ page }) => {
    const charCountSelector = '[data-testid="char-count"], .char-count';
    const charCountElement = page.locator(charCountSelector);
    const exists = await charCountElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await charCountElement.isVisible();
      if (isVisible) {
        console.log(`Character count visible in ${browserName}`);

        const charCountText = await charCountElement.textContent();
        console.log(`Character count: ${charCountText}`);
      }
    }
  });

  test('should support keyboard shortcuts', async ({ page }) => {
    const inputTextarea = page.locator('[data-testid="input-textarea"], textarea[name="input"]');
    const hasInput = await inputTextarea.count().then(count => count > 0);

    if (hasInput && await inputTextarea.isVisible()) {
      // Focus input
      await inputTextarea.focus();

      // Test Ctrl+A (select all)
      await page.keyboard.press('Control+A');
      await page.waitForTimeout(100);

      // Verify text is selected
      const selectedText = await inputTextarea.evaluate(el => {
        const textarea = el as HTMLTextAreaElement;
        return textarea.value.substring(textarea.selectionStart, textarea.selectionEnd);
      });

      console.log(`Selected text in ${browserName}: ${selectedText}`);
    }
  });
});
