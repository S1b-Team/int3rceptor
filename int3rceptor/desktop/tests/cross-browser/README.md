# Cross-Browser Testing for INT3RCEPTOR

This directory contains cross-browser compatibility tests for the INT3RCEPTOR desktop application using Playwright.

## Overview

The cross-browser testing suite ensures that INT3RCEPTOR works correctly across different web browsers including:
- **Chrome** (Chromium)
- **Firefox**
- **Edge** (Microsoft Edge)
- **Safari** (WebKit)

## Configuration

The Playwright configuration is defined in [`../playwright.config.ts`](../playwright.config.ts) and includes:

### Browser Projects

```typescript
projects: [
  {
    name: 'cross-browser-chrome',
    testDir: './tests/cross-browser',
    use: { ...devices['Desktop Chrome'], channel: 'chrome' },
  },
  {
    name: 'cross-browser-firefox',
    testDir: './tests/cross-browser',
    use: { ...devices['Desktop Firefox'], channel: 'firefox' },
  },
  {
    name: 'cross-browser-edge',
    testDir: './tests/cross-browser',
    use: { ...devices['Desktop Edge'], channel: 'msedge' },
  },
  {
    name: 'cross-browser-safari',
    testDir: './tests/cross-browser',
    use: { ...devices['Desktop Safari'], channel: 'webkit' },
  },
]
```

## Available Scripts

Run cross-browser tests from the root of the `desktop` directory:

```bash
# Run all cross-browser tests on all configured browsers
npm run test:cross-browser

# Run tests only on Chrome
npm run test:cross-browser:chrome

# Run tests only on Firefox
npm run test:cross-browser:firefox

# Run tests only on Edge
npm run test:cross-browser:edge

# Run tests only on Safari
npm run test:cross-browser:safari

# Run tests with UI mode
npm run test:cross-browser:ui

# View test reports
npm run test:cross-browser:report
```

## Test Files

Each major feature of INT3RCEPTOR has its own test file:

| Test File | Description |
|-----------|-------------|
| [`dashboard.test.ts`](dashboard.test.ts) | Dashboard view rendering, stats, charts, navigation |
| [`traffic.test.ts`](traffic.test.ts) | Traffic interception table, filters, data display |
| [`intruder.test.ts`](intruder.test.ts) | Intruder tool tabs, attack configuration, payload handling |
| [`rest.test.ts`](rest.test.ts) | REST client functionality, HTTP methods, request/response handling |
| [`scanner.test.ts`](scanner.test.ts) | Scanner options, settings, scan execution |
| [`settings.test.ts`](settings.test.ts) | Settings panels, form inputs, configuration persistence |
| [`plugins.test.ts`](plugins.test.ts) | Plugin management, installation, configuration |
| [`decoder.test.ts`](decoder.test.ts) | Decoder/encoder functionality, format conversions |
| [`comparer.test.ts`](comparer.test.ts) | Diff comparison tool, side-by-side panels |
| [`utils.ts`](utils.ts) | Utility functions for cross-browser testing |

## Utilities

The [`utils.ts`](utils.ts) file provides helper functions for cross-browser testing:

### Key Functions

- **`getBrowserInfo(page)`** - Get browser name, version, user agent, and platform
- **`captureComparisonScreenshot(page, testName, browserName)`** - Capture screenshots for comparison
- **`isElementVisibleAndInteractive(page, selector)`** - Check if element is visible and interactive
- **`waitForElementInteractive(page, selector, timeout)`** - Wait for element to be ready
- **`testResponsiveLayout(page, testSelector, viewports)`** - Test layout at different viewports
- **`testFormInput(page, formSelector, inputData)`** - Test form input and validation
- **`testTabNavigation(page, tabSelector, expectedContentSelector)`** - Test tab switching
- **`checkFeatureSupport(page, feature)`** - Check browser feature support
- **`getBrowserKnownIssues(browserName)`** - Get known issues for specific browsers

### Test Fixtures

- **`testDataFixtures`** - Sample data for consistent testing
- **`testViewports`** - Standard viewport sizes for responsive testing
- **`viewSelectors`** - Common view selectors
- **`navSelectors`** - Navigation selectors

## Test Coverage

Each test file includes:

1. **Rendering Tests** - Verify UI elements render correctly
2. **Interaction Tests** - Test buttons, forms, and interactive elements
3. **Navigation Tests** - Verify navigation between views
4. **Responsive Tests** - Test layout at different viewport sizes
5. **Browser-Specific Tests** - Handle browser-specific edge cases
6. **Accessibility Tests** - Check ARIA attributes and accessibility
7. **Screenshot Tests** - Capture screenshots for visual comparison

## Browser-Specific Considerations

### Chrome/Chromium
- Uses `channel: 'chrome'` for latest Chrome
- Generally has the best feature support
- CSS Grid and Flexbox fully supported

### Firefox
- Uses `channel: 'firefox'`
- May have minor CSS transition timing differences
- Flexbox gap property supported in recent versions

### Edge
- Uses `channel: 'msedge'`
- Chromium-based, similar to Chrome
- May have minor rendering differences

### Safari
- Uses `channel: 'webkit'`
- Form input styling may differ
- Scrollbar styling is limited
- Some CSS filters may not render identically

## Reports

Test results are saved in the following locations:

- **HTML Report**: `test-results/html/`
- **JSON Report**: `test-results/results.json`
- **Screenshots**: `tests/cross-browser/screenshots/`
- **Compatibility Reports**: `tests/cross-browser/reports/`

View the HTML report with:
```bash
npm run test:cross-browser:report
```

## Known Issues

Each browser may have known issues that are tracked in the test suite. See `getBrowserKnownIssues()` in [`utils.ts`](utils.ts) for details.

## CI/CD Integration

For CI/CD pipelines, use:
```bash
npm run test:cross-browser
```

Tests will run with:
- `retries: 2` (on CI)
- `workers: 1` (on CI)
- Automatic retry on first failure

## Adding New Tests

To add a new cross-browser test:

1. Create a new test file in this directory (e.g., `newfeature.test.ts`)
2. Import utilities from [`utils.ts`](utils.ts)
3. Use the standard Playwright test structure:
   ```typescript
   import { test, expect } from '@playwright/test';
   import { getBrowserInfo, testViewports } from './utils';

   test.describe('New Feature - Cross-Browser Tests', () => {
     test.beforeEach(async ({ page, browserName }) => {
       const browserInfo = await getBrowserInfo(page);
       console.log(`Testing on ${browserName} ${browserInfo.version}`);
       await page.goto('/');
     });

     test('should work correctly', async ({ page }) => {
       // Your test code here
     });
   });
   ```

## Troubleshooting

### Browser Not Found

If a browser is not found, install Playwright browsers:
```bash
npx playwright install
```

Or install specific browsers:
```bash
npx playwright install chrome firefox webkit msedge
```

### Tests Timing Out

Increase timeouts in [`playwright.config.ts`](../playwright.config.ts):
```typescript
use: {
  actionTimeout: 10 * 1000,
  navigationTimeout: 30 * 1000,
}
```

### Screenshots Not Saving

Ensure the screenshots directory exists:
```bash
mkdir -p tests/cross-browser/screenshots
```

## Contributing

When adding new features to INT3RCEPTOR, ensure cross-browser compatibility by:

1. Adding tests to the appropriate test file
2. Testing on all supported browsers
3. Checking for browser-specific issues
4. Updating this README if new browser-specific considerations are needed

## Additional Resources

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Cross-Browser Testing](https://playwright.dev/docs/test-configuration#projects)
- [INT3RCEPTOR Documentation](../../../docs/)
