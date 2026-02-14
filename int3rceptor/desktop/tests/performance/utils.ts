/**
 * Performance Testing Utilities
 *
 * This module provides helper functions for measuring and collecting
 * performance metrics in Vue 3 components and views.
 */

import { describe, expect, beforeAll, afterAll } from 'vitest';
import type { Metric } from 'web-vitals';

/**
 * Performance metric interface
 */
export interface PerformanceMetric {
  name: string;
  value: number;
  unit: string;
  timestamp: number;
}

/**
 * Extended performance result with comparison data
 */
export interface PerformanceResult {
  viewName: string;
  metrics: PerformanceMetric[];
  baseline?: PerformanceMetric[];
  comparison?: {
    improved: string[];
    degraded: string[];
    unchanged: string[];
  };
  status: 'pass' | 'fail' | 'warning';
  timestamp: string;
}

/**
 * Performance thresholds for different metrics
 */
export const PERFORMANCE_THRESHOLDS = {
  // Load time thresholds (in milliseconds)
  loadTime: {
    excellent: 1000,
    good: 2000,
    acceptable: 3000,
  },
  // Render time thresholds (in milliseconds)
  renderTime: {
    excellent: 100,
    good: 300,
    acceptable: 500,
  },
  // Interaction latency thresholds (in milliseconds)
  interactionLatency: {
    excellent: 50,
    good: 100,
    acceptable: 200,
  },
  // Memory usage thresholds (in MB)
  memoryUsage: {
    excellent: 50,
    good: 100,
    acceptable: 150,
  },
  // First Contentful Paint (in milliseconds)
  fcp: {
    excellent: 1000,
    good: 1800,
    acceptable: 3000,
  },
  // Largest Contentful Paint (in milliseconds)
  lcp: {
    excellent: 1200,
    good: 2500,
    acceptable: 4000,
  },
  // Cumulative Layout Shift (score)
  cls: {
    excellent: 0.1,
    good: 0.25,
    acceptable: 0.5,
  },
  // First Input Delay (in milliseconds)
  fid: {
    excellent: 50,
    good: 100,
    acceptable: 200,
  },
  // Time to Interactive (in milliseconds)
  tti: {
    excellent: 2000,
    good: 3800,
    acceptable: 7300,
  },
} as const;

/**
 * Performance measurement markers
 */
export class PerformanceMarker {
  private marks: Map<string, number> = new Map();

  /**
   * Create a performance mark
   */
  mark(name: string): void {
    this.marks.set(name, performance.now());
  }

  /**
   * Measure time between two marks
   */
  measure(name: string, startMark: string, endMark?: string): number {
    const start = this.marks.get(startMark);
    if (!start) {
      throw new Error(`Mark "${startMark}" not found`);
    }

    const end = endMark ? this.marks.get(endMark) : performance.now();
    if (!end) {
      throw new Error(`Mark "${endMark}" not found`);
    }

    return end - start;
  }

  /**
   * Get a specific mark value
   */
  getMark(name: string): number | undefined {
    return this.marks.get(name);
  }

  /**
   * Clear all marks
   */
  clear(): void {
    this.marks.clear();
  }

  /**
   * Clear a specific mark
   */
  clearMark(name: string): void {
    this.marks.delete(name);
  }
}

/**
 * Measure component render time
 */
export async function measureRenderTime(
  renderFn: () => Promise<void> | void
): Promise<number> {
  const marker = new PerformanceMarker();
  marker.mark('render-start');

  await renderFn();

  marker.mark('render-end');
  const renderTime = marker.measure('render', 'render-start', 'render-end');
  marker.clear();

  return renderTime;
}

/**
 * Measure interaction latency
 */
export async function measureInteractionLatency(
  interactionFn: () => Promise<void> | void
): Promise<number> {
  const marker = new PerformanceMarker();
  marker.mark('interaction-start');

  await interactionFn();

  marker.mark('interaction-end');
  const latency = marker.measure('interaction', 'interaction-start', 'interaction-end');
  marker.clear();

  return latency;
}

/**
 * Get current memory usage (if available)
 */
export function getMemoryUsage(): number {
  if (performance.memory) {
    return Math.round(performance.memory.usedJSHeapSize / 1048576); // Convert to MB
  }
  return 0;
}

/**
 * Collect Web Vitals metrics
 */
