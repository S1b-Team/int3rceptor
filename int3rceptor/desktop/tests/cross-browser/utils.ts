import { Page, Browser, BrowserContext } from '@playwright/test';

/**
 * Cross-browser testing utilities for INT3RCEPTOR UI
 * Provides helper functions for browser detection, version reporting,
 * screenshot capture, and compatibility testing
 */

export interface BrowserInfo {
  name: string;
  version: string;
  userAgent: string;
  platform: string;
  vendor: string;
}

export interface TestResult {
  browser: string;
  test: string;
  status: 'passed' | 'failed' | 'skipped';
  duration: number;
  error?: string;
  screenshot?: string;
}

export interface CompatibilityReport {
  timestamp: string;
  browsers: BrowserInfo[];
  results: TestResult[];
  summary: {
    total: number;
    passed: number;
    failed: number;
    skipped: number;
  };
}

/**
 * Get browser information including version and user agent
 */
export async function getBrowserInfo(page: Page): Promise<BrowserInfo> {
  const browserName = page.context().browser()?.browserType().name() || 'unknown';
  const userAgent = await page.evaluate(() => navigator.userAgent);
  const platform = await page.evaluate(() => navigator.platform);
  const vendor = await page.evaluate(() => navigator.vendor);

  // Extract version from user agent
  let version = 'unknown';
  if (userAgent.includes('Chrome/') || userAgent.includes('Chromium/')) {
    const match = userAgent.match(/(?:Chrome|Chromium)\/(\d+\.\d+\.\d+\.\d+)/);
    version = match ? match[1] : 'unknown';
  } else if (userAgent.includes('Firefox/')) {
    const match = userAgent.match(/Firefox\/(\d+\.\d+)/);
    version = match ? match[1] : 'unknown';
  } else if (userAgent.includes('Safari/') && !userAgent.includes('Chrome')) {
    const match = userAgent.match(/Version\/(\d+\.\d+)/);
    version = match ? match[1] : 'unknown';
  } else if (userAgent.includes('Edg/')) {
    const match = userAgent.match(/Edg\/(\d+\.\d+\.\d+\.\d+)/);
    version = match ? match[1] : 'unknown';
  }

  return {
    name: browserName,
    version,
    userAgent,
    platform,
    vendor
  };
}

/**
 * Capture screenshot for browser comparison
 */
export async function captureComparisonScreenshot(
  page: Page,
  testName: string,
  browserName: string
): Promise<string> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `screenshots/${testName}-${browserName}-${timestamp}.png`;

  await page.screenshot({
    path: `tests/cross-browser/${filename}`,
    fullPage: true
  });

  return filename;
}

/**
 * Check if element is visible and interactive
 */
export async function isElementVisibleAndInteractive(
  page: Page,
  selector: string
): Promise<boolean> {
  try {
    const isVisible = await page.isVisible(selector);
    const isEnabled = await page.isEnabled(selector);
    return isVisible && isEnabled;
  } catch {
    return false;
  }
}

/**
 * Wait for element to be visible and interactive
 */
export async function waitForElementInteractive(
  page: Page,
  selector: string,
  timeout: number = 5000
): Promise<void> {
  await page.waitForSelector(selector, {
    state: 'visible',
    timeout
  });
  await page.waitForFunction(
    (sel) => {
      const el = document.querySelector(sel);
      return el && !(el as HTMLElement).disabled;
    },
    selector,
    { timeout }
  );
}

/**
 * Navigate to view and verify load
 */
export async function navigateToView(
  page: Page,
  viewName: string,
  expectedSelector: string
): Promise<boolean> {
  try {
    await page.click(`[data-testid="nav-${viewName}"]`);
    await page.waitForURL(new RegExp(`/${viewName}`));
    await page.waitForSelector(expectedSelector, { state: 'visible' });
    return true;
  } catch (error) {
    console.error(`Failed to navigate to ${viewName}:`, error);
    return false;
  }
}

/**
 * Test interactive element (button, link, input)
 */
export async function testInteractiveElement(
  page: Page,
  selector: string,
  action: 'click' | 'type' | 'focus' = 'click',
  value?: string
): Promise<boolean> {
  try {
    await waitForElementInteractive(page, selector);

    switch (action) {
      case 'click':
        await page.click(selector);
        break;
      case 'type':
        await page.fill(selector, value || '');
        break;
      case 'focus':
        await page.focus(selector);
        break;
    }

    return true;
  } catch (error) {
    console.error(`Failed to interact with ${selector}:`, error);
    return false;
  }
}

