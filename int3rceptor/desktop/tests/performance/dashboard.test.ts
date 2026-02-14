import { test, expect } from '@playwright/test';
import {
  runPerformanceTest,
  saveReport,
  generateHtmlReport,
  loadBaseline,
  saveBaseline,
  loadPerformanceConfig,
  PERFORMANCE_THRESHOLDS
} from './utils';

test.describe('Dashboard Performance Tests', () => {
  const viewName = 'Dashboard';
  const url = 'http://localhost:5173/#/dashboard';
  
  test.beforeEach(async ({ page }) => {
    // Ensure we start from a clean state
    await page.context().clearCookies();
    await page.context().clearPermissions();
  });

  test('should load within performance budget (cold start)', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
      useColdCache: true,
    });
    
    // Check load time against threshold
    const loadTimeMetric = result.metrics.find(m => m.name === 'loadTime');
    expect(loadTimeMetric).toBeDefined();
    expect(loadTimeMetric!.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.loadTime.acceptable);
    
    // Check FCP against threshold
    const fcpMetric = result.metrics.find(m => m.name === 'FCP');
    if (fcpMetric) {
      expect(fcpMetric.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.fcp.acceptable);
    }
  });

  test('should load within performance budget (warm cache)', async ({ page }) => {
    // First visit to populate cache
    await page.goto(url);
    await page.waitForTimeout(1000);
    
    // Second visit with warm cache
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
      useColdCache: false,
    });
    
    // Warm cache should be faster
    const loadTimeMetric = result.metrics.find(m => m.name === 'loadTime');
    expect(loadTimeMetric).toBeDefined();
    expect(loadTimeMetric!.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.loadTime.good);
  });

  test('should have acceptable bundle sizes', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
    });
    
    const bundleSizeMetric = result.metrics.find(m => m.name === 'totalBundleSize');
    expect(bundleSizeMetric).toBeDefined();
    expect(bundleSizeMetric!.value).toBeLessThan(500); // Less than 500KB
  });

  test('should have low Cumulative Layout Shift', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
    });
    
    const clsMetric = result.metrics.find(m => m.name === 'CLS');
    expect(clsMetric).toBeDefined();
    expect(clsMetric!.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.cls.acceptable);
  });

  test('should pass Lighthouse performance audit', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: true,
    });
    
    const scoreMetric = result.metrics.find(m => m.name === 'lighthousePerformanceScore');
    expect(scoreMetric).toBeDefined();
    expect(scoreMetric!.value).toBeGreaterThanOrEqual(90); // Score > 90
  });

  test('should compare against baseline', async ({ page }) => {
    const baseline = await loadBaseline('./tests/performance/baselines/dashboard.json');
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
    });
    
    if (baseline.length > 0) {
      // Compare with baseline
      const comparison = result.comparison;
      expect(comparison).toBeDefined();
      
      // Should not have degraded metrics
      if (comparison) {
        expect(comparison.degraded.length).toBe(0);
      }
    }
  });
});
