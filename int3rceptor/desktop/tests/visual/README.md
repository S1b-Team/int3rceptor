# Visual Regression Testing Suite

Comprehensive visual regression testing automation for INT3RCEPTOR UI, designed to ensure UI consistency during the dashboard realignment testing phase.

## Overview

This visual regression testing suite uses Playwright to capture screenshots of UI views and compare them against approved mockups and baseline screenshots. It helps detect unintended visual changes during development and ensures the UI matches approved designs.

## Features

- **Automated Screenshot Capture**: Captures full-page screenshots of all UI views
- **Mockup Comparison**: Compares current UI against approved design mockups
- **Baseline Tracking**: Maintains baseline screenshots for regression detection
- **Multi-Browser Support**: Tests across Chrome, Firefox, and Safari (WebKit)
- **Responsive Testing**: Validates UI at multiple viewport sizes
- **Diff Generation**: Creates visual diff images highlighting changes
- **HTML Reports**: Generates comprehensive HTML reports with side-by-side comparisons
- **Pixel-Perfect Thresholds**: Configurable tolerance levels (default: 0.1%)

## Installation

### Prerequisites

- Node.js (v18 or higher)
- npm or yarn
- Linux, macOS, or Windows with WSL2

### Setup Steps

1. **Navigate to the desktop directory:**
   ```bash
   cd int3rceptor/desktop
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Install Playwright browsers:**
   ```bash
   npx playwright install --with-deps
   ```

4. **Verify installation:**
   ```bash
   npx playwright --version
   ```

## Directory Structure

```
tests/visual/
├── README.md                    # This documentation
├── utils.ts                     # Visual regression utilities
├── mockups.json                 # Mockup reference mapping
├── dashboard.test.ts            # Dashboard view tests
├── traffic.test.ts              # Traffic tab tests
├── intruder.test.ts            # Intruder tab tests
├── repeater.test.ts            # Repeater/REST tab tests
├── websocket.test.ts           # WebSocket tab tests
├── loading.test.ts             # Loading state tests
├── scanner.test.ts             # Scanner tab tests (baseline only)
├── settings.test.ts            # Settings tab tests (baseline only)
├── plugins.test.ts             # Plugins tab tests (baseline only)
├── decoder.test.ts             # Decoder tab tests (baseline only)
├── comparer.test.ts            # Comparer tab tests (baseline only)
└── reports/
    ├── sample-diff-report.html  # Sample HTML diff report
    ├── html/                    # Playwright HTML reports
    └── results.json            # JSON test results
