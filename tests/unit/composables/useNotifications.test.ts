import { useNotifications } from '@/composables/useNotifications';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * Test suite for useNotifications composable
 * Tests notification creation, auto-dismiss, manual dismiss, and queue management
 */

describe('useNotifications', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  // ==========================================
  // Test Suite 1: Initialization
  // ==========================================

  describe('Initialization', () => {
    it('should initialize with empty notifications array', () => {
      const { notifications } = useNotifications();

      expect(notifications.value).toEqual([]);
    });

    it('should initialize with notificationCount of 0', () => {
      const { notificationCount } = useNotifications();

      expect(notificationCount.value).toBe(0);
    });

    it('should initialize with hasErrors as false', () => {
      const { hasErrors } = useNotifications();

      expect(hasErrors.value).toBe(false);
    });
  });

  // ==========================================
  // Test Suite 2: Show Notification
  // ==========================================

  describe('Show Notification', () => {
    it('should add notification with default type', () => {
      const { notifications, showNotification } = useNotifications();

      showNotification('Test message');

      expect(notifications.value).toHaveLength(1);
      expect(notifications.value[0].message).toBe('Test message');
      expect(notifications.value[0].type).toBe('info');
    });

    it('should add notification with specified type', () => {
      const { notifications, showNotification } = useNotifications();

      showNotification('Error message', 'error');

      expect(notifications.value[0].type).toBe('error');
    });

    it('should generate unique ID for each notification', () => {
      const { notifications, showNotification } = useNotifications();

      const id1 = showNotification('First');
      const id2 = showNotification('Second');

      expect(id1).not.toBe(id2);
      expect(notifications.value[0].id).toBe(id1);
      expect(notifications.value[1].id).toBe(id2);
    });

    it('should set timestamp on notification', () => {
      const { notifications, showNotification } = useNotifications();

      const before = Date.now();
      showNotification('Test');
      const after = Date.now();

      expect(notifications.value[0].timestamp).toBeGreaterThanOrEqual(before);
      expect(notifications.value[0].timestamp).toBeLessThanOrEqual(after);
    });

    it('should return notification ID', () => {
      const { showNotification } = useNotifications();

      const id = showNotification('Test');

      expect(id).toBeDefined();
      expect(typeof id).toBe('string');
      expect(id.startsWith('notif_')).toBe(true);
    });
  });

  // ==========================================
  // Test Suite 3: Convenience Methods
  // ==========================================

  describe('Convenience Methods', () => {
    it('should add success notification', () => {
      const { notifications, showSuccess } = useNotifications();

      showSuccess('Success message');

      expect(notifications.value[0].type).toBe('success');
    });

    it('should add error notification', () => {
      const { notifications, showError } = useNotifications();

      showError('Error message');

      expect(notifications.value[0].type).toBe('error');
    });

    it('should add warning notification', () => {
      const { notifications, showWarning } = useNotifications();

      showWarning('Warning message');

      expect(notifications.value[0].type).toBe('warning');
    });

    it('should add info notification', () => {
      const { notifications, showInfo } = useNotifications();

      showInfo('Info message');

      expect(notifications.value[0].type).toBe('info');
    });
  });

  // ==========================================
  // Test Suite 4: Auto-Dismiss
  // ==========================================

  describe('Auto-Dismiss', () => {
    it('should auto-dismiss after default duration', () => {
      const { notifications, showNotification } = useNotifications();

      showNotification('Test');

      expect(notifications.value).toHaveLength(1);

      // Default duration is 5000ms for info
      vi.advanceTimersByTime(5100);

      expect(notifications.value).toHaveLength(0);
    });

    it('should use shorter duration for success', () => {
      const { notifications, showSuccess } = useNotifications();

      showSuccess('Test');

      // Success duration is 4000ms
      vi.advanceTimersByTime(4100);

      expect(notifications.value).toHaveLength(0);
    });

    it('should use longer duration for error', () => {
      const { notifications, showError } = useNotifications();

      showError('Test');

      // Error duration is 7000ms
      vi.advanceTimersByTime(5000);
      expect(notifications.value).toHaveLength(1);

      vi.advanceTimersByTime(2100);
      expect(notifications.value).toHaveLength(0);
    });

    it('should respect custom duration', () => {
      const { notifications, showNotification } = useNotifications();

      showNotification('Test', 'info', 2000);

      vi.advanceTimersByTime(2100);

      expect(notifications.value).toHaveLength(0);
    });
  });

  // ==========================================
  // Test Suite 5: Manual Dismiss
  // ==========================================

  describe('Manual Dismiss', () => {
    it('should dismiss notification by ID', () => {
      const { notifications, showNotification, dismissNotification } = useNotifications();

      const id = showNotification('Test');

      expect(notifications.value).toHaveLength(1);

      dismissNotification(id);

      expect(notifications.value).toHaveLength(0);
    });

    it('should not throw on invalid ID', () => {
      const { dismissNotification } = useNotifications();

      expect(() => {
        dismissNotification('invalid-id');
      }).not.toThrow();
    });

    it('should cancel auto-dismiss timeout when manually dismissed', () => {
      const { notifications, showNotification, dismissNotification } = useNotifications();

      const id = showNotification('Test', 'info', 5000);
      dismissNotification(id);

      // Advance past auto-dismiss time
      vi.advanceTimersByTime(6000);

      // Should still be empty (no error from orphaned timeout)
      expect(notifications.value).toHaveLength(0);
    });
  });

  // ==========================================
  // Test Suite 6: Dismiss All
  // ==========================================

  describe('Dismiss All', () => {
    it('should dismiss all notifications', () => {
      const { notifications, showNotification, dismissAll } = useNotifications();

      showNotification('First');
      showNotification('Second');
      showNotification('Third');

      expect(notifications.value).toHaveLength(3);

      dismissAll();

      expect(notifications.value).toHaveLength(0);
    });

    it('should clear all pending timeouts', () => {
      const { notifications, showNotification, dismissAll } = useNotifications();

      showNotification('First');
      showNotification('Second');

      dismissAll();

      // Advance time past all possible auto-dismiss
      vi.advanceTimersByTime(10000);

      // Should not throw or have any issues
      expect(notifications.value).toHaveLength(0);
    });
  });

  // ==========================================
  // Test Suite 7: Dismiss By Type
  // ==========================================

  describe('Dismiss By Type', () => {
    it('should dismiss notifications of specific type', () => {
      const { notifications, showNotification, dismissByType } = useNotifications();

      showNotification('Error 1', 'error');
      showNotification('Info 1', 'info');
      showNotification('Error 2', 'error');

      expect(notifications.value).toHaveLength(3);

      dismissByType('error');

      expect(notifications.value).toHaveLength(1);
      expect(notifications.value[0].type).toBe('info');
    });

    it('should not affect other types', () => {
      const { notifications, showSuccess, showError, dismissByType } = useNotifications();

      showSuccess('Success');
      showError('Error');

      dismissByType('warning');

      expect(notifications.value).toHaveLength(2);
    });
  });

  // ==========================================
  // Test Suite 8: Computed Properties
  // ==========================================

  describe('Computed Properties', () => {
    it('should update notificationCount reactively', () => {
      const { notificationCount, showNotification, dismissNotification } = useNotifications();

      expect(notificationCount.value).toBe(0);

      const id1 = showNotification('First');
      expect(notificationCount.value).toBe(1);

      showNotification('Second');
      expect(notificationCount.value).toBe(2);

      dismissNotification(id1);
      expect(notificationCount.value).toBe(1);
    });

    it('should update hasErrors when errors present', () => {
      const { hasErrors, showError, dismissAll } = useNotifications();

      expect(hasErrors.value).toBe(false);

      showError('Error');
      expect(hasErrors.value).toBe(true);

      dismissAll();
      expect(hasErrors.value).toBe(false);
    });

    it('should hasErrors be false with only non-error notifications', () => {
      const { hasErrors, showSuccess, showInfo, showWarning } = useNotifications();

      showSuccess('Success');
      showInfo('Info');
      showWarning('Warning');

      expect(hasErrors.value).toBe(false);
    });
  });

  // ==========================================
  // Test Suite 9: Max Notifications Limit
  // ==========================================

  describe('Max Notifications Limit', () => {
    it('should enforce max notification limit', () => {
      const { notifications, showNotification } = useNotifications();

      // Add 15 notifications (max is 10)
      for (let i = 0; i < 15; i++) {
        showNotification(`Message ${i}`);
      }

      expect(notifications.value.length).toBeLessThanOrEqual(10);
    });

    it('should remove oldest notifications when limit exceeded', () => {
      const { notifications, showNotification } = useNotifications();

      // Add 12 notifications
      for (let i = 0; i < 12; i++) {
        showNotification(`Message ${i}`);
      }

      // The first 2 should have been removed
      expect(notifications.value.some((n) => n.message === 'Message 0')).toBe(false);
      expect(notifications.value.some((n) => n.message === 'Message 1')).toBe(false);
      expect(notifications.value.some((n) => n.message === 'Message 11')).toBe(true);
    });
  });

  // ==========================================
  // Test Suite 10: Return Interface
  // ==========================================

  describe('Return Interface', () => {
    it('should return all expected properties', () => {
      const result = useNotifications();

      expect(result).toHaveProperty('notifications');
      expect(result).toHaveProperty('notificationCount');
      expect(result).toHaveProperty('hasErrors');
      expect(result).toHaveProperty('showNotification');
      expect(result).toHaveProperty('showSuccess');
      expect(result).toHaveProperty('showError');
      expect(result).toHaveProperty('showWarning');
      expect(result).toHaveProperty('showInfo');
      expect(result).toHaveProperty('dismissNotification');
      expect(result).toHaveProperty('dismissAll');
      expect(result).toHaveProperty('dismissByType');
    });
  });

  // ==========================================
  // Test Suite 11: Edge Cases
  // ==========================================

  describe('Edge Cases', () => {
    it('should handle rapid notifications', () => {
      const { notifications, showNotification } = useNotifications();

      for (let i = 0; i < 100; i++) {
        showNotification(`Message ${i}`);
      }

      // Should have enforced limit
      expect(notifications.value.length).toBeLessThanOrEqual(10);
    });

    it('should handle empty message', () => {
      const { notifications, showNotification } = useNotifications();

      showNotification('');

      expect(notifications.value).toHaveLength(1);
      expect(notifications.value[0].message).toBe('');
    });

    it('should handle very long message', () => {
      const { notifications, showNotification } = useNotifications();

      const longMessage = 'A'.repeat(10000);
      showNotification(longMessage);

      expect(notifications.value[0].message).toBe(longMessage);
    });

    it('should handle special characters in message', () => {
      const { notifications, showNotification } = useNotifications();

      const specialMessage = '<script>alert("XSS")</script>';
      showNotification(specialMessage);

      expect(notifications.value[0].message).toBe(specialMessage);
    });
  });
});
