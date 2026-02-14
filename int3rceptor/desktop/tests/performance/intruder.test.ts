import { test, expect } from '@playwright/test';
import {
  runPerformanceTest,
  loadBaseline,
  PERFORMANCE_THRESHOLDS
} from './utils';

test.describe('Intruder Performance Tests', () => {
  const viewName = 'Intruder';
  const url = 'http://localhost:5173/#/intruder';
  
  test.beforeEach(async ({ page }) => {
    await page.context().clearCookies();
    await page.context().clearPermissions();
  });

  test('should load within performance budget (cold start)', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
      useColdCache: true,
    });
    
    const loadTimeMetric = result.metrics.find(m => m.name === 'loadTime');
    expect(loadTimeMetric).toBeDefined();
    expect(loadTimeMetric!.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.loadTime.acceptable);
  });

  test('should load within performance budget (warm cache)', async ({ page }) => {
    await page.goto(url);
    await page.waitForTimeout(1000);
    
    const result = await runPerformanceTest(page, viewName, url, {
      runLighthouse: false,
      useColdCache: false,
    });
    
    const loadTimeMetric = result.metrics.find(m => m.name === 'loadTime');
    expect(loadTimeMetric).toBeDefined();
    expect(loadTimeMetric!.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.loadTime.good);
  });

  test('should have acceptable FCP', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url);
    
    const fcpMetric = result.metrics.find(m => m.name === 'FCP');
    if (fcpMetric) {
      expect(fcpMetric.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.fcp.acceptable);
    }
  });

  test('should have acceptable LCP', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url);
    
    const lcpMetric = result.metrics.find(m => m.name === 'LCP');
    if (lcpMetric) {
      expect(lcpMetric.value).toBeLessThanOrEqual(PERFORMANCE_THRESHOLDS.lcp.acceptable);
    }
  });

  test('should have low CLS', async ({ page }) => {
    const result = await runPerformanceTest(page, viewName, url);
    
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
    expect(scoreMetric!.value).toBeGreaterThanOrEqual(90);
  });

  test('should compare against baseline', async ({ page }) => {
    const baseline = await loadBaseline('./tests/performance/baselines/intruder.json');
    const result = await runPerformanceTest(page, viewName, url);
    
    if (baseline.length > 0 && result.comparison) {
      expect(result.comparison.degraded.length).toBe(0);
    }
  });
});
