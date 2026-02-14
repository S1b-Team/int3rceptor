# Master Test Workflow - INT3RCEPTOR UI

This document describes the complete test workflow for the INT3RCEPTOR desktop application, including cross-browser, accessibility, visual regression, and performance testing.

## Overview

The master test workflow provides a comprehensive testing infrastructure that runs all test suites sequentially, captures results, and generates consolidated reports with actionable findings.

## Test Suites

### 1. Cross-Browser Testing
Tests the application across multiple browsers to ensure compatibility:
- Chrome
- Firefox
- Edge
- Safari (WebKit)

**Location:** [`tests/cross-browser/`](./cross-browser/)

**Run:** `npm run test:cross-browser`

### 2. Accessibility Testing
Validates WCAG 2.1 AA compliance using axe-core:
- ARIA attributes
- Keyboard navigation
- Screen reader compatibility
- Color contrast
- Focus management

**Location:** [`tests/accessibility/`](./accessibility/)

**Run:** `npm run test:a11y`

### 3. Visual Regression Testing
Captures and compares screenshots to detect visual changes:
- Dashboard views
- Traffic monitoring
- Intruder testing
- Settings pages

**Location:** [`tests/visual/`](./visual/)

**Run:** `npm run test:visual`

### 4. Performance Testing
Measures application performance metrics:
- Render time
- Load time
- Interaction latency
- Memory usage
- Update latency

**Location:** [`tests/performance/`](./performance/)

**Run:** `npm run test:perf`

## Master Test Runner

The master test runner executes all test suites in optimal order (fastest first) and generates a consolidated report.

### Usage

```bash
# Run all test suites with report generation
npm run test:all

# Run all test suites without report
bash scripts/run-all-tests.sh --summary-only

# Skip specific test suites
bash scripts/run-all-tests.sh --skip-perf
bash scripts/run-all-tests.sh --skip-visual
bash scripts/run-all-tests.sh --skip-a11y
bash scripts/run-all-tests.sh --skip-cross-browser
```

### Available Options

| Option | Description |
|--------|-------------|
| `--skip-cross-browser` | Skip cross-browser tests |
| `--skip-a11y` | Skip accessibility tests |
| `--skip-visual` | Skip visual regression tests |
| `--skip-perf` | Skip performance tests |
| `--generate-report` | Generate consolidated report after tests (default) |
| `--summary-only` | Print quick summary to console only |
| `--help` | Show help message |

### Test Execution Order

Tests run in the following order (fastest first):

1. **Cross-Browser Tests** (~2-3 minutes)
2. **Accessibility Tests** (~3-5 minutes)
3. **Visual Regression Tests** (~5-8 minutes)
4. **Performance Tests** (~10-15 minutes)

## Report Generation

### Generate Consolidated Report

```bash
# Generate report from existing test results
npm run test:report

# Or using the script directly
node scripts/generate-report.js
```

### Report Outputs

The report generator creates two files:

1. **HTML Report:** `test-results/consolidated-report.html`
   - Summary dashboard with pass/fail counts
   - Cross-browser compatibility matrix
   - Accessibility violations grouped by severity
   - Performance metrics with baseline comparison
   - Visual regression diff gallery
   - Failed test details with screenshots

2. **Findings JSON:** `test-results/findings.json`
   - Structured data for all issues
   - Categorized by type and severity
   - Actionable recommendations
   - Estimated fix effort (low/medium/high)

### Report Sections

#### Summary Dashboard
- Total suites passed/failed
- Overall execution time
- Quick status indicators

#### Cross-Browser Compatibility Matrix
- Browser-specific test results
- Pass/fail status per browser
- Compatibility issues with details

#### Accessibility Violations
- Grouped by severity (critical, serious, moderate, minor)
- WCAG rule references
- Affected elements
- Fix recommendations

#### Performance Metrics
- Metric values vs. baselines
- Performance trends
- Optimization opportunities