export function collectWebVitals(): Promise<PerformanceMetric[]> {
  return new Promise((resolve) => {
    const metrics: PerformanceMetric[] = [];

    // Dynamic import web-vitals to avoid issues in test environment
    import('web-vitals').then(({ onCLS, onFID, onFCP, onLCP, onTTFB }) => {
      const collectMetric = (name: string, metric: Metric) => {
        metrics.push({
          name,
          value: metric.value,
          unit: getUnitForMetric(name),
          timestamp: Date.now(),
        });
      };

      onCLS((metric) => collectMetric('CLS', metric));
      onFID((metric) => collectMetric('FID', metric));
      onFCP((metric) => collectMetric('FCP', metric));
      onLCP((metric) => collectMetric('LCP', metric));
      onTTFB((metric) => collectMetric('TTFB', metric));

      // Wait a bit for all metrics to be collected
      setTimeout(() => resolve(metrics), 2000);
    }).catch(() => {
      // If web-vitals is not available, return empty array
      resolve([]);
    });
  });
}

/**
 * Get unit for a metric name
 */
function getUnitForMetric(name: string): string {
  const units: Record<string, string> = {
    CLS: 'score',
    FID: 'ms',
    FCP: 'ms',
    LCP: 'ms',
    TTFB: 'ms',
    loadTime: 'ms',
    renderTime: 'ms',
    interactionLatency: 'ms',
    memoryUsage: 'MB',
  };
  return units[name] || 'ms';
}

/**
 * Evaluate metric against thresholds
 */
export function evaluateMetric(
  name: string,
  value: number
): { status: 'pass' | 'fail' | 'warning'; threshold: number } {
  const thresholds = PERFORMANCE_THRESHOLDS[name as keyof typeof PERFORMANCE_THRESHOLDS];

  if (!thresholds) {
    return { status: 'pass', threshold: 0 };
  }

  if (value <= thresholds.excellent) {
    return { status: 'pass', threshold: thresholds.excellent };
  } else if (value <= thresholds.good) {
    return { status: 'pass', threshold: thresholds.good };
  } else if (value <= thresholds.acceptable) {
    return { status: 'warning', threshold: thresholds.acceptable };
  } else {
    return { status: 'fail', threshold: thresholds.acceptable };
  }
}

/**
 * Compare metrics against baseline
 */
export function compareWithBaseline(
  current: PerformanceMetric[],
  baseline: PerformanceMetric[]
): PerformanceResult['comparison'] {
  const improved: string[] = [];
  const degraded: string[] = [];
  const unchanged: string[] = [];

  const baselineMap = new Map(baseline.map((m) => [m.name, m]));

  for (const metric of current) {
    const baselineMetric = baselineMap.get(metric.name);

    if (!baselineMetric) {
      unchanged.push(metric.name);
      continue;
    }

    const diff = metric.value - baselineMetric.value;
    const percentChange = (diff / baselineMetric.value) * 100;

    // For most metrics, lower is better (except CLS where higher is worse)
    if (metric.name === 'CLS') {
      if (percentChange < -10) {
        improved.push(metric.name);
      } else if (percentChange > 10) {
        degraded.push(metric.name);
      } else {
        unchanged.push(metric.name);
      }
    } else {
      if (percentChange < -10) {
        improved.push(metric.name);
      } else if (percentChange > 10) {
        degraded.push(metric.name);
      } else {
        unchanged.push(metric.name);
      }
    }
  }

  return { improved, degraded, unchanged };
}

/**
 * Generate performance report in JSON format
 */
export function generateJsonReport(results: PerformanceResult[]): string {
  return JSON.stringify(
    {
      timestamp: new Date().toISOString(),
      summary: {
        total: results.length,
        passed: results.filter((r) => r.status === 'pass').length,
        failed: results.filter((r) => r.status === 'fail').length,
        warnings: results.filter((r) => r.status === 'warning').length,
      },
      results,
    },
    null,
    2
  );
}

/**
 * Generate performance report in HTML format
 */
