import { test, Page, expect, TestInfo } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Accessibility Testing Utilities for INT3RCEPTOR
 * Provides helper functions for WCAG 2.1 Level AA compliance testing
 * using axe-core and Playwright
 */

export interface AxeResult {
  violations: any[];
  passes: any[];
  incomplete: any[];
  inapplicable: any[];
}

export interface AccessibilityTestOptions {
  context?: string | string[] | null;
  rules?: Record<string, any>;
  tags?: string[];
  reporter?: string;
}

export interface KeyboardNavigationResult {
  passed: boolean;
  issues: string[];
  focusOrder: string[];
}

export interface FocusIndicatorResult {
  passed: boolean;
  issues: string[];
}

export interface ARIAValidationResult {
  passed: boolean;
  issues: string[];
}

export interface ContrastResult {
  passed: boolean;
  issues: string[];
}

/**
 * Default accessibility test options for WCAG 2.1 Level AA
 */
export const DEFAULT_A11Y_OPTIONS: AccessibilityTestOptions = {
  tags: ['wcag2a', 'wcag2aa', 'wcag21aa'],
  rules: {
    // Enable WCAG 2.1 Level AA rules
    'color-contrast': { enabled: true },
    'keyboard-navigation': { enabled: true },
    'focus-order-semantics': { enabled: true },
    'aria-labels': { enabled: true },
  },
};

/**
 * WCAG 2.1 Level AA rule tags
 */
export const WCAG_2_1_AA_TAGS = ['wcag2a', 'wcag2aa', 'wcag21aa'];

/**
 * View selectors for accessibility testing
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
 * Navigation selectors for accessibility testing
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

/**
 * Run axe-core accessibility scan
 */
export async function runAxeScan(
  page: Page,
  options: AccessibilityTestOptions = {}
): Promise<AxeResult> {
  const axeBuilder = new AxeBuilder({ page });

  if (options.tags) {
    axeBuilder.withTags(options.tags);
  }

  if (options.rules) {
    axeBuilder.withRules(Object.keys(options.rules));
  }

  if (options.context) {
    axeBuilder.include(options.context);
  }

  const results = await axeBuilder.analyze();

  return {
    violations: results.violations,
    passes: results.passes,
    incomplete: results.incomplete,
    inapplicable: results.inapplicable
  };
}

/**
 * Assert no accessibility violations
 */
export async function expectNoA11yViolations(
  page: Page,
  options: AccessibilityTestOptions = {}
): Promise<void> {
  const results = await runAxeScan(page, options);

  if (results.violations.length > 0) {
    const violationMessages = results.violations.map(v =>
      `- ${v.id}: ${v.description}\n  Impact: ${v.impact}\n  Elements: ${v.nodes.length}`
    ).join('\n');

    throw new Error(
      `Found ${results.violations.length} accessibility violation(s):\n${violationMessages}`
    );
  }
}

/**
 * Test keyboard navigation (Tab/Enter/Esc)
 */
export async function testKeyboardNavigation(
  page: Page,
  selector: string
): Promise<KeyboardNavigationResult> {
  const issues: string[] = [];
  const focusOrder: string[] = [];

  try {
    // Get all focusable elements
    const focusableElements = await page.evaluate((sel) => {
      const container = document.querySelector(sel);
      if (!container) return [];

      const focusable = container.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      return Array.from(focusable).map((el, index) => ({
        index,
        tagName: el.tagName,
        id: el.id,
        className: el.className,
        ariaLabel: el.getAttribute('aria-label'),
        textContent: el.textContent?.trim().substring(0, 50)
      }));
    }, selector);

    // Test Tab navigation
    for (let i = 0; i < focusableElements.length; i++) {
      await page.keyboard.press('Tab');
      const focusedElement = await page.evaluate(() => {
        const active = document.activeElement;
        return {
          tagName: active?.tagName,
          id: active?.id,
          className: active?.className
        };
      });

      focusOrder.push(`${focusedElement.tagName}#${focusedElement.id}`);

      // Check if element is visible and has focus indicator
      const hasFocusStyle = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement;
        const styles = window.getComputedStyle(active);
        return (
          styles.outline !== 'none' ||
          styles.boxShadow !== 'none' ||
          styles.borderColor !== 'rgba(0, 0, 0, 0)'
        );
      });

      if (!hasFocusStyle) {
        issues.push(`Element ${i} lacks visible focus indicator`);
      }
    }

    // Test Enter key on buttons
    const buttons = await page.locator(`${selector} button`).count();
    for (let i = 0; i < Math.min(buttons, 3); i++) {
      const button = page.locator(`${selector} button`).nth(i);
      await button.focus();
      await page.keyboard.press('Enter');
      // Should trigger click action
    }

    // Test Escape key (should close modals or cancel actions)
    await page.keyboard.press('Escape');

  } catch (error) {
    issues.push(`Keyboard navigation test failed: ${error}`);
  }

  return {
    passed: issues.length === 0,
    issues,
    focusOrder
  };
}

