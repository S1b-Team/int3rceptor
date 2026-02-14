import { useDashboardConnections } from '@/composables/dashboard/useDashboardConnections';
import type { ConnectionBreakdown, ConnectionStats } from '@/types/dashboard';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * Test suite for useDashboardConnections composable
 * Tests connection stats fetching, utilization calculation, and lifecycle management
 */

vi.mock('@/composables/useApi', () => ({
  api: {
    get: vi.fn(),
  },
}));

// Mock Vue lifecycle hooks
const mockOnBeforeUnmount = vi.fn();
vi.mock('vue', async () => {
  const actual = await vi.importActual('vue');
  return {
    ...actual,
    onBeforeUnmount: mockOnBeforeUnmount,
  };
});

import { api } from '@/composables/useApi';

describe('useDashboardConnections', () => {
  let mockApi: any;

  const mockConnectionStats: ConnectionStats = {
    active: 250,
    established: 38,
    pending: 12,
    closing: 5,
    failed: 2,
    total: 1500,
    concurrent_limit: 1000,
    rate_per_sec: 45.5,
    avg_duration_ms: 150.2,
  };

  const mockBreakdown: ConnectionBreakdown = {
    by_protocol: {
      http: 100,
      https: 120,
      websocket: 30,
    },
    by_state: {
      idle: 50,
      active: 180,
      waiting: 20,
    },
  };

  beforeEach(() => {
    mockApi = api as any;
    vi.clearAllMocks();
    vi.clearAllTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  // ==========================================
  // Test Suite 1: Initialization
  // ==========================================

  describe('Initialization', () => {
    it('should initialize with null/empty state', () => {
      const { connections, breakdown, isLoading, error, lastUpdated } =
        useDashboardConnections();

      expect(connections.value).toBeNull();
      expect(breakdown.value).toBeNull();
      expect(isLoading.value).toBe(false);
      expect(error.value).toBeNull();
      expect(lastUpdated.value).toBe(0);
    });
  });

  // ==========================================
  // Test Suite 2: Successful Fetching
  // ==========================================

  describe('Successful Connection Fetching', () => {
    it('should fetch connections successfully', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats,
          breakdown: mockBreakdown,
        },
      });

      const { connections, breakdown, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(connections.value).toEqual(mockConnectionStats);
      expect(breakdown.value).toEqual(mockBreakdown);
      expect(mockApi.get).toHaveBeenCalledWith('/api/dashboard/connections');
    });

    it('should set lastUpdated timestamp on successful fetch', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats,
        },
      });

      const beforeFetch = Date.now();
      const { lastUpdated, fetchConnections } = useDashboardConnections();

      await fetchConnections();
      const afterFetch = Date.now();

      expect(lastUpdated.value).toBeGreaterThanOrEqual(beforeFetch);
      expect(lastUpdated.value).toBeLessThanOrEqual(afterFetch);
    });

    it('should clear error on successful fetch', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { error, fetchConnections } = useDashboardConnections();
      error.value = 'Previous error';

      await fetchConnections();

      expect(error.value).toBeNull();
    });

    it('should handle missing breakdown', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats,
          // No breakdown
        },
      });

      const { connections, breakdown, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(connections.value).toEqual(mockConnectionStats);
      expect(breakdown.value).toBeNull();
    });
  });

  // ==========================================
  // Test Suite 3: Utilization Calculation
  // ==========================================

  describe('Utilization Calculation', () => {
    it('should calculate utilization percentage correctly', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats, // active: 250, concurrent_limit: 1000
        },
      });

      const { utilizationPercent, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      // 250 / 1000 * 100 = 25%
      expect(utilizationPercent.value).toBe(25);
    });

    it('should return 0 if no data', () => {
      const { utilizationPercent } = useDashboardConnections();

      expect(utilizationPercent.value).toBe(0);
    });

    it('should return 0 if concurrent_limit is 0', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: {
            ...mockConnectionStats,
            concurrent_limit: 0,
          },
        },
      });

      const { utilizationPercent, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(utilizationPercent.value).toBe(0);
    });

    it('should round utilization to nearest integer', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: {
            ...mockConnectionStats,
            active: 333,
            concurrent_limit: 1000,
          },
        },
      });

      const { utilizationPercent, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      // 333 / 1000 * 100 = 33.3 -> 33
      expect(utilizationPercent.value).toBe(33);
    });
  });

  // ==========================================
  // Test Suite 4: Error Handling
  // ==========================================

  describe('Error Handling', () => {
    it('should capture API errors', async () => {
      mockApi.get.mockRejectedValueOnce(new Error('Network error'));

      const { error, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(error.value).toContain('Network error');
    });

    it('should set isLoading to false after error', async () => {
      mockApi.get.mockRejectedValueOnce(new Error('Test error'));

      const { isLoading, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(isLoading.value).toBe(false);
    });

    it('should handle non-Error throws', async () => {
      mockApi.get.mockRejectedValueOnce('String error');

      const { error, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      expect(error.value).toBe('Failed to fetch connections');
    });
  });

  // ==========================================
  // Test Suite 5: Retry Logic
  // ==========================================

  describe('Retry Logic with Exponential Backoff', () => {
    it('should retry after first failure', async () => {
      mockApi.get
        .mockRejectedValueOnce(new Error('First attempt fails'))
        .mockResolvedValueOnce({
          data: {
            stats: mockConnectionStats,
          },
        });

      const { fetchConnections, connections } = useDashboardConnections();

      await fetchConnections();

      // Advance timers to allow retry
      vi.advanceTimersByTime(1500);
      await vi.runAllTimersAsync();

      expect(mockApi.get).toHaveBeenCalledTimes(2);
    });

    it('should give up after max retries exceeded', async () => {
      mockApi.get.mockRejectedValue(new Error('Persistent failure'));

      const { error, fetchConnections } = useDashboardConnections();

      await fetchConnections();

      // Simulate 3 retries
      for (let i = 0; i < 3; i++) {
        vi.advanceTimersByTime(5000);
        await vi.runAllTimersAsync();
      }

      expect(error.value).toContain('max retries exceeded');
      expect(mockApi.get).toHaveBeenCalledTimes(4); // 1 initial + 3 retries
    });
  });

  // ==========================================
  // Test Suite 6: Auto-Fetch Polling
  // ==========================================

  describe('Auto-Fetch Polling', () => {
    it('should start polling on startAutoFetch', async () => {
      mockApi.get.mockResolvedValue({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { startAutoFetch } = useDashboardConnections();

      startAutoFetch(2000);

      // Advance time past interval
      vi.advanceTimersByTime(2500);
      await vi.runAllTimersAsync();

      expect(mockApi.get.mock.calls.length).toBeGreaterThanOrEqual(2);
    });

    it('should not allow duplicate auto-fetch', async () => {
      mockApi.get.mockResolvedValue({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { startAutoFetch } = useDashboardConnections();

      startAutoFetch(1000);
      startAutoFetch(1000); // Try to start again

      vi.advanceTimersByTime(1500);
      await vi.runAllTimersAsync();

      // Should only have expected calls, not doubled
      expect(mockApi.get.mock.calls.length).toBeLessThanOrEqual(3);
    });

    it('should use default 5 second interval', async () => {
      mockApi.get.mockResolvedValue({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { startAutoFetch } = useDashboardConnections();

      startAutoFetch(); // Use default interval

      // Advance less than default interval (5000ms)
      vi.advanceTimersByTime(3000);
      await vi.runAllTimersAsync();

      // Should only have initial call
      expect(mockApi.get).toHaveBeenCalledTimes(1);
    });
  });

  // ==========================================
  // Test Suite 7: Stop Auto-Fetch
  // ==========================================

  describe('Stop Auto-Fetch', () => {
    it('should stop polling when stopAutoFetch called', async () => {
      mockApi.get.mockResolvedValue({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { startAutoFetch, stopAutoFetch } = useDashboardConnections();

      startAutoFetch(1000);

      vi.advanceTimersByTime(500);
      stopAutoFetch();

      const callCountAfterStop = mockApi.get.mock.calls.length;

      vi.advanceTimersByTime(3000);
      await vi.runAllTimersAsync();

      expect(mockApi.get.mock.calls.length).toBe(callCountAfterStop);
    });

    it('should be safe to call multiple times', async () => {
      const { startAutoFetch, stopAutoFetch } = useDashboardConnections();

      mockApi.get.mockResolvedValue({
        data: {
          stats: mockConnectionStats,
        },
      });

      startAutoFetch(1000);

      expect(() => {
        stopAutoFetch();
        stopAutoFetch();
        stopAutoFetch();
      }).not.toThrow();
    });
  });

  // ==========================================
  // Test Suite 8: Request Deduplication
  // ==========================================

  describe('Request Deduplication', () => {
    it('should prevent concurrent overlapping requests', async () => {
      let resolveApi: any;
      mockApi.get.mockImplementation(
        () =>
          new Promise((resolve) => {
            resolveApi = resolve;
          })
      );

      const { fetchConnections } = useDashboardConnections();

      const promise1 = fetchConnections();
      const promise2 = fetchConnections(); // Second call while first is pending

      resolveApi({
        data: {
          stats: mockConnectionStats,
        },
      });

      await promise1;
      await promise2;

      expect(mockApi.get).toHaveBeenCalledTimes(1);
    });
  });

  // ==========================================
  // Test Suite 9: Computed Properties
  // ==========================================

  describe('Computed Properties', () => {
    it('should calculate timeSinceUpdate correctly', async () => {
      mockApi.get.mockResolvedValueOnce({
        data: {
          stats: mockConnectionStats,
        },
      });

      const { timeSinceUpdate, fetchConnections } = useDashboardConnections();

      expect(timeSinceUpdate.value).toBe(0);

      await fetchConnections();
      const timeAfterFetch = timeSinceUpdate.value;

      vi.advanceTimersByTime(1000);
      const timeAfter1Second = timeSinceUpdate.value;

      expect(timeAfterFetch).toBeLessThan(100);
      expect(timeAfter1Second).toBeGreaterThan(900);
      expect(timeAfter1Second).toBeLessThan(1100);
    });
  });

  // ==========================================
  // Test Suite 10: Error Clearing
  // ==========================================

  describe('Error Clearing', () => {
    it('should clear error with clearError method', async () => {
      mockApi.get.mockRejectedValueOnce(new Error('Test error'));

      const { error, fetchConnections, clearError } = useDashboardConnections();

      await fetchConnections();
      expect(error.value).toBeTruthy();

      clearError();

      expect(error.value).toBeNull();
    });
  });

  // ==========================================
  // Test Suite 11: Return Interface
  // ==========================================

  describe('Return Interface', () => {
    it('should return all expected properties', () => {
      const result = useDashboardConnections();

      expect(result).toHaveProperty('connections');
      expect(result).toHaveProperty('breakdown');
      expect(result).toHaveProperty('isLoading');
      expect(result).toHaveProperty('error');
      expect(result).toHaveProperty('lastUpdated');
      expect(result).toHaveProperty('timeSinceUpdate');
      expect(result).toHaveProperty('utilizationPercent');
      expect(result).toHaveProperty('fetchConnections');
      expect(result).toHaveProperty('startAutoFetch');
      expect(result).toHaveProperty('stopAutoFetch');
      expect(result).toHaveProperty('clearError');
    });
  });
});