/**
 * Test table rendering and data
 */
export async function testTableRendering(
  page: Page,
  tableSelector: string
): Promise<{ rendered: boolean; rowCount: number; hasHeaders: boolean }> {
  try {
    const table = page.locator(tableSelector);
    const isRendered = await table.isVisible();

    if (!isRendered) {
      return { rendered: false, rowCount: 0, hasHeaders: false };
    }

    const rowCount = await table.locator('tbody tr').count();
    const hasHeaders = await table.locator('thead').count() > 0;

    return { rendered: true, rowCount, hasHeaders };
  } catch (error) {
    console.error(`Failed to test table ${tableSelector}:`, error);
    return { rendered: false, rowCount: 0, hasHeaders: false };
  }
}

/**
 * Test responsive layout at different viewports
 */
export async function testResponsiveLayout(
  page: Page,
  testSelector: string,
  viewports: { width: number; height: number }[]
): Promise<{ viewport: string; visible: boolean; issues: string[] }[]> {
  const results = [];

  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    await page.waitForTimeout(500); // Wait for layout to adjust

    const isVisible = await page.isVisible(testSelector);
    const issues: string[] = [];

    if (!isVisible) {
      issues.push('Element not visible at this viewport');
    }

    // Check for horizontal scroll
    const hasHorizontalScroll = await page.evaluate(() => {
      return document.body.scrollWidth > document.body.clientWidth;
    });

    if (hasHorizontalScroll) {
      issues.push('Horizontal scroll detected');
    }

    results.push({
      viewport: `${viewport.width}x${viewport.height}`,
      visible: isVisible,
      issues
    });
  }

  return results;
}

/**
 * Test form input and validation
 */
export async function testFormInput(
  page: Page,
  formSelector: string,
  inputData: Record<string, string>
): Promise<{ success: boolean; errors: string[] }> {
  const errors: string[] = [];

  try {
    for (const [field, value] of Object.entries(inputData)) {
      const selector = `${formSelector} [name="${field}"], ${formSelector} [data-testid="${field}"]`;
      const success = await testInteractiveElement(page, selector, 'type', value);

      if (!success) {
        errors.push(`Failed to fill field: ${field}`);
      }
    }

    return { success: errors.length === 0, errors };
  } catch (error) {
    errors.push(`Form test failed: ${error}`);
    return { success: false, errors };
  }
}

/**
 * Test tab navigation
 */
export async function testTabNavigation(
  page: Page,
  tabSelector: string,
  expectedContentSelector: string
): Promise<boolean> {
  try {
    await page.click(tabSelector);
    await page.waitForSelector(expectedContentSelector, { state: 'visible' });
    return true;
  } catch (error) {
    console.error(`Failed to test tab ${tabSelector}:`, error);
    return false;
  }
}

/**
 * Generate compatibility report
 */
export function generateCompatibilityReport(
  browserInfos: BrowserInfo[],
  results: TestResult[]
): CompatibilityReport {
  const summary = {
    total: results.length,
    passed: results.filter(r => r.status === 'passed').length,
    failed: results.filter(r => r.status === 'failed').length,
    skipped: results.filter(r => r.status === 'skipped').length
  };

  return {
    timestamp: new Date().toISOString(),
    browsers: browserInfos,
    results,
    summary
  };
}

/**
 * Save compatibility report to JSON file
 */
export async function saveCompatibilityReport(
  report: CompatibilityReport,
  filename: string
): Promise<void> {
  const fs = await import('fs');
  const path = await import('path');

  const reportPath = path.join(
    __dirname,
    'reports',
    filename
  );

  // Ensure reports directory exists
  const reportsDir = path.dirname(reportPath);
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
}

/**
 * Get browser-specific known issues
 */
export function getBrowserKnownIssues(browserName: string): string[] {
  const knownIssues: Record<string, string[]> = {
    chromium: [
      'Some CSS grid layouts may have minor rendering differences',
      'Scroll behavior may vary from other browsers'
    ],
    firefox: [
      'Flexbox gap property not fully supported in older versions',
      'Some CSS transitions may have timing differences'
    ],
    webkit: [
      'Form input styling may differ from other browsers',
      'Scrollbar styling limited',
      'Some CSS filters may not render identically'
    ],
    edge: [
      'Similar to Chromium but may have minor rendering differences',
      'Some Microsoft-specific APIs may behave differently'
    ]
  };

  return knownIssues[browserName] || [];
}

