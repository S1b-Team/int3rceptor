import { expect, Page, TestInfo } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { PNG } from 'pngjs';
import { match as pixelmatch } from 'pixelmatch';

/**
 * Visual Regression Testing Utilities
 * Provides helper functions for screenshot capture, comparison, and baseline management
 */

export interface VisualTestOptions {
  threshold?: number;
  maxDiffPixels?: number;
  fullPage?: boolean;
  animations?: 'disabled' | 'allowed';
}

export interface ComparisonResult {
  passed: boolean;
  diffPixels: number;
  diffPercentage: number;
  diffImagePath?: string;
  baselinePath?: string;
  actualPath?: string;
}

export interface MockupMapping {
  view: string;
  mockupPath: string;
  route: string;
}

/**
 * Default configuration for visual tests
 */
const DEFAULT_CONFIG: VisualTestOptions = {
  threshold: 0.1, // 0.1% pixel difference tolerance
  maxDiffPixels: 100,
  fullPage: true,
  animations: 'disabled'
};

/**
 * Load mockup mappings from configuration file
 */
export function loadMockupMappings(): MockupMapping[] {
  const mockupsPath = path.join(process.cwd(), 'tests', 'visual', 'mockups.json');

  if (!fs.existsSync(mockupsPath)) {
    return [];
  }

  try {
    const content = fs.readFileSync(mockupsPath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    console.error('Failed to load mockup mappings:', error);
    return [];
  }
}

/**
 * Get mockup path for a specific view
 */
export function getMockupPath(view: string): string | null {
  const mappings = loadMockupMappings();
  const mapping = mappings.find(m => m.view.toLowerCase() === view.toLowerCase());
  return mapping?.mockupPath || null;
}

/**
 * Disable animations for consistent screenshots
 */
export async function disableAnimations(page: Page): Promise<void> {
  await page.addInitScript(() => {
    // Disable CSS transitions
    const style = document.createElement('style');
    style.textContent = `
      *, *::before, *::after {
        transition: none !important;
        animation: none !important;
      }
    `;
    document.head.appendChild(style);

    // Disable requestAnimationFrame
    (window as any).requestAnimationFrame = (callback: FrameRequestCallback) => {
      return setTimeout(() => callback(Date.now()), 0) as unknown as number;
    };
  });
}

/**
 * Wait for page to be stable (no network requests, no animations)
 */
export async function waitForPageStability(page: Page, timeout: number = 5000): Promise<void> {
  await page.waitForLoadState('networkidle', { timeout });
  await page.waitForTimeout(500); // Additional buffer for rendering
}

/**
 * Capture screenshot with options
 */
export async function captureScreenshot(
  page: Page,
  testInfo: TestInfo,
  options: VisualTestOptions = {}
): Promise<Buffer> {
  const config = { ...DEFAULT_CONFIG, ...options };

  if (config.animations === 'disabled') {
    await disableAnimations(page);
  }

  await waitForPageStability(page);

  const screenshot = await page.screenshot({
    fullPage: config.fullPage,
    animations: 'disabled'
  });

  return screenshot;
}

/**
 * Ensure baseline directory exists
 */
export function ensureBaselineDir(testInfo: TestInfo): string {
  const baselineDir = path.join(
    testInfo.project.outputDir,
    'baselines',
    testInfo.titlePath[0]
  );

  if (!fs.existsSync(baselineDir)) {
    fs.mkdirSync(baselineDir, { recursive: true });
  }

  return baselineDir;
}

/**
 * Ensure actuals directory exists
 */
export function ensureActualsDir(testInfo: TestInfo): string {
  const actualsDir = path.join(
    testInfo.project.outputDir,
    'actuals',
    testInfo.titlePath[0]
  );

  if (!fs.existsSync(actualsDir)) {
    fs.mkdirSync(actualsDir, { recursive: true });
  }

  return actualsDir;
}

/**
 * Ensure diffs directory exists
 */
export function ensureDiffsDir(testInfo: TestInfo): string {
  const diffsDir = path.join(
    testInfo.project.outputDir,
    'diffs',
    testInfo.titlePath[0]
  );

  if (!fs.existsSync(diffsDir)) {
    fs.mkdirSync(diffsDir, { recursive: true });
  }

  return diffsDir;
}

/**
 * Save baseline screenshot
 */
export function saveBaseline(
  testInfo: TestInfo,
  screenshot: Buffer,
  filename: string
): string {
  const baselineDir = ensureBaselineDir(testInfo);
  const baselinePath = path.join(baselineDir, filename);
  fs.writeFileSync(baselinePath, screenshot);
  return baselinePath;
}

/**
 * Save actual screenshot
 */
export function saveActual(
  testInfo: TestInfo,
  screenshot: Buffer,
  filename: string
): string {
  const actualsDir = ensureActualsDir(testInfo);
  const actualPath = path.join(actualsDir, filename);
  fs.writeFileSync(actualPath, screenshot);
  return actualPath;
}

/**
 * Compare two PNG images using pixelmatch
 */
export function compareImages(
  img1: PNG,
  img2: PNG,
  options: VisualTestOptions = {}
): ComparisonResult {
  const config = { ...DEFAULT_CONFIG, ...options };

  const width = Math.max(img1.width, img2.width);
  const height = Math.max(img1.height, img2.height);

  // Create diff image
  const diff = new PNG({ width, height });

  // Compare images
  const diffPixels = pixelmatch(
    img1.data,
    img2.data,
    diff.data,
    width,
    height,
    {
      threshold: 0.1,
      alpha: 0.5,
      diffColor: [255, 0, 0],
      diffMask: false
    }
  );

  const totalPixels = width * height;
  const diffPercentage = (diffPixels / totalPixels) * 100;

  const passed = diffPercentage <= config.threshold && diffPixels <= (config.maxDiffPixels || Infinity);

  return {
    passed,
    diffPixels,
    diffPercentage,
  };
}

/**
 * Compare screenshot against baseline
 */
export async function compareWithBaseline(
  testInfo: TestInfo,
  actual: Buffer,
  filename: string,
  options: VisualTestOptions = {}
): Promise<ComparisonResult> {
  const config = { ...DEFAULT_CONFIG, ...options };
  const baselineDir = ensureBaselineDir(testInfo);
  const baselinePath = path.join(baselineDir, filename);

  // Save actual screenshot
  const actualPath = saveActual(testInfo, actual, filename);

  // Check if baseline exists
  if (!fs.existsSync(baselinePath)) {
    return {
      passed: false,
      diffPixels: 0,
      diffPercentage: 0,
      actualPath,
      baselinePath
    };
  }

  // Load images
  const baselineImg = PNG.sync.read(fs.readFileSync(baselinePath));
  const actualImg = PNG.sync.read(actual);

  // Compare images
  const result = compareImages(baselineImg, actualImg, config);
  result.baselinePath = baselinePath;
  result.actualPath = actualPath;

  // Create diff image if comparison failed
  if (!result.passed) {
    const diffsDir = ensureDiffsDir(testInfo);
    const diffPath = path.join(diffsDir, filename.replace('.png', '-diff.png'));

    const width = Math.max(baselineImg.width, actualImg.width);
    const height = Math.max(baselineImg.height, actualImg.height);
    const diff = new PNG({ width, height });

    pixelmatch(
      baselineImg.data,
      actualImg.data,
      diff.data,
      width,
      height,
      {
        threshold: 0.1,
        alpha: 0.5,
        diffColor: [255, 0, 0],
        diffMask: false
      }
    );

    fs.writeFileSync(diffPath, PNG.sync.write(diff));
    result.diffImagePath = diffPath;
  }

  return result;
}

/**
 * Compare screenshot against mockup
 */
export async function compareWithMockup(
  testInfo: TestInfo,
  actual: Buffer,
  view: string,
  options: VisualTestOptions = {}
): Promise<ComparisonResult> {
  const config = { ...DEFAULT_CONFIG, ...options };
  const mockupPath = getMockupPath(view);

  if (!mockupPath) {
    return {
      passed: false,
      diffPixels: 0,
      diffPercentage: 0,
      actualPath: 'N/A',
      baselinePath: 'No mockup found'
    };
  }

  const absoluteMockupPath = path.resolve(process.cwd(), mockupPath);

  if (!fs.existsSync(absoluteMockupPath)) {
    return {
      passed: false,
      diffPixels: 0,
      diffPercentage: 0,
      actualPath: 'N/A',
      baselinePath: absoluteMockupPath
    };
  }

  // Save actual screenshot
  const actualsDir = ensureActualsDir(testInfo);
  const filename = `${view}-mockup-comparison.png`;
  const actualPath = path.join(actualsDir, filename);
  fs.writeFileSync(actualPath, actual);

  // Load images
  const mockupImg = PNG.sync.read(fs.readFileSync(absoluteMockupPath));
  const actualImg = PNG.sync.read(actual);

  // Compare images
  const result = compareImages(mockupImg, actualImg, config);
  result.baselinePath = absoluteMockupPath;
  result.actualPath = actualPath;

  // Create diff image if comparison failed
  if (!result.passed) {
    const diffsDir = ensureDiffsDir(testInfo);
    const diffPath = path.join(diffsDir, `${view}-mockup-diff.png`);

    const width = Math.max(mockupImg.width, actualImg.width);
    const height = Math.max(mockupImg.height, actualImg.height);
    const diff = new PNG({ width, height });

    pixelmatch(
      mockupImg.data,
      actualImg.data,
      diff.data,
      width,
      height,
      {
        threshold: 0.1,
        alpha: 0.5,
        diffColor: [255, 0, 0],
        diffMask: false
      }
    );

    fs.writeFileSync(diffPath, PNG.sync.write(diff));
    result.diffImagePath = diffPath;
  }

  return result;
}

/**
 * Assert visual match with baseline
 */
export async function expectVisualMatch(
  page: Page,
  testInfo: TestInfo,
  filename: string,
  options: VisualTestOptions = {}
): Promise<void> {
  const screenshot = await captureScreenshot(page, testInfo, options);
  const result = await compareWithBaseline(testInfo, screenshot, filename, options);

  expect(result.passed, `
Visual regression test failed for ${filename}
Diff pixels: ${result.diffPixels}
Diff percentage: ${result.diffPercentage.toFixed(2)}%
Baseline: ${result.baselinePath}
Actual: ${result.actualPath}
Diff: ${result.diffImagePath || 'N/A'}
`).toBe(true);
}

/**
 * Assert visual match with mockup
 */
export async function expectMockupMatch(
  page: Page,
  testInfo: TestInfo,
  view: string,
  options: VisualTestOptions = {}
): Promise<void> {
  const screenshot = await captureScreenshot(page, testInfo, options);
  const result = await compareWithMockup(testInfo, screenshot, view, options);

  expect(result.passed, `
Mockup comparison failed for ${view}
Diff pixels: ${result.diffPixels}
Diff percentage: ${result.diffPercentage.toFixed(2)}%
Mockup: ${result.baselinePath}
Actual: ${result.actualPath}
Diff: ${result.diffImagePath || 'N/A'}
`).toBe(true);
}

/**
 * Update baseline for a test
 */
export async function updateBaseline(
  page: Page,
  testInfo: TestInfo,
  filename: string,
  options: VisualTestOptions = {}
): Promise<string> {
  const screenshot = await captureScreenshot(page, testInfo, options);
  return saveBaseline(testInfo, screenshot, filename);
}

/**
 * Generate diff report
 */
export function generateDiffReport(results: Map<string, ComparisonResult>): string {
  const report: any[] = [];

  for (const [testName, result] of results.entries()) {
    report.push({
      test: testName,
      passed: result.passed,
      diffPixels: result.diffPixels,
      diffPercentage: result.diffPercentage,
      baselinePath: result.baselinePath,
      actualPath: result.actualPath,
      diffImagePath: result.diffImagePath
    });
  }

  return JSON.stringify(report, null, 2);
}

/**
 * Save diff report to file
 */
export function saveDiffReport(testInfo: TestInfo, report: string): string {
  const reportsDir = path.join(testInfo.project.outputDir, 'reports');
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  const reportPath = path.join(reportsDir, 'diff-report.json');
  fs.writeFileSync(reportPath, report);
  return reportPath;
}