#### Visual Regression Gallery
- Side-by-side comparisons
- Diff highlights
- Screenshot thumbnails

#### Recommendations
- Prioritized action items
- Estimated effort levels
- Implementation guidance

## Individual Test Suite Scripts

### Cross-Browser Tests

```bash
# Run all cross-browser tests
npm run test:cross-browser

# Run specific browser tests
npm run test:cross-browser:chrome
npm run test:cross-browser:firefox
npm run test:cross-browser:edge
npm run test:cross-browser:safari

# View cross-browser report
npm run test:cross-browser:report
```

### Accessibility Tests

```bash
# Run accessibility tests
npm run test:a11y

# View accessibility report
npm run test:a11y:report

# Run Pa11y CI
npm run test:a11y:pa11y
```

### Visual Regression Tests

```bash
# Run visual tests
npm run test:visual

# Update baseline screenshots
npm run test:visual:update

# Compare against mockups
npm run test:visual:mockups

# Generate visual report
npm run test:visual:report
```

### Performance Tests

```bash
# Run performance tests
npm run test:perf

# Generate performance report
npm run test:perf:report

# Run Lighthouse
npm run test:perf:lighthouse

# Save baseline
npm run test:perf:baseline

# Run cold start tests
npm run test:perf:cold

# Run warm start tests
npm run test:perf:warm
```

## Test Results Location

All test results are stored in the [`test-results/`](../test-results/) directory:

```
test-results/
├── consolidated-report.html    # Consolidated HTML report
├── findings.json               # Structured findings data
├── results.json                # Playwright results
├── cross-browser/              # Cross-browser test outputs
│   └── test-output.log
├── accessibility/              # Accessibility test outputs
│   └── test-output.log
├── visual/                     # Visual regression outputs
│   ├── test-output.log
│   └── screenshots/
└── performance/                # Performance test outputs
    └── test-output.log
```

## Configuration Files

- **Playwright Config:** [`playwright.config.ts`](../playwright.config.ts)
- **Visual Regression Config:** [`visual-regression.config.json`](../visual-regression.config.json)
- **Pa11y CI Config:** [`.pa11yci.json`](../.pa11yci.json)
- **Performance Config:** [`tests/performance/performance.config.json`](./performance/performance.config.json)

## Continuous Integration

The test workflow is designed for CI/CD integration:

```yaml
# Example GitHub Actions workflow
- name: Run all tests
  run: npm run test:all

- name: Upload test results
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: test-results/
```

## Troubleshooting

### Tests Fail to Start

1. Ensure dependencies are installed:
   ```bash
   npm install
   npx playwright install --with-deps
   ```

2. Check if dev server is running on port 5173
3. Verify browser installations

### Visual Tests Fail

1. Review diff images in `test-results/visual/`
2. Update baselines if changes are intentional:
   ```bash
   npm run test:visual:update
   ```

### Accessibility Tests Fail

1. Check violations in the HTML report
2. Review WCAG rule references
3. Fix issues and re-run tests

### Performance Tests Fail

1. Review metrics in the performance report
2. Compare against baselines
3. Optimize code and re-run tests

## Best Practices

1. **Run tests before committing:** Always run the full test suite before pushing changes
2. **Review reports:** Check the consolidated report for any regressions
3. **Update baselines:** When making intentional visual changes, update baselines
4. **Fix accessibility issues:** Address accessibility violations promptly
5. **Monitor performance:** Keep an eye on performance trends over time

## Contributing

When adding new features:

1. Write tests for the new functionality
2. Ensure all existing tests pass
3. Update visual baselines if UI changes
4. Run the full test suite before submitting PRs
5. Include test results in PR description

## Support

For issues or questions about the test workflow:

1. Check this README for common solutions
2. Review individual test suite documentation
3. Check the consolidated report for detailed error information
4. Open an issue with test results attached

## License

See [`../../LICENSE`](../../LICENSE) for license information.