```

## Running Tests

### Quick Start

Run all visual regression tests against baselines:

```bash
npm run test:visual
```

Or using the script directly:

```bash
./scripts/run-visual-tests.sh
```

### Test Modes

#### 1. Baseline Comparison (Default)

Compares current screenshots against saved baselines:

```bash
npm run test:visual
```

#### 2. Update Baselines

Updates baseline screenshots with current UI state:

```bash
npm run test:visual:update
```

Or:

```bash
./scripts/run-visual-tests.sh --update-baselines
```

**When to use:** After making intentional UI changes or when first setting up tests.

#### 3. Mockup Comparison

Compares current UI against approved design mockups:

```bash
npm run test:visual:mockups
```

Or:

```bash
./scripts/run-visual-tests.sh --compare-mockups
```

**When to use:** During initial development to verify alignment with approved designs.

#### 4. Generate Report

Generates HTML diff report after tests:

```bash
npm run test:visual:report
```

Or:

```bash
./scripts/run-visual-tests.sh --generate-report
```

### Advanced Options

#### Run Tests in Headed Mode

Shows the browser during test execution:

```bash
./scripts/run-visual-tests.sh --headed
```

#### Test Specific Browser

```bash
./scripts/run-visual-tests.sh --browser firefox
```

Available browsers: `chromium`, `firefox`, `webkit`

#### Verbose Output

```bash
./scripts/run-visual-tests.sh --verbose
```

#### View Help

```bash
./scripts/run-visual-tests.sh --help
```

## Understanding Test Results

### Test Status

- **PASSED**: Screenshot matches baseline/mockup within threshold
- **FAILED**: Screenshot differs beyond threshold

### Metrics

Each test provides the following metrics:

- **Diff Pixels**: Number of pixels that differ
- **Diff Percentage**: Percentage of total pixels that differ
- **Threshold**: Maximum allowed difference (default: 0.1%)
- **Status**: Whether the test passed or failed

### Interpreting Diff Reports

When a test fails, three images are generated:

1. **Baseline/Mockup**: Expected appearance
2. **Actual**: Current appearance
3. **Diff**: Visual difference overlay (red pixels show changes)

The HTML report displays these images side-by-side for easy comparison.

## Configuration

### Threshold Settings

Edit the default threshold in [`utils.ts`](utils.ts):

```typescript
const DEFAULT_CONFIG: VisualTestOptions = {
  threshold: 0.1,        // 0.1% pixel difference tolerance
  maxDiffPixels: 100,    // Maximum allowed different pixels
  fullPage: true,        // Capture full page
  animations: 'disabled' // Disable animations for consistency
};
```

### Browser Configuration

Edit [`playwright.config.ts`](../../playwright.config.ts):

```typescript
projects: [
  {
    name: 'chromium',
    use: { ...devices['Desktop Chrome'] },
  },
  {
    name: 'firefox',
    use: { ...devices['Desktop Firefox'] },
  },
  {
    name: 'webkit',
    use: { ...devices['Desktop Safari'] },
  },
],
```

### Mockup Mappings

Edit [`mockups.json`](mockups.json) to add or update mockup references:

```json
{
  "mappings": [
    {
      "view": "dashboard",
      "mockupPath": "../../docs/mockups/dashboard-mockup.png",
      "route": "/dashboard"
    }
  ]
}
```

## Troubleshooting

### Common Issues

#### 1. Tests Fail After UI Changes

**Problem:** Tests fail after making intentional UI changes.

**Solution:** Update baselines to reflect the new UI:

```bash
npm run test:visual:update
```

#### 2. "No baseline found" Error

**Problem:** Tests fail because no baseline exists.

**Solution:** Run tests with update flag to create initial baselines:

```bash
npm run test:visual:update
```

#### 3. Timeout Waiting for Dev Server

**Problem:** Tests timeout waiting for the dev server to start.

**Solution:** 
- Ensure no other process is using port 5173
- Check that `npm run dev` works independently
- Increase timeout in [`playwright.config.ts`](../../playwright.config.ts)

#### 4. Browser Not Installed

**Problem:** Playwright browser not found.

**Solution:** Install Playwright browsers:

```bash
npx playwright install --with-deps
```

#### 5. Flaky Tests (Inconsistent Results)

**Problem:** Tests sometimes pass, sometimes fail without code changes.

**Solution:**
- Check for dynamic content (timestamps, random data)
- Ensure animations are disabled (configured by default)
- Increase threshold slightly if needed
- Check for network-dependent content

#### 6. Mockup Path Not Found

**Problem:** Mockup comparison fails with "file not found" error.

**Solution:**
- Verify mockup paths in [`mockups.json`](mockups.json)
- Ensure mockup files exist in [`docs/mockups/`](../../../docs/mockups/)
- Check relative path resolution

### Debug Mode

Run tests in headed mode to see what's happening:

```bash
./scripts/run-visual-tests.sh --headed --verbose
```

### Inspecting Screenshots

Screenshots are saved to:

- **Baselines:** `test-results/baselines/`
- **Actuals:** `test-results/actuals/`
- **Diffs:** `test-results/diffs/`

Manually review these files to understand test failures.

## Best Practices

### 1. Update Baselines After Intentional Changes

Always update baselines after making intentional UI changes:

```bash
npm run test:visual:update
```

### 2. Review Failed Tests Carefully

Don't blindly update baselines. Review the diff to ensure changes are intentional.

### 3. Use Mockup Comparison During Development

Compare against mockups during initial development to ensure alignment with designs:

```bash
npm run test:visual:mockups
```

### 4. Run Tests Before Committing

Include visual regression tests in your pre-commit workflow:

```bash
npm run test:visual
```

### 5. Keep Thresholds Reasonable

Use the smallest threshold that works for your use case. 0.1% is a good starting point.

### 6. Test Multiple Viewports

Tests automatically run at multiple viewport sizes to ensure responsive design.

### 7. Disable Animations

Animations are disabled by default for consistent screenshots. Keep this setting.

### 8. Handle Dynamic Content

For content that changes (timestamps, counters), consider:
- Mocking the data
- Using CSS to hide dynamic elements during tests
- Increasing threshold for those specific tests

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Visual Regression Tests

on: [push, pull_request]

jobs:
  visual-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Install dependencies
        run: |
          cd int3rceptor/desktop
          npm install
      - name: Install Playwright browsers
        run: |
          cd int3rceptor/desktop
          npx playwright install --with-deps
      - name: Run visual tests
        run: |
          cd int3rceptor/desktop
          npm run test:visual
      - name: Upload test results
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: visual-test-results
          path: int3rceptor/desktop/playwright-report/
```

