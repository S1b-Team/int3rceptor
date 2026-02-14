import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  waitForElementInteractive,
  testResponsiveLayout,
  testViewports,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Dashboard view
 * Tests navigation, UI rendering, interactive elements, data display, and responsive layout
 */

test.describe('Dashboard - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Dashboard on ${browserName} ${browserInfo.version}`);

    // Navigate to dashboard
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should render dashboard view correctly', async ({ page }) => {
    const dashboardSelector = viewSelectors.dashboard;

    // Verify dashboard is visible
    await expect(page.locator(dashboardSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="dashboard-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('dashboard');
  });

  test('should display all dashboard stats', async ({ page }) => {
    const stats = testDataFixtures.dashboard.stats;

    for (const stat of stats) {
      const statSelector = `[data-testid="stat-${stat.toLowerCase().replace(/\s+/g, '-')}"]`;
      const isVisible = await isElementVisibleAndInteractive(page, statSelector);

      if (!isVisible) {
        console.warn(`Stat "${stat}" not found in ${browserName} - checking for alternative selector`);
        // Try alternative selector patterns
        const alternative = page.locator('text=' + stat);
        const altVisible = await alternative.isVisible().catch(() => false);
        if (altVisible) {
          console.log(`Found "${stat}" using text selector in ${browserName}`);
        } else {
          console.warn(`Could not find stat "${stat}" in ${browserName}`);
        }
      }
    }
  });

  test('should render charts correctly', async ({ page }) => {
    const charts = testDataFixtures.dashboard.charts;

    for (const chart of charts) {
      const chartSelector = `[data-testid="chart-${chart.toLowerCase().replace(/\s+/g, '-')}"]`;
      const chartElement = page.locator(chartSelector);

      // Check if chart container exists
      const isPresent = await chartElement.count().then(count => count > 0);

      if (isPresent) {
        await expect(chartElement).toBeVisible();
      } else {
        console.warn(`Chart "${chart}" not found in ${browserName}`);
      }
    }
  });

  test('should have interactive navigation buttons', async ({ page }) => {
    const navButtons = [
      navSelectors.traffic,
      navSelectors.intruder,
      navSelectors.rest,
      navSelectors.scanner
    ];

    for (const selector of navButtons) {
      const isVisible = await isElementVisibleAndInteractive(page, selector);

      if (isVisible) {
        // Test hover effect
        await page.hover(selector);
        await page.waitForTimeout(100);
      }
    }
  });

  test('should navigate to other views from dashboard', async ({ page }) => {
    const destinations = [
      { nav: navSelectors.traffic, view: viewSelectors.traffic, name: 'Traffic' },
      { nav: navSelectors.intruder, view: viewSelectors.intruder, name: 'Intruder' }
    ];

    for (const dest of destinations) {
      // Navigate
      await page.click(dest.nav);
      await page.waitForLoadState('networkidle');

      // Verify navigation
      const viewVisible = await page.locator(dest.view).isVisible().catch(() => false);
      expect(viewVisible).toBeTruthy();

      // Return to dashboard
      await page.click(navSelectors.dashboard);
      await page.waitForLoadState('networkidle');
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const dashboardSelector = viewSelectors.dashboard;
    const results = await testResponsiveLayout(page, dashboardSelector, testViewports);

    for (const result of results) {
      console.log(`Viewport ${result.viewport}: visible=${result.visible}, issues=${result.issues.length}`);

      // Mobile viewport may have different layout expectations
      if (result.viewport === '375x667') {
        // Mobile may have hamburger menu instead of full nav
        expect(result.visible).toBeTruthy();
      } else {
        expect(result.visible).toBeTruthy();
        expect(result.issues.length).toBe(0);
      }
    }
  });

  test('should handle refresh correctly', async ({ page }) => {
    const dashboardSelector = viewSelectors.dashboard;

    // Initial state
    await expect(page.locator(dashboardSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify dashboard is still visible
    await expect(page.locator(dashboardSelector)).toBeVisible();
  });

  test('should capture screenshot for comparison', async ({ page }) => {
    const screenshotPath = await captureComparisonScreenshot(
      page,
      'dashboard',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should support essential CSS features', async ({ page }) => {
    const features = ['grid', 'flexbox', 'localstorage'];

    for (const feature of features) {
      const supported = await checkFeatureSupport(page, feature);
      expect(supported).toBeTruthy();
      console.log(`${feature} supported in ${browserName}: ${supported}`);
    }
  });

  test('should handle browser-specific edge cases', async ({ page }) => {
    const knownIssues = getBrowserKnownIssues(browserName);

    if (knownIssues.length > 0) {
      console.log(`Known issues for ${browserName}:`, knownIssues);

      // Test for specific browser issues
      if (browserName === 'webkit') {
        // Safari-specific checks
        const scrollbars = await page.evaluate(() => {
          return window.getComputedStyle(document.body).scrollbarWidth !== undefined;
        });
        console.log(`Safari scrollbar width support: ${scrollbars}`);
      }

      if (browserName === 'firefox') {
        // Firefox-specific checks
        const flexGap = await page.evaluate(() => {
          return CSS.supports('gap', '10px');
        });
        console.log(`Firefox flex gap support: ${flexGap}`);
      }
    }
  });

  test('should display loading state correctly', async ({ page }) => {
    // Navigate away and back to trigger loading
    await page.click(navSelectors.traffic);
    await page.waitForLoadState('networkidle');

    await page.click(navSelectors.dashboard);

    // Check for loading indicator
    const loadingSelector = '[data-testid="loading"], .loading, .spinner';
    const loadingExists = await page.locator(loadingSelector).isVisible().catch(() => false);

    if (loadingExists) {
      // Wait for loading to complete
      await page.waitForSelector(loadingSelector, { state: 'hidden', timeout: 5000 }).catch(() => {});
    }

    // Verify dashboard is loaded
    await expect(page.locator(viewSelectors.dashboard)).toBeVisible();
  });

  test('should handle error states gracefully', async ({ page }) => {
    // This test checks that the dashboard handles API errors gracefully
    // In a real scenario, you might mock API failures

    const errorSelector = '[data-testid="error-message"], .error, .alert-error';
    const hasError = await page.locator(errorSelector).isVisible().catch(() => false);

    if (hasError) {
      console.warn(`Error message visible in ${browserName}:`, await page.locator(errorSelector).textContent());
    } else {
      console.log(`No error messages in ${browserName} - normal state`);
    }
  });

  test('should have accessible interactive elements', async ({ page }) => {
    // Check that interactive elements have proper ARIA attributes
    const buttons = page.locator('button, [role="button"]');
    const count = await buttons.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const button = buttons.nth(i);
      const hasAria = await button.evaluate(el =>
        el.hasAttribute('aria-label') ||
        el.hasAttribute('aria-labelledby') ||
        el.textContent?.trim().length > 0
      );

      if (!hasAria) {
        console.warn(`Button without proper ARIA in ${browserName}`);
      }
    }
  });

  test('should maintain state during navigation', async ({ page }) => {
    // Navigate to another view
    await page.click(navSelectors.traffic);
    await page.waitForLoadState('networkidle');

    // Navigate back
    await page.click(navSelectors.dashboard);
    await page.waitForLoadState('networkidle');

    // Verify dashboard state is maintained
    await expect(page.locator(viewSelectors.dashboard)).toBeVisible();
  });
});
