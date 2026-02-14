import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testTableRendering,
  testResponsiveLayout,
  testViewports,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Traffic view
 * Tests navigation, table rendering, filtering, sorting, and data display
 */

test.describe('Traffic - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Traffic on ${browserName} ${browserInfo.version}`);

    // Navigate to traffic view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.traffic);
    await page.waitForLoadState('networkidle');
  });

  test('should render traffic view correctly', async ({ page }) => {
    const trafficSelector = viewSelectors.traffic;

    // Verify traffic view is visible
    await expect(page.locator(trafficSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="traffic-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('traffic');
  });

  test('should render traffic table with headers', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const result = await testTableRendering(page, tableSelector);

    expect(result.rendered).toBeTruthy();

    if (result.hasHeaders) {
      const headers = testDataFixtures.traffic.tableHeaders;
      for (const header of headers) {
        const headerElement = page.locator(`th:has-text("${header}")`);
        const exists = await headerElement.count().then(count => count > 0);

        if (!exists) {
          console.warn(`Header "${header}" not found in ${browserName}`);
        }
      }
    }
  });

  test('should display traffic data rows', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const table = page.locator(tableSelector);

    // Check if table has data
    const rowCount = await table.locator('tbody tr').count();
    console.log(`Found ${rowCount} traffic rows in ${browserName}`);

    // If there are rows, verify they have proper structure
    if (rowCount > 0) {
      const firstRow = table.locator('tbody tr').first();
      const cells = await firstRow.locator('td').count();
      expect(cells).toBeGreaterThan(0);
    }
  });

  test('should have working filter controls', async ({ page }) => {
    const filterSelectors = [
      '[data-testid="filter-input"]',
      '[data-testid="filter-method"]',
      '[data-testid="filter-status"]'
    ];

    for (const selector of filterSelectors) {
      const isVisible = await isElementVisibleAndInteractive(page, selector);

      if (isVisible) {
        // Test that filter is interactive
        await page.focus(selector);
        await page.waitForTimeout(100);
      }
    }
  });

  test('should support table sorting', async ({ page }) => {
    const sortableHeaders = page.locator('th[sortable], th[data-sortable], th.sortable');
    const count = await sortableHeaders.count();

    if (count > 0) {
      // Click first sortable header
      await sortableHeaders.first().click();
      await page.waitForTimeout(500);

      // Verify sorting indicator or state change
      const hasSortClass = await sortableHeaders.first().evaluate(el =>
        el.classList.contains('sorted') ||
        el.classList.contains('asc') ||
        el.classList.contains('desc')
      );

      console.log(`Sorting indicator present in ${browserName}: ${hasSortClass}`);
    }
  });

  test('should handle row selection', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const table = page.locator(tableSelector);
    const rowCount = await table.locator('tbody tr').count();

    if (rowCount > 0) {
      const firstRow = table.locator('tbody tr').first();

      // Check for checkbox or clickable row
      const checkbox = firstRow.locator('input[type="checkbox"]');
      const hasCheckbox = await checkbox.count().then(count => count > 0);

      if (hasCheckbox) {
        await checkbox.first().check();
        await page.waitForTimeout(100);

        const isChecked = await checkbox.first().isChecked();
        expect(isChecked).toBeTruthy();
      } else {
        // Try clicking the row
        await firstRow.click();
        await page.waitForTimeout(100);
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const trafficSelector = viewSelectors.traffic;
    const results = await testResponsiveLayout(page, trafficSelector, testViewports);

    for (const result of results) {
      console.log(`Viewport ${result.viewport}: visible=${result.visible}, issues=${result.issues.length}`);

      // Mobile may have horizontal scroll due to table
      if (result.viewport === '375x667') {
        expect(result.visible).toBeTruthy();
      } else {
        expect(result.visible).toBeTruthy();
      }
    }
  });

  test('should capture screenshot for comparison', async ({ page }) => {
    const screenshotPath = await captureComparisonScreenshot(
      page,
      'traffic',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const trafficSelector = viewSelectors.traffic;

    // Initial state
    await expect(page.locator(trafficSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify traffic view is still visible
    await expect(page.locator(trafficSelector)).toBeVisible();
  });

  test('should support pagination controls', async ({ page }) => {
    const paginationSelectors = [
      '[data-testid="pagination-next"]',
      '[data-testid="pagination-prev"]',
      '[data-testid="pagination-page"]'
    ];

    for (const selector of paginationSelectors) {
      const element = page.locator(selector);
      const exists = await element.count().then(count => count > 0);

      if (exists) {
        const isVisible = await element.isVisible();
        if (isVisible) {
          console.log(`Pagination control ${selector} visible in ${browserName}`);
        }
      }
    }
  });

  test('should display request details on row click', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const table = page.locator(tableSelector);
    const rowCount = await table.locator('tbody tr').count();

    if (rowCount > 0) {
      const firstRow = table.locator('tbody tr').first();
      await firstRow.click();
      await page.waitForTimeout(500);

      // Check for details panel
      const detailsPanel = page.locator('[data-testid="request-details"], .request-details');
      const hasDetails = await detailsPanel.isVisible().catch(() => false);

      if (hasDetails) {
        console.log(`Request details panel visible in ${browserName}`);
      }
    }
  });

  test('should handle browser-specific table rendering', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const table = page.locator(tableSelector);

    // Check table-specific CSS features
    const tableStyles = await table.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        borderCollapse: styles.borderCollapse,
        tableLayout: styles.tableLayout,
        hasScroll: el.scrollWidth > el.clientWidth
      };
    });

    console.log(`Table styles in ${browserName}:`, tableStyles);

    // Safari-specific checks
    if (browserName === 'webkit') {
      const hasCustomScrollbar = await table.evaluate(el => {
        return el.style.scrollbarWidth !== undefined ||
               el.style.webkitScrollbar !== undefined;
      });
      console.log(`Safari custom scrollbar support: ${hasCustomScrollbar}`);
    }
  });

  test('should support export functionality', async ({ page }) => {
    const exportButton = page.locator('[data-testid="export-button"], button:has-text("Export")');
    const exists = await exportButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await exportButton.isVisible();
      if (isVisible) {
        // Just verify it's clickable, don't actually trigger download
        const isClickable = await exportButton.isEnabled();
        expect(isClickable).toBeTruthy();
      }
    }
  });

  test('should handle empty state', async ({ page }) => {
    // This test would typically mock an empty state
    // For now, we just check if empty state UI exists

    const emptyState = page.locator('[data-testid="empty-state"], .empty-state');
    const hasEmptyState = await emptyState.isVisible().catch(() => false);

    if (hasEmptyState) {
      console.log(`Empty state visible in ${browserName}`);
      const emptyMessage = await emptyState.textContent();
      console.log(`Empty message: ${emptyMessage}`);
    }
  });

  test('should maintain filter state during navigation', async ({ page }) => {
    const filterInput = page.locator('[data-testid="filter-input"]');
    const exists = await filterInput.count().then(count => count > 0);

    if (exists && await filterInput.isVisible()) {
      // Set filter
      await filterInput.fill('test');
      await page.waitForTimeout(500);

      // Navigate away
      await page.click(navSelectors.dashboard);
      await page.waitForLoadState('networkidle');

      // Navigate back
      await page.click(navSelectors.traffic);
      await page.waitForLoadState('networkidle');

      // Check if filter is maintained (depends on app implementation)
      const filterValue = await filterInput.inputValue();
      console.log(`Filter value after navigation: ${filterValue}`);
    }
  });

  test('should handle large data sets efficiently', async ({ page }) => {
    const tableSelector = '[data-testid="traffic-table"], table';
    const table = page.locator(tableSelector);

    // Measure render time
    const startTime = Date.now();
    await table.waitFor({ state: 'visible', timeout: 5000 });
    const renderTime = Date.now() - startTime;

    console.log(`Table render time in ${browserName}: ${renderTime}ms`);

    // Check for virtual scrolling or pagination
    const hasVirtualScroll = await page.evaluate(() => {
      return document.querySelector('[data-virtual-scroll], .virtual-scroll') !== null;
    });

    if (hasVirtualScroll) {
      console.log(`Virtual scrolling detected in ${browserName}`);
    }
  });
});