## Mockup Comparison System

### How It Works

The mockup comparison system:

1. Loads mockup mappings from [`mockups.json`](mockups.json)
2. Resolves mockup paths relative to the project root
3. Captures current UI screenshots
4. Compares using pixelmatch with configurable thresholds
5. Generates diff images for failed comparisons

### Supported Mockups

The following views have approved mockups:

- Dashboard ([`dashboard-mockup.png`](../../../docs/mockups/dashboard-mockup.png))
- Traffic ([`traffic-tab_mockup.png`](../../../docs/mockups/traffic-tab_mockup.png))
- Intruder ([`intruder-tab_mockup.png`](../../../docs/mockups/intruder-tab_mockup.png))
- Repeater ([`repeater-tab_mockup.png`](../../../docs/mockups/repeater-tab_mockup.png))
- WebSocket ([`WebSocket Interception Tab_mockup.png`](../../../docs/mockups/WebSocket%20Interception%20Tab_mockup.png))
- Loading ([`loading_mockup.png`](../../../docs/mockups/loading_mockup.png))

### Views Without Mockups

The following views use baseline comparison only:

- Scanner
- Settings
- Plugins
- Decoder
- Comparer

## API Reference

### Utility Functions

#### `expectVisualMatch(page, testInfo, filename, options)`

Compares screenshot against baseline.

**Parameters:**
- `page`: Playwright Page object
- `testInfo`: TestInfo object
- `filename`: Baseline filename
- `options`: Optional configuration (threshold, maxDiffPixels, etc.)

#### `expectMockupMatch(page, testInfo, view, options)`

Compares screenshot against approved mockup.

**Parameters:**
- `page`: Playwright Page object
- `testInfo`: TestInfo object
- `view`: View name (must match mockup mapping)
- `options`: Optional configuration

#### `updateBaseline(page, testInfo, filename, options)`

Updates baseline with current screenshot.

**Parameters:**
- `page`: Playwright Page object
- `testInfo`: TestInfo object
- `filename`: Baseline filename
- `options`: Optional configuration

## Contributing

When adding new visual tests:

1. Create a new test file following the naming pattern: `[view].test.ts`
2. Add mockup mapping to [`mockups.json`](mockups.json) if applicable
3. Run tests with update flag to create initial baselines
4. Review generated screenshots
5. Update this README if needed

## License

This visual regression testing suite is part of the INT3RCEPTOR project.

## Support

For issues or questions:

1. Check the troubleshooting section above
2. Review Playwright documentation: https://playwright.dev
3. Check existing issues in the project repository

## Changelog

### Version 1.0.0 (2025-01-26)

- Initial release
- Support for 11 UI views
- Mockup comparison for 6 views
- Baseline comparison for all views
- Multi-browser support (Chrome, Firefox, Safari)
- Responsive testing at multiple viewports
- HTML diff reports
- Configurable thresholds
- Executable test runner script
