# INT3RCEPTOR Accessibility Tests

Comprehensive accessibility testing suite for INT3RCEPTOR desktop application, validating WCAG 2.1 Level AA compliance across all 10 views.

## Overview

This accessibility test suite ensures that INT3RCEPTOR is usable by everyone, including people with disabilities. The tests validate:

- **Keyboard Navigation** - All functionality accessible via Tab, Enter, and Escape keys
- **Focus Indicators** - Visible focus states for all interactive elements
- **ARIA Labels** - Proper labeling for screen readers
- **Contrast Ratios** - Sufficient color contrast for readability
- **Semantic HTML** - Proper use of HTML elements and ARIA attributes

## Tools Used

- **@axe-core/playwright** - Automated accessibility testing integrated with Playwright
- **pa11y-ci** - Command-line accessibility testing with HTML reports
- **Playwright** - Browser automation and test framework

## Test Coverage

The suite includes comprehensive tests for all 10 INT3RCEPTOR views:

| View | Test File | Coverage |
|-------|-----------|-----------|
| Dashboard | [`dashboard.test.ts`](dashboard.test.ts) | Stats, charts, navigation, cards |
| Traffic | [`traffic.test.ts`](traffic.test.ts) | Tables, filters, pagination, search |
| Intruder | [`intruder.test.ts`](intruder.test.ts) | Tabs, attack controls, payload editors |
| REST | [`rest.test.ts`](rest.test.ts) | Method selector, URL input, headers, body |
| Scanner | [`scanner.test.ts`](scanner.test.ts) | Scan options, targets, results, progress |
| Settings | [`settings.test.ts`](settings.test.ts) | Forms, toggles, sliders, fieldsets |
| Plugins | [`plugins.test.ts`](plugins.test.ts) | Plugin list, install/uninstall, search |
| Decoder | [`decoder.test.ts`](decoder.test.ts) | Format selector, input/output, encode/decode |
| Comparer | [`comparer.test.ts`](comparer.test.ts) | Left/right panels, diff view, compare controls |
| WebSocket | [`websocket.test.ts`](websocket.test.ts) | Connection, messages, history, filters |

## Running Tests

### Run All Accessibility Tests

```bash
# Using Playwright with axe-core
npm run test:a11y

# Using pa11y-ci
npm run test:a11y:pa11y
```

### Run Specific View Tests

```bash
# Run only Dashboard accessibility tests
npx playwright test --project=accessibility-chrome dashboard

# Run only Traffic accessibility tests
npx playwright test --project=accessibility-chrome traffic
```

### View HTML Reports

```bash
# View Playwright HTML report
npm run test:a11y:report

# View pa11y-ci HTML reports
open test-results/pa11y/reports/
```

## Test Categories

### 1. Axe-Core Violations

Automated accessibility scans using axe-core with WCAG 2.1 Level AA rules:

```typescript
const results = await runAxeScan(page, {
  tags: WCAG_2_1_AA_TAGS
});
```

**Validated Rules:**
- Color contrast (WCAG 2.1 AA)
- Keyboard navigation
- ARIA labels and roles
- Form labels
- Image alt text
- Heading hierarchy
- Table headers
- Link accessibility

### 2. Keyboard Navigation Tests

Validates that all interactive elements are accessible via keyboard:

```typescript
const result = await testKeyboardNavigation(page, selector);
```

**Tests:**
- Tab navigation through all focusable elements
- Enter key activates buttons and links
- Escape key dismisses modals and dialogs
- Arrow key navigation for lists and menus
- Focus order follows logical sequence

### 3. Focus Indicator Tests

Ensures visible focus states for all interactive elements:

```typescript
const result = await testFocusIndicators(page, selector);
```

**Validates:**
- Visible outline on focus
- Box shadow or border color change
- High contrast focus indicators
- Consistent focus styles across elements

### 4. ARIA Label Tests

Validates proper ARIA attributes for screen readers:

```typescript
const result = await testARIALabels(page, selector);
```

**Checks:**
- Buttons have accessible names (text, aria-label, or aria-labelledby)
- Inputs have associated labels
- Images have alt text (unless decorative)
- Links have descriptive text
- Regions have aria-label or aria-labelledby

### 5. Contrast Ratio Tests

Validates sufficient color contrast for readability:

```typescript
const result = await testContrastRatios(page, selector);
```

**Standards:**
- WCAG 2.1 Level AA: 4.5:1 for normal text, 3:1 for large text
- WCAG 2.1 Level AAA: 7:1 for normal text, 4.5:1 for large text

## Accessibility Features Tested

### Keyboard Accessibility

- **Tab Navigation**: Sequential focus through all interactive elements
- **Enter Key**: Activates buttons and links
- **Escape Key**: Closes modals and cancels actions
- **Arrow Keys**: Navigates lists, menus, and grids
- **Space Bar**: Toggles checkboxes and radio buttons

### Screen Reader Support

- **Semantic HTML**: Proper use of headings, lists, and landmarks
- **ARIA Roles**: Correct role attributes for custom components
- **ARIA Labels**: Descriptive labels for all controls
- **ARIA States**: Current state communicated (expanded, checked, etc.)
- **Live Regions**: Dynamic content announced to screen readers

### Visual Accessibility

- **Color Contrast**: Minimum 4.5:1 for normal text
- **Focus Indicators**: Visible focus on all interactive elements
- **Text Sizing**: Resizable up to 200% without loss of content
- **No Color-Only Information**: Color not used as the only means of conveying information

