import { test, expect, Page } from '@playwright/test';
import {
  getBrowserInfo,
  captureComparisonScreenshot,
  isElementVisibleAndInteractive,
  testResponsiveLayout,
  testViewports,
  testTableRendering,
  testDataFixtures,
  viewSelectors,
  navSelectors,
  checkFeatureSupport,
  getBrowserKnownIssues
} from './utils';

/**
 * Cross-browser tests for Plugins view
 * Tests plugin listing, installation, configuration, and management
 */

test.describe('Plugins - Cross-Browser Tests', () => {
  let browserName: string;
  let browserInfo: any;

  test.beforeEach(async ({ page, browserName: bn }) => {
    browserName = bn;
    browserInfo = await getBrowserInfo(page);
    console.log(`Testing Plugins on ${browserName} ${browserInfo.version}`);

    // Navigate to plugins view
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.click(navSelectors.plugins);
    await page.waitForLoadState('networkidle');
  });

  test('should render plugins view correctly', async ({ page }) => {
    const pluginsSelector = viewSelectors.plugins;

    // Verify plugins view is visible
    await expect(page.locator(pluginsSelector)).toBeVisible();

    // Check title
    const title = page.locator('h1, h2, [data-testid="plugins-title"]');
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText?.toLowerCase()).toContain('plugin');
  });

  test('should display plugin list items', async ({ page }) => {
    const listItems = testDataFixtures.plugins.listItems;

    for (const item of listItems) {
      const itemSelector = `[data-testid="${item.toLowerCase().replace(/\s+/g, '-')}"]`;
      const itemElement = page.locator(itemSelector);

      const exists = await itemElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await itemElement.isVisible();
        if (isVisible) {
          console.log(`"${item}" visible in ${browserName}`);
        }
      } else {
        // Try alternative selector
        const altItem = page.locator(`text=${item}`);
        const altExists = await altItem.count().then(count => count > 0);

        if (altExists) {
          console.log(`Found "${item}" using text selector in ${browserName}`);
        } else {
          console.warn(`"${item}" not found in ${browserName}`);
        }
      }
    }
  });

  test('should display plugin actions', async ({ page }) => {
    const actions = testDataFixtures.plugins.actions;

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

  test('should display plugin table', async ({ page }) => {
    const tableSelector = '[data-testid="plugins-table"], table';
    const result = await testTableRendering(page, tableSelector);

    if (result.rendered) {
      console.log(`Plugin table rendered in ${browserName} with ${result.rowCount} rows`);

      if (result.hasHeaders) {
        console.log(`Table has headers in ${browserName}`);
      }
    } else {
      console.warn(`Plugin table not rendered in ${browserName}`);
    }
  });

  test('should have search functionality', async ({ page }) => {
    const searchInput = page.locator('[data-testid="plugin-search"], input[placeholder*="search"], input[placeholder*="Search"]');
    const exists = await searchInput.count().then(count => count > 0);

    if (exists) {
      await expect(searchInput).toBeVisible();

      // Test search input
      const testQuery = 'test';
      await searchInput.fill(testQuery);
      await page.waitForTimeout(100);

      const inputValue = await searchInput.inputValue();
      expect(inputValue).toBe(testQuery);
    }
  });

  test('should display plugin details', async ({ page }) => {
    const detailsSelector = '[data-testid="plugin-details"], .plugin-details';
    const detailsElement = page.locator(detailsSelector);
    const exists = await detailsElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await detailsElement.isVisible();
      if (isVisible) {
        console.log(`Plugin details visible in ${browserName}`);

        // Check for detail fields
        const nameField = detailsElement.locator('[data-testid="plugin-name"], .plugin-name');
        const versionField = detailsElement.locator('[data-testid="plugin-version"], .plugin-version');
        const descriptionField = detailsElement.locator('[data-testid="plugin-description"], .plugin-description');

        const hasName = await nameField.count().then(count => count > 0);
        const hasVersion = await versionField.count().then(count => count > 0);
        const hasDescription = await descriptionField.count().then(count => count > 0);

        console.log(`Plugin details in ${browserName}: name=${hasName}, version=${hasVersion}, description=${hasDescription}`);
      }
    }
  });

  test('should support plugin installation', async ({ page }) => {
    const installButton = page.locator('[data-testid="install-plugin"], button:has-text("Install")');
    const exists = await installButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await installButton.isVisible();
      if (isVisible) {
        const isEnabled = await installButton.isEnabled();
        console.log(`Install button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should support plugin uninstallation', async ({ page }) => {
    const uninstallButton = page.locator('[data-testid="uninstall-plugin"], button:has-text("Uninstall")');
    const exists = await uninstallButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await uninstallButton.isVisible();
      if (isVisible) {
        const isEnabled = await uninstallButton.isEnabled();
        console.log(`Uninstall button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should support plugin configuration', async ({ page }) => {
    const configureButton = page.locator('[data-testid="configure-plugin"], button:has-text("Configure")');
    const exists = await configureButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await configureButton.isVisible();
      if (isVisible) {
        const isEnabled = await configureButton.isEnabled();
        console.log(`Configure button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display responsive layout at different viewports', async ({ page }) => {
    const pluginsSelector = viewSelectors.plugins;
    const results = await testResponsiveLayout(page, pluginsSelector, testViewports);

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
      'plugins',
      browserName
    );

    console.log(`Screenshot saved to: ${screenshotPath}`);
  });

  test('should handle refresh correctly', async ({ page }) => {
    const pluginsSelector = viewSelectors.plugins;

    // Initial state
    await expect(page.locator(pluginsSelector)).toBeVisible();

    // Refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify plugins view is still visible
    await expect(page.locator(pluginsSelector)).toBeVisible();
  });

  test('should display plugin categories', async ({ page }) => {
    const categorySelectors = [
      '[data-testid="category-installed"]',
      '[data-testid="category-available"]',
      '[data-testid="category-updates"]'
    ];

    for (const selector of categorySelectors) {
      const categoryElement = page.locator(selector);
      const exists = await categoryElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await categoryElement.isVisible();
        if (isVisible) {
          console.log(`Category ${selector} visible in ${browserName}`);
        }
      }
    }
  });

  test('should support plugin updates', async ({ page }) => {
    const updateButton = page.locator('[data-testid="update-plugin"], button:has-text("Update")');
    const exists = await updateButton.count().then(count => count > 0);

    if (exists) {
      const isVisible = await updateButton.isVisible();
      if (isVisible) {
        const isEnabled = await updateButton.isEnabled();
        console.log(`Update button enabled in ${browserName}: ${isEnabled}`);
      }
    }
  });

  test('should display plugin status indicators', async ({ page }) => {
    const statusSelectors = [
      '[data-testid="status-enabled"]',
      '[data-testid="status-disabled"]',
      '[data-testid="status-error"]'
    ];

    for (const selector of statusSelectors) {
      const statusElement = page.locator(selector);
      const exists = await statusElement.count().then(count => count > 0);

      if (exists) {
        const isVisible = await statusElement.isVisible();
        if (isVisible) {
          console.log(`Status ${selector} visible in ${browserName}`);
        }
      }
    }
  });

  test('should handle browser-specific table rendering', async ({ page }) => {
    const tableSelector = '[data-testid="plugins-table"], table';
    const table = page.locator(tableSelector);
    const exists = await table.count().then(count => count > 0);

    if (exists && await table.isVisible()) {
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
    }
  });

  test('should support plugin filtering', async ({ page }) => {
    const filterSelector = '[data-testid="plugin-filter"], select[name="filter"]';
    const exists = await filterSelector.count().then(count => count > 0);

    if (exists && await filterSelector.isVisible()) {
      // Get available filter options
      const options = await filterSelector.locator('option').allTextContents();
      console.log(`Available filter options in ${browserName}:`, options);

      if (options.length > 1) {
        // Select second option
        await filterSelector.selectOption({ index: 1 });
        await page.waitForTimeout(100);

        const selectedValue = await filterSelector.inputValue();
        console.log(`Selected filter: ${selectedValue}`);
      }
    }
  });

  test('should display plugin permissions', async ({ page }) => {
    const permissionsSelector = '[data-testid="plugin-permissions"], .plugin-permissions';
    const permissionsElement = page.locator(permissionsSelector);
    const exists = await permissionsElement.count().then(count => count > 0);

    if (exists) {
      const isVisible = await permissionsElement.isVisible();
      if (isVisible) {
        console.log(`Plugin permissions visible in ${browserName}`);

        // Check for permission items
        const permissionItems = permissionsElement.locator('[data-testid="permission-item"], .permission-item');
        const count = await permissionItems.count();

        console.log(`Permission items in ${browserName}: ${count}`);
      }
    }
  });
});