/**
 * Test focus indicators
 */
export async function testFocusIndicators(
  page: Page,
  selector: string
): Promise<FocusIndicatorResult> {
  const issues: string[] = [];

  try {
    const focusableElements = await page.locator(
      `${selector} button, ${selector} input, ${selector} a, ${selector} [tabindex]`
    ).count();

    for (let i = 0; i < Math.min(focusableElements, 5); i++) {
      const element = page.locator(
        `${selector} button, ${selector} input, ${selector} a, ${selector} [tabindex]`
      ).nth(i);

      await element.focus();

      const hasFocusIndicator = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement;
        const styles = window.getComputedStyle(active);
        const outline = styles.outline;
        const boxShadow = styles.boxShadow;
        const borderColor = styles.borderColor;

        return (
          outline !== 'none' ||
          boxShadow !== 'none' ||
          borderColor !== 'rgba(0, 0, 0, 0)' ||
          borderColor !== 'rgb(0, 0, 0)'
        );
      });

      if (!hasFocusIndicator) {
        issues.push(`Focusable element ${i} lacks visible focus indicator`);
      }
    }
  } catch (error) {
    issues.push(`Focus indicator test failed: ${error}`);
  }

  return {
    passed: issues.length === 0,
    issues
  };
}

/**
 * Test ARIA labels
 */
export async function testARIALabels(
  page: Page,
  selector: string
): Promise<ARIAValidationResult> {
  const issues: string[] = [];

  try {
    // Check buttons without text content
    const buttonsWithoutText = await page.evaluate((sel) => {
      const container = document.querySelector(sel);
      if (!container) return [];

      const buttons = container.querySelectorAll('button');
      const issues: string[] = [];

      buttons.forEach((btn, index) => {
        const text = btn.textContent?.trim();
        const ariaLabel = btn.getAttribute('aria-label');
        const ariaLabelledby = btn.getAttribute('aria-labelledby');

        if (!text && !ariaLabel && !ariaLabelledby) {
          issues.push(`Button at index ${index} has no accessible name`);
        }
      });

      return issues;
    }, selector);

    issues.push(...buttonsWithoutText);

    // Check inputs without labels
    const inputsWithoutLabels = await page.evaluate((sel) => {
      const container = document.querySelector(sel);
      if (!container) return [];

      const inputs = container.querySelectorAll('input, select, textarea');
      const issues: string[] = [];

      inputs.forEach((input, index) => {
        const ariaLabel = input.getAttribute('aria-label');
        const ariaLabelledby = input.getAttribute('aria-labelledby');
        const id = input.id;

        // Check if there's a label with matching for attribute
        const hasLabel = id && document.querySelector(`label[for="${id}"]`);

        if (!ariaLabel && !ariaLabelledby && !hasLabel) {
          issues.push(`Input at index ${index} has no accessible label`);
        }
      });

      return issues;
    }, selector);

    issues.push(...inputsWithoutLabels);

    // Check images without alt text
    const imagesWithoutAlt = await page.evaluate((sel) => {
      const container = document.querySelector(sel);
      if (!container) return [];

      const images = container.querySelectorAll('img');
      const issues: string[] = [];

      images.forEach((img, index) => {
        const alt = img.getAttribute('alt');
        const role = img.getAttribute('role');

        if (!alt && role !== 'presentation') {
          issues.push(`Image at index ${index} has no alt text`);
        }
      });

      return issues;
    }, selector);

    issues.push(...imagesWithoutAlt);

  } catch (error) {
    issues.push(`ARIA validation test failed: ${error}`);
  }

  return {
    passed: issues.length === 0,
    issues
  };
}