/**
 * Check browser feature support
 */
export async function checkFeatureSupport(
  page: Page,
  feature: string
): Promise<boolean> {
  const featureChecks: Record<string, string> = {
    'grid': 'CSS.supports("display", "grid")',
    'flexbox': 'CSS.supports("display", "flex")',
    'webgl': '!!window.WebGLRenderingContext',
    'webgl2': '!!window.WebGL2RenderingContext',
    'websockets': '!!window.WebSocket',
    'fetch': '!!window.fetch',
    'localstorage': '!!window.localStorage',
    'sessionstorage': '!!window.sessionStorage'
  };

  const check = featureChecks[feature.toLowerCase()];
  if (!check) {
    throw new Error(`Unknown feature: ${feature}`);
  }

  return page.evaluate(check);
}

/**
 * Test data fixtures for consistent testing across browsers
 */
export const testDataFixtures = {
  dashboard: {
    title: 'Dashboard',
    stats: ['Total Requests', 'Active Connections', 'Errors', 'Warnings'],
    charts: ['Traffic Chart', 'Error Distribution']
  },
  traffic: {
    tableHeaders: ['Method', 'URL', 'Status', 'Size', 'Time'],
    sampleData: {
      method: 'GET',
      url: 'https://example.com/api/test',
      status: '200',
      size: '1.2KB',
      time: '45ms'
    }
  },
  intruder: {
    tabs: ['Target', 'Positions', 'Payloads', 'Attack'],
    buttons: ['Start Attack', 'Stop Attack', 'Clear']
  },
  rest: {
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
    fields: ['URL', 'Headers', 'Body', 'Auth']
  },
  scanner: {
    options: ['Active Scan', 'Passive Scan', 'Custom Scan'],
    settings: ['Scan Depth', 'Request Delay', 'Thread Count']
  },
  settings: {
    categories: ['General', 'Proxy', 'Security', 'Appearance'],
    fields: ['Port', 'Host', 'Timeout', 'Logging']
  },
  plugins: {
    listItems: ['Plugin Manager', 'Installed Plugins', 'Available Plugins'],
    actions: ['Install', 'Uninstall', 'Update', 'Configure']
  },
  decoder: {
    formats: ['Base64', 'URL', 'Hex', 'JWT'],
    actions: ['Encode', 'Decode', 'Format']
  },
  comparer: {
    panels: ['Left Panel', 'Right Panel', 'Diff View'],
    actions: ['Compare', 'Clear', 'Export']
  },
  websocket: {
    sections: ['Connection', 'Messages', 'History'],
    actions: ['Connect', 'Disconnect', 'Send', 'Clear']
  }
};

/**
 * Viewport sizes for responsive testing
 */
export const testViewports = [
  { width: 1920, height: 1080 }, // Desktop
  { width: 1366, height: 768 },  // Laptop
  { width: 768, height: 1024 },  // Tablet
  { width: 375, height: 667 }    // Mobile
];

/**
 * Standard view selectors
 */
export const viewSelectors = {
  dashboard: '[data-testid="dashboard-view"]',
  traffic: '[data-testid="traffic-view"]',
  intruder: '[data-testid="intruder-view"]',
  rest: '[data-testid="rest-view"]',
  scanner: '[data-testid="scanner-view"]',
  settings: '[data-testid="settings-view"]',
  plugins: '[data-testid="plugins-view"]',
  decoder: '[data-testid="decoder-view"]',
  comparer: '[data-testid="comparer-view"]',
  websocket: '[data-testid="websocket-view"]'
};

/**
 * Navigation selectors
 */
export const navSelectors = {
  dashboard: '[data-testid="nav-dashboard"]',
  traffic: '[data-testid="nav-traffic"]',
  intruder: '[data-testid="nav-intruder"]',
  rest: '[data-testid="nav-rest"]',
  scanner: '[data-testid="nav-scanner"]',
  settings: '[data-testid="nav-settings"]',
  plugins: '[data-testid="nav-plugins"]',
  decoder: '[data-testid="nav-decoder"]',
  comparer: '[data-testid="nav-comparer"]',
  websocket: '[data-testid="nav-websocket"]'
};