export function generateHtmlReport(results: PerformanceResult[]): string {
  const total = results.length;
  const passed = results.filter((r) => r.status === 'pass').length;
  const failed = results.filter((r) => r.status === 'fail').length;
  const warnings = results.filter((r) => r.status === 'warning').length;

  const resultsHtml = results
    .map(
      (result) => `
    <div class="result-card status-${result.status}">
      <h3>${result.viewName}</h3>
      <div class="status-badge ${result.status}">${result.status.toUpperCase()}</div>
      <table class="metrics-table">
        <thead>
          <tr>
            <th>Metric</th>
            <th>Value</th>
            <th>Unit</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          ${result.metrics
            .map(
              (metric) => {
                const evaluation = evaluateMetric(metric.name, metric.value);
                return `
            <tr>
              <td>${metric.name}</td>
              <td>${metric.value.toFixed(2)}</td>
              <td>${metric.unit}</td>
              <td class="status-${evaluation.status}">${evaluation.status}</td>
            </tr>
          `;
              }
            )
            .join('')}
        </tbody>
      </table>
      ${result.comparison ? `
      <div class="comparison">
        <h4>Comparison with Baseline</h4>
        <div class="comparison-stats">
          <span class="improved">Improved: ${result.comparison.improved.join(', ') || 'None'}</span>
          <span class="degraded">Degraded: ${result.comparison.degraded.join(', ') || 'None'}</span>
          <span class="unchanged">Unchanged: ${result.comparison.unchanged.join(', ') || 'None'}</span>
        </div>
      </div>
      ` : ''}
    </div>
  `
    )
    .join('');

  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Test Report</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }
        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
        }
        .header .timestamp {
            opacity: 0.9;
            font-size: 0.9em;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
        }
        .summary-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }
        .summary-card h3 {
            font-size: 2em;
            margin-bottom: 5px;
        }
        .summary-card p {
            color: #666;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .summary-card.total h3 { color: #667eea; }
        .summary-card.passed h3 { color: #10b981; }
        .summary-card.failed h3 { color: #ef4444; }
        .summary-card.warnings h3 { color: #f59e0b; }
        .results {
            padding: 30px;
        }
        .result-card {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
            border-left: 4px solid #667eea;
        }
        .result-card.status-pass { border-left-color: #10b981; }
        .result-card.status-fail { border-left-color: #ef4444; }
        .result-card.status-warning { border-left-color: #f59e0b; }
        .result-card h3 {
            display: inline-block;
            margin-right: 15px;
            font-size: 1.3em;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: bold;
            text-transform: uppercase;
        }
        .status-badge.pass { background: #d1fae5; color: #065f46; }
        .status-badge.fail { background: #fee2e2; color: #991b1b; }
        .status-badge.warning { background: #fef3c7; color: #92400e; }
        .metrics-table {
            width: 100%;
            margin-top: 15px;
            border-collapse: collapse;
        }
        .metrics-table th,
        .metrics-table td {
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #e5e7eb;
        }
        .metrics-table th {
            background: #f3f4f6;
            font-weight: 600;
            color: #374151;
        }
        .status-pass { color: #10b981; font-weight: bold; }
        .status-fail { color: #ef4444; font-weight: bold; }
        .status-warning { color: #f59e0b; font-weight: bold; }
        .comparison {
            margin-top: 15px;
            padding-top: 15px;
            border-top: 1px solid #e5e7eb;
        }
        .comparison h4 {
            margin-bottom: 10px;
            color: #374151;
        }
        .comparison-stats {
            display: flex;
            flex-wrap: wrap;
            gap: 15px;
        }
        .comparison-stats span {
            padding: 5px 10px;
            border-radius: 4px;
            font-size: 0.9em;
        }
        .comparison-stats .improved { background: #d1fae5; color: #065f46; }
        .comparison-stats .degraded { background: #fee2e2; color: #991b1b; }
        .comparison-stats .unchanged { background: #f3f4f6; color: #4b5563; }
        .footer {
            text-align: center;
            padding: 20px;
            color: #666;
            font-size: 0.9em;
            background: #f8f9fa;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Performance Test Report</h1>
            <div class="timestamp">Generated: ${new Date().toISOString()}</div>
        </div>

        <div class="summary">
            <div class="summary-card total">
                <h3>${total}</h3>
                <p>Total Tests</p>
            </div>
            <div class="summary-card passed">
                <h3>${passed}</h3>
                <p>Passed</p>
            </div>
            <div class="summary-card failed">
                <h3>${failed}</h3>
                <p>Failed</p>
            </div>
            <div class="summary-card warnings">
                <h3>${warnings}</h3>
                <p>Warnings</p>
            </div>
        </div>

        <div class="results">
            ${resultsHtml}
        </div>

        <div class="footer">
            INT3RCEPTOR UI Performance Testing Suite
        </div>
    </div>
</body>
</html>`;
}

/**
 * Save report to file
 */
export async function saveReport(
  content: string,
  filename: string,
  directory: string = './tests/performance/reports'
): Promise<void> {
  try {
    // Create directory if it doesn't exist
    await import('fs/promises').then((fs) => {
      return fs.mkdir(directory, { recursive: true });
    });

    const filepath = `${directory}/${filename}`;
    await import('fs/promises').then((fs) => {
      return fs.writeFile(filepath, content, 'utf-8');
    });
  } catch (error) {
    console.error(`Failed to save report: ${error}`);
    throw error;
  }
}

/**
 * Load baseline data from file
 */
export async function loadBaseline(
  filename: string = './tests/performance/baseline.json'
): Promise<PerformanceMetric[]> {
  try {
    const fs = await import('fs/promises');
    const content = await fs.readFile(filename, 'utf-8');
    const data = JSON.parse(content);
    return data.metrics || [];
  } catch (error) {
    console.warn(`Could not load baseline: ${error}`);
    return [];
  }
}

/**
 * Save baseline data to file
 */
export async function saveBaseline(
  metrics: PerformanceMetric[],
  filename: string = './tests/performance/baseline.json'
): Promise<void> {
  try {
    const fs = await import('fs/promises');
    const content = JSON.stringify(
      {
        timestamp: new Date().toISOString(),
        metrics,
      },
      null,
      2
    );
    await fs.writeFile(filename, content, 'utf-8');
  } catch (error) {
    console.error(`Failed to save baseline: ${error}`);
    throw error;
  }
}

/**
 * Calculate overall status from metrics
 */
export function calculateOverallStatus(metrics: PerformanceMetric[]): 'pass' | 'fail' | 'warning' {
  if (metrics.length === 0) {
    return 'pass';
  }

  const statuses = metrics.map((m) => evaluateMetric(m.name, m.value).status);

  if (statuses.some((s) => s === 'fail')) {
    return 'fail';
  } else if (statuses.some((s) => s === 'warning')) {
    return 'warning';
  }
  return 'pass';
}

/**
 * Create a performance result object
 */
export function createPerformanceResult(
  viewName: string,
  metrics: PerformanceMetric[],
  baseline?: PerformanceMetric[]
): PerformanceResult {
  const comparison = baseline ? compareWithBaseline(metrics, baseline) : undefined;
  const status = calculateOverallStatus(metrics);

  return {
    viewName,
    metrics,
    baseline,
    comparison,
    status,
    timestamp: new Date().toISOString(),
  };
}

/**
 * Measure page load time using Playwright
 */
export async function measurePageLoadTime(
  page: any,
  url: string
): Promise<PerformanceMetric> {
  const startTime = Date.now();

  // Navigate to the page and wait for load
  await page.goto(url, { waitUntil: 'networkidle' });

  const loadTime = Date.now() - startTime;

  return {
    name: 'loadTime',
    value: loadTime,
    unit: 'ms',
    timestamp: Date.now(),
  };
}

/**
 * Measure rendering performance metrics (FCP, LCP)
 */
export async function measureRenderingPerformance(
  page: any
): Promise<PerformanceMetric[]> {
  const metrics: PerformanceMetric[] = [];

  // Get performance metrics from the page
  const perfMetrics = await page.evaluate(() => {
    const perfEntries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[];
    const paintEntries = performance.getEntriesByType('paint') as PerformancePaintTiming[];

    const navigation = perfEntries[0];

    return {
      domContentLoaded: navigation?.domContentLoadedEventEnd - navigation?.domContentLoadedEventStart,
      loadComplete: navigation?.loadEventEnd - navigation?.loadEventStart,
      firstPaint: paintEntries.find(e => e.name === 'first-paint')?.startTime,
      firstContentfulPaint: paintEntries.find(e => e.name === 'first-contentful-paint')?.startTime,
    };
  });

  if (perfMetrics.firstContentfulPaint) {
    metrics.push({
      name: 'FCP',
      value: perfMetrics.firstContentfulPaint,
      unit: 'ms',
      timestamp: Date.now(),
    });
  }

  // Measure LCP using PerformanceObserver
  const lcpValue = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        resolve(lastEntry.startTime);
      });
      observer.observe({ entryTypes: ['largest-contentful-paint'] });

      // Fallback timeout
      setTimeout(() => resolve(0), 5000);
    });
  });

  if (lcpValue > 0) {
    metrics.push({
      name: 'LCP',
      value: lcpValue,
      unit: 'ms',
      timestamp: Date.now(),
    });
  }

  return metrics;
}

/**
 * Measure resource usage (network requests, bundle sizes)
 */
export async function measureResourceUsage(
  page: any
): Promise<PerformanceMetric[]> {
  const metrics: PerformanceMetric[] = [];

  // Get resource timing data
  const resourceData = await page.evaluate(() => {
    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[];

    let totalSize = 0;
    let scriptSize = 0;
    let stylesheetSize = 0;
    let imageSize = 0;
    let requestCount = resources.length;

    resources.forEach(resource => {
      const size = (resource.transferSize || 0);
      totalSize += size;

      if (resource.initiatorType === 'script') {
        scriptSize += size;
      } else if (resource.initiatorType === 'link') {
        stylesheetSize += size;
      } else if (resource.initiatorType === 'img') {
        imageSize += size;
      }
    });

    return {
      totalSize,
      scriptSize,
      stylesheetSize,
      imageSize,
      requestCount,
    };
  });

  metrics.push({
    name: 'totalBundleSize',
    value: resourceData.totalSize / 1024, // Convert to KB
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'scriptSize',
    value: resourceData.scriptSize / 1024,
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'stylesheetSize',
    value: resourceData.stylesheetSize / 1024,
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'requestCount',
    value: resourceData.requestCount,
    unit: 'count',
    timestamp: Date.now(),
  });

  return metrics;
}

/**
 * Run Lighthouse audit on a page
 */
export async function runLighthouseAudit(
  url: string,
  options: any = {}
): Promise<{
  score: number;
  metrics: PerformanceMetric[];
  report: any;
}> {
  const lighthouse = await import('lighthouse');
  const chromeLauncher = await import('chrome-launcher');

  const chrome = await chromeLauncher.launch({
    chromeFlags: ['--headless', '--disable-gpu', '--no-sandbox'],
  });

  const lighthouseOptions = {
    logLevel: 'info',
    output: 'json',
    onlyCategories: ['performance'],
    port: chrome.port,
    ...options,
  };

  const runnerResult = await lighthouse.default(url, lighthouseOptions);
  await chrome.kill();

  const report = JSON.parse(runnerResult.report);
  const score = report.categories.performance.score * 100;

  const metrics: PerformanceMetric[] = [
    {
      name: 'lighthousePerformanceScore',
      value: score,
      unit: 'score',
      timestamp: Date.now(),
    },
    {
      name: 'FCP',
      value: report.audits['first-contentful-paint'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'LCP',
      value: report.audits['largest-contentful-paint'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'TBT',
      value: report.audits['total-blocking-time'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'CLS',
      value: report.audits['cumulative-layout-shift'].numericValue,
      unit: 'score',
      timestamp: Date.now(),
    },
    {
      name: 'SpeedIndex',
      value: report.audits['speed-index'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'TTI',
      value: report.audits['interactive'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
  ];

  return {
    score,
    metrics,
    report,
  };
}

/**
 * Measure Time to Interactive (TTI)
 */
export async function measureTimeToInteractive(
  page: any
): Promise<PerformanceMetric> {
  const tti = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        resolve(lastEntry.startTime);
      });
      observer.observe({ entryTypes: ['longtask'] });

      // Fallback timeout
      setTimeout(() => resolve(0), 5000);
    });
  });

  return {
    name: 'TTI',
    value: tti,
    unit: 'ms',
    timestamp: Date.now(),
  };
}

/**
 * Measure Cumulative Layout Shift (CLS)
 */
export async function measureCumulativeLayoutShift(
  page: any
): Promise<PerformanceMetric> {
  const cls = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      let clsValue = 0;
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (!(entry as any).hadRecentInput) {
            clsValue += (entry as any).value;
          }
        }
      });
      observer.observe({ entryTypes: ['layout-shift'] });

      // Wait a bit for layout shifts to settle
      setTimeout(() => resolve(clsValue), 3000);
    });
  });

  return {
    name: 'CLS',
    value: cls,
    unit: 'score',
    timestamp: Date.now(),
  };
}

/**
 * Run performance test for a view
 */
export async function runPerformanceTest(
  page: any,
  viewName: string,
  url: string,
  options: {
    runLighthouse?: boolean;
    useColdCache?: boolean;
  } = {}
): Promise<PerformanceResult> {
  const metrics: PerformanceMetric[] = [];

  // Clear cache for cold start
  if (options.useColdCache !== false) {
    await page.context().clearCookies();
    await page.context().clearPermissions();
  }

  // Measure page load time
  const loadTimeMetric = await measurePageLoadTime(page, url);
  metrics.push(loadTimeMetric);

  // Wait for page to be stable
  await page.waitForTimeout(1000);

  // Measure rendering performance
  const renderMetrics = await measureRenderingPerformance(page);
  metrics.push(...renderMetrics);

  // Measure resource usage
  const resourceMetrics = await measureResourceUsage(page);
  metrics.push(...resourceMetrics);

  // Measure TTI
  const ttiMetric = await measureTimeToInteractive(page);
  if (ttiMetric.value > 0) {
    metrics.push(ttiMetric);
  }

  // Measure CLS
  const clsMetric = await measureCumulativeLayoutShift(page);
  metrics.push(clsMetric);

  // Run Lighthouse audit if requested
  if (options.runLighthouse) {
    try {
      const lighthouseResult = await runLighthouseAudit(url);
      metrics.push(...lighthouseResult.metrics);
    } catch (error) {
      console.warn('Lighthouse audit failed:', error);
    }
  }

  return createPerformanceResult(viewName, metrics);
}

/**
 * Generate performance comparison report
 */
export function generateComparisonReport(
  currentResults: PerformanceResult[],
  baselineResults: PerformanceResult[]
): string {
  const comparisonHtml = currentResults
    .map((current) => {
      const baseline = baselineResults.find((b) => b.viewName === current.viewName);

      if (!baseline) {
        return `
          <div class="comparison-card no-baseline">
            <h3>${current.viewName}</h3>
            <p class="no-baseline-message">No baseline data available</p>
          </div>
        `;
      }

      const comparison = compareWithBaseline(current.metrics, baseline.metrics);

      return `
        <div class="comparison-card">
          <h3>${current.viewName}</h3>
          <div class="comparison-summary">
            <div class="stat improved">
              <span class="label">Improved:</span>
              <span class="value">${comparison.improved.length}</span>
            </div>
            <div class="stat degraded">
              <span class="label">Degraded:</span>
              <span class="value">${comparison.degraded.length}</span>
            </div>
            <div class="stat unchanged">
              <span class="label">Unchanged:</span>
              <span class="value">${comparison.unchanged.length}</span>
            </div>
          </div>
          ${comparison.degraded.length > 0 ? `
            <div class="degraded-metrics">
              <h4>Degraded Metrics:</h4>
              <ul>
                ${comparison.degraded.map(metric => `<li>${metric}</li>`).join('')}
              </ul>
            </div>
          ` : ''}
        </div>
      `;
    })
    .join('');

  return comparisonHtml;
}

/**
 * Load performance configuration
 */
export async function loadPerformanceConfig(
  configPath: string = './tests/performance/performance.config.json'
): Promise<any> {
  try {
    const fs = await import('fs/promises');
    const content = await fs.readFile(configPath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    console.warn(`Could not load performance config: ${error}`);
    return {};
  }
}
/**
 * Measure page load time using Playwright
 */
export async function measurePageLoadTime(
  page: any,
  url: string
): Promise<PerformanceMetric> {
  const startTime = Date.now();

  // Navigate to the page and wait for load
  await page.goto(url, { waitUntil: 'networkidle' });

  const loadTime = Date.now() - startTime;

  return {
    name: 'loadTime',
    value: loadTime,
    unit: 'ms',
    timestamp: Date.now(),
  };
}

/**
 * Measure rendering performance metrics (FCP, LCP)
 */
export async function measureRenderingPerformance(
  page: any
): Promise<PerformanceMetric[]> {
  const metrics: PerformanceMetric[] = [];

  // Get performance metrics from the page
  const perfMetrics = await page.evaluate(() => {
    const perfEntries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[];
    const paintEntries = performance.getEntriesByType('paint') as PerformancePaintTiming[];

    const navigation = perfEntries[0];

    return {
      domContentLoaded: navigation?.domContentLoadedEventEnd - navigation?.domContentLoadedEventStart,
      loadComplete: navigation?.loadEventEnd - navigation?.loadEventStart,
      firstPaint: paintEntries.find(e => e.name === 'first-paint')?.startTime,
      firstContentfulPaint: paintEntries.find(e => e.name === 'first-contentful-paint')?.startTime,
    };
  });

  if (perfMetrics.firstContentfulPaint) {
    metrics.push({
      name: 'FCP',
      value: perfMetrics.firstContentfulPaint,
      unit: 'ms',
      timestamp: Date.now(),
    });
  }

  // Measure LCP using PerformanceObserver
  const lcpValue = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        resolve(lastEntry.startTime);
      });
      observer.observe({ entryTypes: ['largest-contentful-paint'] });

      // Fallback timeout
      setTimeout(() => resolve(0), 5000);
    });
  });

  if (lcpValue > 0) {
    metrics.push({
      name: 'LCP',
      value: lcpValue,
      unit: 'ms',
      timestamp: Date.now(),
    });
  }

  return metrics;
}

/**
 * Measure resource usage (network requests, bundle sizes)
 */
export async function measureResourceUsage(
  page: any
): Promise<PerformanceMetric[]> {
  const metrics: PerformanceMetric[] = [];

  // Get resource timing data
  const resourceData = await page.evaluate(() => {
    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[];

    let totalSize = 0;
    let scriptSize = 0;
    let stylesheetSize = 0;
    let imageSize = 0;
    let requestCount = resources.length;

    resources.forEach(resource => {
      const size = (resource.transferSize || 0);
      totalSize += size;

      if (resource.initiatorType === 'script') {
        scriptSize += size;
      } else if (resource.initiatorType === 'link') {
        stylesheetSize += size;
      } else if (resource.initiatorType === 'img') {
        imageSize += size;
      }
    });

    return {
      totalSize,
      scriptSize,
      stylesheetSize,
      imageSize,
      requestCount,
    };
  });

  metrics.push({
    name: 'totalBundleSize',
    value: resourceData.totalSize / 1024, // Convert to KB
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'scriptSize',
    value: resourceData.scriptSize / 1024,
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'stylesheetSize',
    value: resourceData.stylesheetSize / 1024,
    unit: 'KB',
    timestamp: Date.now(),
  });

  metrics.push({
    name: 'requestCount',
    value: resourceData.requestCount,
    unit: 'count',
    timestamp: Date.now(),
  });

  return metrics;
}

/**
 * Run Lighthouse audit on a page
 */
export async function runLighthouseAudit(
  url: string,
  options: any = {}
): Promise<{
  score: number;
  metrics: PerformanceMetric[];
  report: any;
}> {
  const lighthouse = await import('lighthouse');
  const chromeLauncher = await import('chrome-launcher');

  const chrome = await chromeLauncher.launch({
    chromeFlags: ['--headless', '--disable-gpu', '--no-sandbox'],
  });

  const lighthouseOptions = {
    logLevel: 'info',
    output: 'json',
    onlyCategories: ['performance'],
    port: chrome.port,
    ...options,
  };

  const runnerResult = await lighthouse.default(url, lighthouseOptions);
  await chrome.kill();

  const report = JSON.parse(runnerResult.report);
  const score = report.categories.performance.score * 100;

  const metrics: PerformanceMetric[] = [
    {
      name: 'lighthousePerformanceScore',
      value: score,
      unit: 'score',
      timestamp: Date.now(),
    },
    {
      name: 'FCP',
      value: report.audits['first-contentful-paint'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'LCP',
      value: report.audits['largest-contentful-paint'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'TBT',
      value: report.audits['total-blocking-time'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'CLS',
      value: report.audits['cumulative-layout-shift'].numericValue,
      unit: 'score',
      timestamp: Date.now(),
    },
    {
      name: 'SpeedIndex',
      value: report.audits['speed-index'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
    {
      name: 'TTI',
      value: report.audits['interactive'].numericValue,
      unit: 'ms',
      timestamp: Date.now(),
    },
  ];

  return {
    score,
    metrics,
    report,
  };
}

/**
 * Measure Time to Interactive (TTI)
 */
export async function measureTimeToInteractive(
  page: any
): Promise<PerformanceMetric> {
  const tti = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        resolve(lastEntry.startTime);
      });
      observer.observe({ entryTypes: ['longtask'] });

      // Fallback timeout
      setTimeout(() => resolve(0), 5000);
    });
  });

  return {
    name: 'TTI',
    value: tti,
    unit: 'ms',
    timestamp: Date.now(),
  };
}

/**
 * Measure Cumulative Layout Shift (CLS)
 */
export async function measureCumulativeLayoutShift(
  page: any
): Promise<PerformanceMetric> {
  const cls = await page.evaluate(() => {
    return new Promise<number>((resolve) => {
      let clsValue = 0;
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (!(entry as any).hadRecentInput) {
            clsValue += (entry as any).value;
          }
        }
      });
      observer.observe({ entryTypes: ['layout-shift'] });

      // Wait a bit for layout shifts to settle
      setTimeout(() => resolve(clsValue), 3000);
    });
  });

  return {
    name: 'CLS',
    value: cls,
    unit: 'score',
    timestamp: Date.now(),
  };
}

/**
 * Run performance test for a view
 */
export async function runPerformanceTest(
  page: any,
  viewName: string,
  url: string,
  options: {
    runLighthouse?: boolean;
    useColdCache?: boolean;
  } = {}
): Promise<PerformanceResult> {
  const metrics: PerformanceMetric[] = [];

  // Clear cache for cold start
  if (options.useColdCache !== false) {
    await page.context().clearCookies();
    await page.context().clearPermissions();
  }

  // Measure page load time
  const loadTimeMetric = await measurePageLoadTime(page, url);
  metrics.push(loadTimeMetric);

  // Wait for page to be stable
  await page.waitForTimeout(1000);

  // Measure rendering performance
  const renderMetrics = await measureRenderingPerformance(page);
  metrics.push(...renderMetrics);

  // Measure resource usage
  const resourceMetrics = await measureResourceUsage(page);
  metrics.push(...resourceMetrics);

  // Measure TTI
  const ttiMetric = await measureTimeToInteractive(page);
  if (ttiMetric.value > 0) {
    metrics.push(ttiMetric);
  }

  // Measure CLS
  const clsMetric = await measureCumulativeLayoutShift(page);
  metrics.push(clsMetric);

  // Run Lighthouse audit if requested
  if (options.runLighthouse) {
    try {
      const lighthouseResult = await runLighthouseAudit(url);
      metrics.push(...lighthouseResult.metrics);
    } catch (error) {
      console.warn('Lighthouse audit failed:', error);
    }
  }

  return createPerformanceResult(viewName, metrics);
}

/**
 * Generate performance comparison report
 */
export function generateComparisonReport(
  currentResults: PerformanceResult[],
  baselineResults: PerformanceResult[]
): string {
  const comparisonHtml = currentResults
    .map((current) => {
      const baseline = baselineResults.find((b) => b.viewName === current.viewName);

      if (!baseline) {
        return `
          <div class="comparison-card no-baseline">
            <h3>${current.viewName}</h3>
            <p class="no-baseline-message">No baseline data available</p>
          </div>
        `;
      }

      const comparison = compareWithBaseline(current.metrics, baseline.metrics);

      return `
        <div class="comparison-card">
          <h3>${current.viewName}</h3>
          <div class="comparison-summary">
            <div class="stat improved">
              <span class="label">Improved:</span>
              <span class="value">${comparison.improved.length}</span>
            </div>
            <div class="stat degraded">
              <span class="label">Degraded:</span>
              <span class="value">${comparison.degraded.length}</span>
            </div>
            <div class="stat unchanged">
              <span class="label">Unchanged:</span>
              <span class="value">${comparison.unchanged.length}</span>
            </div>
          </div>
          ${comparison.degraded.length > 0 ? `
            <div class="degraded-metrics">
              <h4>Degraded Metrics:</h4>
              <ul>
                ${comparison.degraded.map(metric => `<li>${metric}</li>`).join('')}
              </ul>
            </div>
          ` : ''}
        </div>
      `;
    })
    .join('');

  return comparisonHtml;
}

/**
 * Load performance configuration
 */
export async function loadPerformanceConfig(
  configPath: string = './tests/performance/performance.config.json'
): Promise<any> {
  try {
    const fs = await import('fs/promises');
    const content = await fs.readFile(configPath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    console.warn(`Could not load performance config: ${error}`);
    return {};
  }
}

