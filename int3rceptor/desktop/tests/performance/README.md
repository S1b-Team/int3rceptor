# Performance Testing for INT3RCEPTOR

This directory contains performance tests for the INT3RCEPTOR desktop application. The tests measure and validate load times, rendering performance, and resource usage across all views.

## Overview

The performance testing suite provides comprehensive metrics for:

- **Page Load Time**: Time from navigation start to page completion
- **First Contentful Paint (FCP)**: Time when first content is rendered
- **Largest Contentful Paint (LCP)**: Time when largest content element is rendered
- **Cumulative Layout Shift (CLS)**: Visual stability score
- **Time to Interactive (TTI)**: Time when page is fully interactive
- **Resource Usage**: Bundle sizes, request counts, and network metrics
- **Lighthouse Performance Score**: Overall performance rating (0-100)

## Performance Budgets

The following performance budgets are configured in [`performance.config.json`](./performance.config.json):

| Metric | Budget | Description |
|---------|----------|-------------|
| Max Page Load Time | 3000ms | Total page load time |
| First Contentful Paint | 1500ms | First meaningful paint |
| Largest Contentful Paint | 2500ms | Largest content element painted |
| Time to Interactive | 3500ms | Page becomes fully interactive |
| Performance Score | > 90 | Lighthouse performance score |
| Cumulative Layout Shift | < 0.1 | Visual stability |
| Total Blocking Time | 300ms | Main thread blocking time |
| Speed Index | 3400ms | Visual completeness speed |

## Test Scenarios

### Cold Start Tests
Tests the application performance with no cached resources, simulating a first-time visitor.

```bash
npm run test:perf:cold
```

### Warm Cache Tests
Tests the application performance with cached resources, simulating a returning visitor.

```bash
npm run test:perf:warm
```

## Usage

### Run All Performance Tests

Run all performance tests across all views:

```bash
npm run test:perf
```

This will execute tests for all 10 views:
- Dashboard
- Traffic
- WebSocket
- Repeater
- Intruder
- Scanner
- Decoder
- Comparer
- Plugins
- Settings

### Run with Lighthouse Audits

Include Lighthouse audits for comprehensive performance analysis:

```bash
npm run test:perf:lighthouse
```

### Generate Performance Report

Generate an HTML performance report with charts and visualizations:

```bash
npm run test:perf:report
```

The report will be saved to `tests/performance/reports/`.

### Save Baseline

Save current performance metrics as a baseline for regression detection:

```bash
npm run test:perf:baseline
```

Baselines are saved to `tests/performance/baselines/`.

## Test Files

Each view has its own test file:

- [`dashboard.test.ts`](./dashboard.test.ts) - Dashboard view performance
- [`traffic.test.ts`](./traffic.test.ts) - Traffic view performance
- [`websocket.test.ts`](./websocket.test.ts) - WebSocket view performance
- [`repeater.test.ts`](./repeater.test.ts) - Repeater view performance
- [`intruder.test.ts`](./intruder.test.ts) - Intruder view performance
- [`scanner.test.ts`](./scanner.test.ts) - Scanner view performance
- [`decoder.test.ts`](./decoder.test.ts) - Decoder view performance
- [`comparer.test.ts`](./comparer.test.ts) - Comparer view performance
- [`plugins.test.ts`](./plugins.test.ts) - Plugins view performance
- [`settings.test.ts`](./settings.test.ts) - Settings view performance

## Performance Utilities

The [`utils.ts`](./utils.ts) file provides helper functions for performance testing:

### Core Functions

- `measurePageLoadTime(page, url)` - Measures initial load performance
- `measureRenderingPerformance(page)` - Measures FCP and LCP
- `measureResourceUsage(page)` - Measures network requests and bundle sizes
- `runLighthouseAudit(url, options)` - Runs Lighthouse performance audit
- `runPerformanceTest(page, viewName, url, options)` - Complete performance test

### Helper Functions

- `evaluateMetric(name, value)` - Evaluates metric against thresholds
- `compareWithBaseline(current, baseline)` - Compares metrics with baseline
- `generateHtmlReport(results)` - Generates HTML performance report
- `generateJsonReport(results)` - Generates JSON performance report
- `saveBaseline(metrics, filename)` - Saves baseline data
- `loadBaseline(filename)` - Loads baseline data

## Performance Metrics

### Load Time Metrics

- **loadTime**: Total time from navigation start to page load completion
- **FCP** (First Contentful Paint): Time when first content is painted
- **LCP** (Largest Contentful Paint): Time when largest content is painted

### Rendering Metrics