### Form Accessibility

- **Labels**: All inputs have associated labels
- **Error Messages**: Associated with inputs using aria-describedby
- **Required Fields**: Clearly indicated
- **Validation**: Real-time feedback with ARIA attributes
- **Fieldsets**: Related controls grouped with fieldset/legend

## Configuration

### Playwright Configuration

The accessibility tests are configured in [`playwright.config.ts`](../playwright.config.ts):

```typescript
{
  name: 'accessibility-chrome',
  testDir: './tests/accessibility',
  use: {
    ...devices['Desktop Chrome'],
    channel: 'chrome',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
}
```

### pa11y-ci Configuration

The pa11y-ci configuration is in [`.pa11yci.json`](../.pa11yci.json):

```json
{
  "defaults": {
    "standard": "WCAG2AA",
    "threshold": 0,
    "reporters": ["html", "json"]
  }
}
```

## Reports

### Playwright Reports

- **Location**: `test-results/html/`
- **Format**: Interactive HTML report
- **Contents**: Test results, screenshots, and videos

### pa11y-ci Reports

- **Location**: `test-results/pa11y/reports/`
- **Format**: HTML and JSON
- **Contents**: Violations, passes, and recommendations

### Accessibility Violation Reports

Each test generates detailed reports:

```json
{
  "testName": "dashboard",
  "timestamp": "2024-01-26T16:00:00.000Z",
  "violations": 0,
  "passes": 145,
  "incomplete": 2,
  "violationDetails": []
}
```

## Utilities

The [`utils.ts`](utils.ts) file provides helper functions:

### `runAxeScan(page, options)`

Run axe-core accessibility scan with custom options.

### `testKeyboardNavigation(page, selector)`

Test keyboard navigation through focusable elements.

### `testFocusIndicators(page, selector)`

Validate visible focus indicators on interactive elements.

### `testARIALabels(page, selector)`

Check for proper ARIA labels and attributes.

### `testContrastRatios(page, selector)`

Validate color contrast ratios meet WCAG standards.

### `runComprehensiveA11yTest(page, selector, testName, testInfo)`

Run complete accessibility test suite with report generation.

## WCAG 2.1 Level AA Compliance

The test suite validates compliance with WCAG 2.1 Level AA success criteria:

### Perceivable

- **1.1.1**: Non-text Content - Images have alt text
- **1.3.1**: Info and Relationships - Semantic HTML and ARIA
- **1.3.2**: Meaningful Sequence - Logical reading order
- **1.4.3**: Contrast (Minimum) - 4.5:1 for normal text

### Operable

- **2.1.1**: Keyboard - All functionality available via keyboard
- **2.1.2**: No Keyboard Trap - Focus can move away from all elements
- **2.4.3**: Focus Order - Logical tab order
- **2.5.5**: Target Size - Touch targets at least 44x44 pixels

### Understandable

- **3.1.1**: Language of Page - HTML lang attribute
- **3.2.1**: On Focus - No unexpected focus changes
- **3.3.2**: Labels or Instructions - Clear labels for inputs

### Robust

- **4.1.2**: Name, Role, Value - ARIA attributes are valid

## Best Practices

### Writing Accessible Code

1. **Use Semantic HTML**
   ```html
   <button>Submit</button> <!-- Good -->
   <div onclick="submit()">Submit</div> <!-- Bad -->
   ```

2. **Provide Labels for Inputs**
   ```html
   <label for="email">Email</label>
   <input id="email" type="email">
   ```

3. **Use ARIA Attributes Correctly**
   ```html
   <button aria-label="Close dialog" aria-expanded="false">
     <span aria-hidden="true">×</span>
   </button>
   ```

4. **Ensure Sufficient Contrast**
   ```css
   /* Good: 4.5:1 contrast ratio */
   .text { color: #333; background: #fff; }
   ```

5. **Test with Keyboard**
   - Can you navigate to all elements with Tab?
   - Can you activate buttons with Enter?
   - Can you dismiss modals with Escape?

## Troubleshooting

### Common Issues

**Issue**: Tests fail due to missing ARIA labels

**Solution**: Add `aria-label`, `aria-labelledby`, or text content to interactive elements.

**Issue**: Focus indicators not visible

**Solution**: Add CSS for `:focus` state with visible outline or border.

**Issue**: Contrast ratio violations

**Solution**: Increase contrast between text and background colors.

**Issue**: Keyboard navigation issues

**Solution**: Ensure all interactive elements are in the tab order and have proper tabindex.

## Continuous Integration

### GitHub Actions

Add accessibility tests to CI:

```yaml
- name: Run accessibility tests
  run: npm run test:a11y
```

### Pre-commit Hooks

Run accessibility tests before committing:

```json
{
  "husky": {
    "hooks": {
      "pre-commit": "npm run test:a11y"
    }
  }
}
```

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [axe-core Documentation](https://www.deque.com/axe/)
- [pa11y Documentation](https://github.com/pa11y/pa11y)
- [Playwright Accessibility](https://playwright.dev/docs/accessibility-testing)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)

## Contributing

When adding new features or views:

1. Create accessibility test file in `tests/accessibility/`
2. Include tests for keyboard navigation, focus indicators, ARIA labels, and contrast
3. Update this README with new test coverage
4. Run `npm run test:a11y` to verify compliance
5. Ensure all tests pass before merging

## License

This accessibility test suite is part of INT3RCEPTOR and follows the same license.
