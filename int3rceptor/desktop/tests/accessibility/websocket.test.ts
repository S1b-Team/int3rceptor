import { test, expect } from '@playwright/test';
import {
  runComprehensiveA11yTest,
  viewSelectors,
  navSelectors,
  navigateToView,
  WCAG_2_1_AA_TAGS,
  runAxeScan,
  testKeyboardNavigation,
  testFocusIndicators,
  testARIALabels,
  testContrastRatios
} from './utils';

/**
 * Accessibility tests for WebSocket view
 * Validates WCAG 2.1 Level AA compliance
 */

test.describe('WebSocket - Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await navigateToView(page, 'websocket', navSelectors.websocket, viewSelectors.websocket);
  });

  test('should have no axe-core violations', async ({ page }) => {
    const results = await runAxeScan(page, {
      tags: WCAG_2_1_AA_TAGS
    });

    expect(results.violations.length).toBe(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    const result = await testKeyboardNavigation(page, viewSelectors.websocket);

    expect(result.passed).toBeTruthy();
    expect(result.focusOrder.length).toBeGreaterThan(0);
  });

  test('should have visible focus indicators', async ({ page }) => {
    const result = await testFocusIndicators(page, viewSelectors.websocket);

    expect(result.passed).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    const result = await testARIALabels(page, viewSelectors.websocket);

    expect(result.passed).toBeTruthy();
  });

  test('should have sufficient contrast ratios', async ({ page }) => {
    const result = await testContrastRatios(page, viewSelectors.websocket);

    expect(result.passed).toBeTruthy();
  });

  test('should have accessible connection controls', async ({ page }) => {
    const connectButton = page.locator('button:has-text("Connect"), button[data-testid*="connect"]');
    const hasAccessibleName = await connectButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();

    const disconnectButton = page.locator('button:has-text("Disconnect"), button[data-testid*="disconnect"]');
    const disconnectAccessibleName = await disconnectButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(disconnectAccessibleName).toBeTruthy();
  });

  test('should have accessible URL input', async ({ page }) => {
    const urlInput = page.locator('input[type="url"], input[aria-label*="url"], input[data-testid*="url"]');
    const hasLabel = await urlInput.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible message input', async ({ page }) => {
    const messageInput = page.locator('textarea[aria-label*="message"], input[aria-label*="message"], [data-testid*="message"]');
    const hasLabel = await messageInput.evaluate((el) => {
      const id = el.id;
      const hasAriaLabel = el.hasAttribute('aria-label');
      const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
    });

    expect(hasLabel).toBeTruthy();
  });

  test('should have accessible send button', async ({ page }) => {
    const sendButton = page.locator('button:has-text("Send"), button[data-testid*="send"]');
    const hasAccessibleName = await sendButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible clear button', async ({ page }) => {
    const clearButton = page.locator('button:has-text("Clear"), button[aria-label*="clear"]');
    const hasAccessibleName = await clearButton.evaluate((el) => {
      return el.textContent?.trim().length > 0 ||
             el.hasAttribute('aria-label');
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible message list', async ({ page }) => {
    const messageList = page.locator('[data-testid*="messages"], [role="log"], [role="list"]');
    const hasAccessibleName = await messageList.evaluate((el) => {
      return el.hasAttribute('aria-label') ||
             el.hasAttribute('aria-labelledby') ||
             el.hasAttribute('aria-live');
    });

    expect(hasAccessibleName).toBeTruthy();

    // Check if list has aria-live for dynamic content
    const ariaLive = await messageList.getAttribute('aria-live');
    if (ariaLive) {
      expect(['polite', 'assertive', 'off'].includes(ariaLive)).toBeTruthy();
    }
  });

  test('should have accessible message items', async ({ page }) => {
    const messageItems = page.locator('[data-testid*="message-item"], [role="listitem"], [role="log"]');
    const count = await messageItems.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const item = messageItems.nth(i);
      const hasAccessibleName = await item.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible connection status', async ({ page }) => {
    const statusIndicator = page.locator('[data-testid*="status"], [role="status"]');
    const hasAccessibleName = await statusIndicator.evaluate((el) => {
      return el.hasAttribute('aria-label') ||
             el.textContent?.trim().length > 0;
    });

    expect(hasAccessibleName).toBeTruthy();
  });

  test('should have accessible message types', async ({ page }) => {
    const typeIndicators = page.locator('[data-testid*="type"], [aria-label*="type"]');
    const count = await typeIndicators.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const indicator = typeIndicators.nth(i);
      const hasAccessibleName = await indicator.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible timestamps', async ({ page }) => {
    const timestamps = page.locator('[data-testid*="timestamp"], time');
    const count = await timestamps.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      const timestamp = timestamps.nth(i);
      const hasAccessibleName = await timestamp.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('datetime') ||
               el.textContent?.trim().length > 0;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible history controls', async ({ page }) => {
    const historyButtons = page.locator('button:has-text("History"), button[aria-label*="history"]');
    const count = await historyButtons.count();

    for (let i = 0; i < count; i++) {
      const button = historyButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible export controls', async ({ page }) => {
    const exportButtons = page.locator('button:has-text("Export"), button[aria-label*="export"]');
    const count = await exportButtons.count();

    for (let i = 0; i < count; i++) {
      const button = exportButtons.nth(i);
      const hasAccessibleName = await button.evaluate((el) => {
        return el.textContent?.trim().length > 0 ||
               el.hasAttribute('aria-label');
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have accessible filter controls', async ({ page }) => {
    const filters = page.locator('select[aria-label*="filter"], [role="combobox"]');
    const count = await filters.count();

    for (let i = 0; i < count; i++) {
      const filter = filters.nth(i);
      const hasLabel = await filter.evaluate((el) => {
        const id = el.id;
        const hasAriaLabel = el.hasAttribute('aria-label');
        const hasAriaLabelledby = el.hasAttribute('aria-labelledby');
        const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

        return hasAriaLabel || hasAriaLabelledby || hasAssociatedLabel;
      });

      expect(hasLabel).toBeTruthy();
    }
  });

  test('should have accessible connection details', async ({ page }) => {
    const connectionDetails = page.locator('[data-testid*="connection"], [role="region"]');
    const exists = await connectionDetails.count() > 0;

    if (exists) {
      const hasAccessibleName = await connectionDetails.evaluate((el) => {
        return el.hasAttribute('aria-label') ||
               el.hasAttribute('aria-labelledby') ||
               el.querySelector('h1, h2, h3') !== null;
      });

      expect(hasAccessibleName).toBeTruthy();
    }
  });

  test('should have comprehensive accessibility compliance', async ({ page, testInfo }) => {
    await runComprehensiveA11yTest(
      page,
      viewSelectors.websocket,
      'websocket',
      testInfo
    );
  });
});