- **CLS** (Cumulative Layout Shift): Score of visual stability (lower is better)
- **TBT** (Total Blocking Time): Time main thread is blocked
- **TTI** (Time to Interactive): Time when page is fully interactive

### Resource Metrics

- **totalBundleSize**: Total size of all resources (KB)
- **scriptSize**: Total size of JavaScript files (KB)
- **stylesheetSize**: Total size of CSS files (KB)
- **requestCount**: Total number of network requests

### Lighthouse Metrics

- **lighthousePerformanceScore**: Overall Lighthouse performance score (0-100)
- **SpeedIndex**: Visual completeness speed metric

## Performance Thresholds

Metrics are evaluated against three levels:

| Level | Description |
|--------|-------------|
| Excellent | Meets or exceeds best practices |
| Good | Acceptable performance |
| Acceptable | Minimum acceptable performance |
| Fail | Below acceptable threshold |

## Reports

### HTML Reports

HTML reports provide visual charts and detailed metrics:

- Summary cards showing pass/fail/warning counts
- Per-view metric tables with status indicators
- Baseline comparisons showing improvements/degradations
- Color-coded status indicators (green=pass, red=fail, yellow=warning)

Reports are saved to `tests/performance/reports/`.

### JSON Reports

JSON reports provide machine-readable metrics for CI/CD integration:

```json
{
  "timestamp": "2025-01-26T17:00:00.000Z",
  "summary": {
    "total": 10,
    "passed": 8,
    "failed": 1,
    "warnings": 1
  },
  "results": [...]
}
```

## Baseline Management

### Creating a Baseline

Run tests with the baseline flag to establish performance baselines:

```bash
npm run test:perf:baseline
```

This saves current metrics to `tests/performance/baselines/`.

### Comparing Against Baseline

Tests automatically compare current metrics against saved baselines and report:
- **Improved**: Metrics that have improved (>10% better)
- **Degraded**: Metrics that have degraded (>10% worse)
- **Unchanged**: Metrics within 10% of baseline

### Updating Baselines

To update baselines after performance improvements:

1. Run performance tests
2. Review the results
3. Run `npm run test:perf:baseline` to save new baselines

## Integration with Playwright

Performance tests integrate with the existing Playwright setup:

- Uses Playwright's [`playwright.config.ts`](../playwright.config.ts)
- Runs in headless Chrome by default
- Supports all Playwright configuration options
- Generates HTML reports via Playwright reporter

## CI/CD Integration

### GitHub Actions

Add performance tests to your CI pipeline:

```yaml
- name: Run Performance Tests
  run: npm run test:perf
  
- name: Upload Performance Report
  uses: actions/upload-artifact@v3
  with:
    name: performance-report
    path: tests/performance/reports/
```

### Performance Regression Detection

Configure CI to fail on performance regressions:

```yaml
- name: Check Performance Regression
  run: |
    if [ "$(npm run test:perf -- --reporter=json | jq '.summary.failed')" -gt 0 ]; then
      echo "Performance regression detected!"
      exit 1
    fi
```

## Troubleshooting

### Tests Failing on Load Time

1. Check if dev server is running: `npm run dev`
2. Verify network conditions are stable
3. Check for large bundle sizes in resource metrics
4. Review browser console for errors

### Lighthouse Audits Failing

1. Ensure Chrome is installed and accessible
2. Check for conflicting Chrome processes
3. Verify sufficient system resources
4. Review Lighthouse output for specific issues

### Baseline Comparisons Not Working

1. Verify baseline files exist in `tests/performance/baselines/`
2. Check baseline JSON format is correct
3. Ensure view names match between tests and baselines

## Best Practices

1. **Run tests regularly**: Monitor performance over time
2. **Update baselines**: After intentional performance changes
3. **Investigate regressions**: Address performance degradations promptly
4. **Monitor bundle sizes**: Keep resources optimized
5. **Test on different devices**: Consider mobile performance
6. **Use real data**: Test with realistic data volumes

## Additional Resources

- [Lighthouse Documentation](https://developers.google.com/web/tools/lighthouse)
- [Web Vitals](https://web.dev/vitals/)
- [Playwright Performance](https://playwright.dev/docs/performance)
- [Performance Budgets](https://web.dev/performance-budgets-101/)

## Contributing

When adding new views:

1. Create a new test file following existing patterns
2. Add the view URL to `performance.config.json`
3. Include tests for cold start, warm cache, and Lighthouse
4. Update this README with the new view

## License

Part of the INT3RCEPTOR project. See main project LICENSE file for details.
