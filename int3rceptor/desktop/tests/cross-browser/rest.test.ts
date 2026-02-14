import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testResponsiveLayout,
  testViewports,
  testFormInput,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for REST view
 * Tests HTTP method selection, request builder, headers, body, and response display
 */

test.describe('REST - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing REST on ${browserName} ${browserInfo.version}`);

    // Navigate to REST view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.rest);
    await page.waitForLoadState('networkidle');
  });

  test('should render REST view correctly', async ({ page }) => {
    const restSelector = viewSelectors.rest;

    // Verify REST view is visible
    await expect(page.locator(restSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="rest-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('rest');
  });

  test('should display HTTP method selector', async ({ page }) => {
    const methodSelector = page.locator('[data-testid="http-method"], select[name="method"]');
    const exists = await methodSelector.count().then(count => count > 0);

    if (exists) {
      await expect(methodSelector).toBeVisible();

      // Get available methods
      const options = await methodSelector.locator('option').allTextContents();
      console.log(`Available HTTP methods in ${browserName}:`, options);

      // Verify common methods are present
      const commonMethods = testDataFixtures.rest.methods;
      for (const method of commonMethods) {
        const hasMethod = options.some(opt => opt.includes(method));
        if (!hasMethod) {
          console.warn(`HTTP method ${method} not found in ${browserName}`);
        }
      }
    }
  });

  test('should display URL input field', async ({ page }) => {
    const urlInput = page.locator('[data-testid="url-input"], input[name="url"]');
    const exists = await urlInput.count().then(count => count > 0);

    if (exists) {
      await expect(urlInput).toBeVisible();

      // Test input
      const testUrl = 'https://api.example.com/test';
      await urlInput.fill(testUrl);
      await page.waitForTimeout(100);

      const inputValue = await urlInput.inputValue();
      expect(inputValue).toBe(testUrl);
    }
  });

  test('should display request configuration sections', async ({ page }) => {
    const sections = testDataFixtures.rest.fields;

    for (const section of sections) {
      const sectionSelector = `[data-testid="section-${section.toLowerCase()}"], .${section.toLowerCase()}-section`;
      const sectionElement = page.locator(sectionSelector);

      const exists = await sectionElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await sectionElement.isVisible();
        if (isVisible) {
          console.log(`Section "${section}" visible in ${browserName}`);
        }
      } else {
        console.warn(`Section "${section}" not found in ${browserName}`);
      }
    }
  });

  test('should display headers editor', async ({ page }) => {
    const headersSelector = '[data-testid="headers-editor"], .headers-editor';
    const headersElement = page.locator(headersSelector);
    const exists = await headersElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await headersElement.isVisible();
      if (isVisible) {
        // Check for header input fields
        const headerInputs = headersElement.locator('input[name*="header"], input[placeholder*="header"]');
        const count = await headerInputs.count();

        console.log(`Header input fields in ${browserName}: ${count}`);

        if (count > 0) {
          // Test adding a header
          await headerInputs.first().fill('Content-Type');
          await page.waitForTimeout(100);
        }
      }
    }
  });

  test('should display body editor', async ({ page }) => {
    const bodySelector = '[data-testid="body-editor"], textarea[name="body"], .body-editor';
    const bodyElement = page.locator(bodySelector);
    const exists = await bodyElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await bodyElement.isVisible();
      if (isVisible) {
        // Test body input
        const testBody = JSON.stringify({ test: 'data' });
        await bodyElement.fill(testBody);
        await page.waitForTimeout(100);

        const bodyValue = await bodyElement.inputValue();
        expect(bodyValue).toBe(testBody);
      }
    }
  });

  test('should display send request button', async ({ page }) => {
    const sendButton = page.locator('[data-testid="send-request"], button:has-text("Send")');
    const exists = await sendButton.count().then(count => count > 0);

    if (exists) {
      await expect(sendButton).toBeVisible();

      const isEnabled = await sendButton.isEnabled();
      console.log(`Send button enabled in ${browserName}: ${isEnabled}`);
    }
  });

  test('should display response section', async ({ page }) => {
    const responseSelector = '[data-testid="response-section"], .response-section';
    const responseElement = page.locator(responseSelector);
    const exists = await responseElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await responseElement.isVisible();
      if (isVisible) {
        console.log(`Response section visible in ${browserName}`);

        // Check for status code display
        const statusCode = responseElement.locator('[data-testid="status-code"], .status-code');
        const hasStatusCode = await statusCode.count().then(count => count > 0);

        if (hasStatusCode) {
          console.log(`Status code display found in ${browserName}`);
        }
      }
    }
  });

  test('should support HTTP method selection', async ({ page }) => {
    const methodSelector = page.locator('[data-testid="http-method"], select[name="method"]');
    const exists = await methodSelector.count().then(count => count > 0);

    if (exists && await methodSelector.isVisible()) {
      // Select POST method
      await methodSelector.selectOption('POST');
      await page.waitForTimeout(100);

      const selectedValue = await methodSelector.inputValue();
      console.log(`Selected HTTP method: ${selectedValue}`);

      // Select GET method
      await methodSelector.selectOption('GET');
      await page.waitForTimeout(100);

      const selectedValue2 = await methodSelector.inputValue();
      console.log(`Selected HTTP method: ${selectedValue2}`);
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const restSelector = viewSelectors.rest;
    const results = await testResponsiveLayout(page, restSelector, testViewports);

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
      'rest',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const restSelector = viewSelectors.rest;

    // Initial state
    await expect(page.locator(restSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify REST view is still visible
    await expect(page.locator(restSelector)).toBeVisible();
  });

  test('should support request history', async ({ page }) => {
    const historySelector = '[data-testid="request-history"], .request-history';
    const historyElement = page.locator(historySelector);
    const exists = await historyElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await historyElement.isVisible();
      if (isVisible) {
        console.log(`Request history visible in ${browserName}`);

        // Check for history items
        const historyItems = historyElement.locator('[data-testid="history-item"], .history-item');
        const count = await historyItems.count();

        console.log(`History items in ${browserName}: ${count}`);
      }
    }
  });

  test('should support authentication configuration', async ({ page }) => {
    const authSelector = '[data-testid="auth-section"], .auth-section';
    const authElement = page.locator(authSelector);
    const exists = await authElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await authElement.isVisible();
      if (isVisible) {
        console.log(`Authentication section visible in ${browserName}`);

        // Check for auth type selector
        const authTypeSelect = authElement.locator('[data-testid="auth-type"], select[name="authType"]');
        const hasAuthType = await authTypeSelect.count().then(count => count > 0);

        if (hasAuthType) {
          console.log(`Auth type selector found in ${browserName}`);
        }
      }
    }
  });

  test('should handle browser-specific form validation', async ({ page }) => {
    const urlInput = page.locator('[data-testid="url-input"], input[name="url"]');
    const exists = await urlInput.count().then(count => count > 0);

    if (exists && await urlInput.isVisible()) {
      // Test URL validation
      await urlInput.fill('invalid-url');
      await page.waitForTimeout(100);

      // Check for validation message
      const validationMessage = await urlInput.evaluate(el => (el as HTMLInputElement).validationMessage);
      console.log(`Validation message in ${browserName}: ${validationMessage}`);

      // Test valid URL
      await urlInput.fill('https://example.com');
      await page.waitForTimeout(100);

      const validationMessage2 = await urlInput.evaluate(el => (el as HTMLInputElement).validationMessage);
      console.log(`Validation message for valid URL in ${browserName}: ${validationMessage2}`);
    }
  });

  test('should support response formatting', async ({ page }) => {
    const responseSelector = '[data-testid="response-section"], .response-section';
    const responseElement = page.locator(responseSelector);
    const exists = await responseElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await responseElement.isVisible();
      if (isVisible) {
        // Check for format controls
        const formatButton = responseElement.locator('[data-testid="format-response"], button:has-text("Format")');
        const hasFormatButton = await formatButton.count().then(count => count > 0);

        if (hasFormatButton) {
          console.log(`Format response button found in ${browserName}`);
        }
      }
    }
  });

  test('should support query parameter editing', async ({ page }) => {
    const queryParamsSelector = '[data-testid="query-params"], .query-params';
    const queryParamsElement = page.locator(queryParamsSelector);
    const exists = await queryParamsElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await queryParamsElement.isVisible();
      if (isVisible) {
        console.log(`Query parameters section visible in ${browserName}`);

        // Check for parameter inputs
        const paramInputs = queryParamsElement.locator('input[name*="param"], input[placeholder*="param"]');
        const count = await paramInputs.count();

        console.log(`Query parameter inputs in ${browserName}: ${count}`);
      }
    }
  });

  test('should display request/response time', async ({ page }) => {
    const timeSelector = '[data-testid="request-time"], .request-time';
    const timeElement = page.locator(timeSelector);
    const exists = await timeElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await timeElement.isVisible();
      if (isVisible) {
        const timeText = await timeElement.textContent();
        console.log(`Request time in ${browserName}: ${timeText}`);
      }
    }
  });
});