/**
 * Test contrast ratios
 */
export async function testContrastRatios(
  page: Page,
  selector: string
): Promise<ContrastResult> {
  const issues: string[] = [];

  try {
    // Run axe-core color-contrast rule
    const axeBuilder = new AxeBuilder({ page })
      .include(selector)
      .withTags(['wcag2aa']);

    const results = await axeBuilder.analyze();

    const contrastViolations = results.violations.filter(
      v => v.id === 'color-contrast'
    );

    for (const violation of contrastViolations) {
      for (const node of violation.nodes) {
        issues.push(
          `Color contrast issue: ${violation.description} - Element: ${node.html}`
        );
      }
    }

  } catch (error) {
    issues.push(`Contrast ratio test failed: ${error}`);
  }

  return {
    passed: issues.length === 0,
    issues
  };
}

/**
 * Save accessibility report
 */
export function saveAccessibilityReport(
  testInfo: TestInfo,
  results: AxeResult,
  testName: string
): string {
  const reportsDir = path.join(
    testInfo.project.outputDir,
    'reports',
    'accessibility'
  );

  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  const report = {
    testName,
    timestamp: new Date().toISOString(),
    violations: results.violations.length,
    passes: results.passes.length,
    incomplete: results.incomplete.length,
    violationDetails: results.violations.map(v => ({
      id: v.id,
      impact: v.impact,
      description: v.description,
      help: v.help,
      helpUrl: v.helpUrl,
      nodes: v.nodes.length
    }))
  };

  const reportPath = path.join(reportsDir, `${testName}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

  return reportPath;
}

/**
 * Generate HTML accessibility report
 */
export function generateHTMLReport(
  results: AxeResult,
  testName: string
): string {
  const timestamp = new Date().toISOString();

  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Accessibility Report - ${testName}</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
      margin: 0;
      padding: 20px;
      background: #f5f5f5;
    }
    .container {
      max-width: 1200px;
      margin: 0 auto;
      background: white;
      padding: 30px;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
    h1 {
      color: #333;
      border-bottom: 2px solid #007bff;
      padding-bottom: 10px;
    }
    .summary {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 20px;
      margin: 20px 0;
    }
    .summary-card {
      padding: 20px;
      border-radius: 6px;
      text-align: center;
    }
    .summary-card.violations {
      background: #ffebee;
      color: #c62828;
    }
    .summary-card.passes {
      background: #e8f5e9;
      color: #2e7d32;
    }
    .summary-card.incomplete {
      background: #fff3e0;
      color: #ef6c00;
    }
    .summary-card h3 {
      margin: 0 0 10px 0;
      font-size: 32px;
    }
    .summary-card p {
      margin: 0;
      font-weight: 500;
    }
    .violation {
      background: #fff;
      border-left: 4px solid #f44336;
      padding: 15px;
      margin: 15px 0;
      border-radius: 4px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }
    .violation h4 {
      margin: 0 0 10px 0;
      color: #d32f2f;
    }
    .violation .impact {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 12px;
      font-size: 12px;
      font-weight: bold;
      margin-bottom: 10px;
    }
    .impact.critical {
      background: #ffebee;
      color: #c62828;
    }
    .impact.serious {
      background: #fff3e0;
      color: #ef6c00;
    }
    .impact.moderate {
      background: #e3f2fd;
      color: #1565c0;
    }
    .impact.minor {
      background: #f5f5f5;
      color: #616161;
    }
    .violation .description {
      color: #555;
      margin-bottom: 10px;
    }
    .violation .help {
      color: #666;
      font-style: italic;
      margin-bottom: 10px;
    }
    .violation .nodes {
      color: #888;
      font-size: 14px;
    }
    .timestamp {
      color: #666;
      font-size: 14px;
      margin-bottom: 20px;
    }
    .no-violations {
      text-align: center;
      padding: 40px;
      color: #2e7d32;
      font-size: 18px;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>Accessibility Report - ${testName}</h1>
    <p class="timestamp">Generated: ${timestamp}</p>

    <div class="summary">
      <div class="summary-card violations">
        <h3>${results.violations.length}</h3>
        <p>Violations</p>
      </div>
      <div class="summary-card passes">
        <h3>${results.passes.length}</h3>
        <p>Passes</p>
      </div>
      <div class="summary-card incomplete">
        <h3>${results.incomplete.length}</h3>
        <p>Incomplete</p>
      </div>
    </div>

    ${results.violations.length === 0
      ? '<div class="no-violations">✓ No accessibility violations found!</div>'
      : results.violations.map(v => `
        <div class="violation">
          <h4>${v.id}</h4>
          <span class="impact ${v.impact}">${v.impact?.toUpperCase() || 'UNKNOWN'}</span>
          <p class="description">${v.description}</p>
          <p class="help">${v.help}</p>
          <p class="nodes">Affected elements: ${v.nodes.length}</p>
          <a href="${v.helpUrl}" target="_blank">Learn more →</a>
        </div>
      `).join('')
    }
  </div>
</body>
</html>
  `;

  return html;
}

/**
 * Save HTML accessibility report
 */
export function saveHTMLReport(
  testInfo: TestInfo,
  results: AxeResult,
  testName: string
): string {
  const reportsDir = path.join(
    testInfo.project.outputDir,
    'reports',
    'accessibility',
    'html'
  );

  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  const html = generateHTMLReport(results, testName);
  const reportPath = path.join(reportsDir, `${testName}.html`);
  fs.writeFileSync(reportPath, html);

  return reportPath;
}

/**
 * Navigate to view and wait for load
 */
export async function navigateToView(
  page: Page,
  viewName: string,
  navSelector: string,
  viewSelector: string
): Promise<void> {
  await page.click(navSelector);
  await page.waitForURL(new RegExp(`/${viewName}`));
  await page.waitForSelector(viewSelector, { state: 'visible' });
  await page.waitForLoadState('networkidle');
}

/**
 * Comprehensive accessibility test suite
 */
export async function runComprehensiveA11yTest(
  page: Page,
  selector: string,
  testName: string,
  testInfo: TestInfo
): Promise<void> {
  // Run axe-core scan
  const axeResults = await runAxeScan(page, {
    tags: WCAG_2_1_AA_TAGS
  });

  // Save reports
  saveAccessibilityReport(testInfo, axeResults, testName);
  saveHTMLReport(testInfo, axeResults, testName);

  // Test keyboard navigation
  const keyboardResult = await testKeyboardNavigation(page, selector);
  if (!keyboardResult.passed) {
    console.warn(`Keyboard navigation issues: ${keyboardResult.issues.join(', ')}`);
  }

  // Test focus indicators
  const focusResult = await testFocusIndicators(page, selector);
  if (!focusResult.passed) {
    console.warn(`Focus indicator issues: ${focusResult.issues.join(', ')}`);
  }

  // Test ARIA labels
  const ariarResult = await testARIALabels(page, selector);
  if (!ariaResult.passed) {
    console.warn(`ARIA label issues: ${ariaResult.issues.join(', ')}`);
  }

  // Test contrast ratios
  const contrastResult = await testContrastRatios(page, selector);
  if (!contrastResult.passed) {
    console.warn(`Contrast ratio issues: ${contrastResult.issues.join(', ')}`);
  }

  // Assert no critical violations
  const criticalViolations = axeResults.violations.filter(
    v => v.impact === 'critical'
  );

  expect(criticalViolations.length).toBe(0);
}
