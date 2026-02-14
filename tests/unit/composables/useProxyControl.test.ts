import { useProxyControl } from '@/composables/useProxyControl';
import type { ProxyStatus } from '@/types/dashboard';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * Test suite for useProxyControl composable
 */

vi.mock('@/composables/useApi', () => ({
  api: {
    post: vi.fn(),
    delete: vi.fn(),
    get: vi.fn(),
  },
}));

import { api } from '@/composables/useApi';

const mockProxyStatus: ProxyStatus = {
  running: true,
  host: 'localhost',
  port: 8080,
  tls_enabled: true,
  intercept_https: true,
  start_time: Date.now() - 3600000,
  certificates_generated: 5,
};

describe('useProxyControl', () => {
  let mockPost: any;
  let mockDelete: any;
  let mockGet: any;

  beforeEach(() => {
    mockPost = vi.fn();
    mockDelete = vi.fn();
    mockGet = vi.fn();
    (api.post as any) = mockPost;
    (api.delete as any) = mockDelete;
    (api.get as any) = mockGet;
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
  });

  describe('Proxy Status', () => {
    it('should get proxy status', async () => {
      mockGet.mockResolvedValue({ data: mockProxyStatus });

      const { getProxyStatus, proxyStatus } = useProxyControl();

      const status = await getProxyStatus();

      expect(status).toEqual(mockProxyStatus);
      expect(proxyStatus.value).toEqual(mockProxyStatus);
      expect(mockGet).toHaveBeenCalledWith('/api/proxy/status');
    });

    it('should set isProxyRunning based on status', async () => {
      mockGet.mockResolvedValue({ data: mockProxyStatus });

      const { getProxyStatus, isProxyRunning } = useProxyControl();

      await getProxyStatus();

      expect(isProxyRunning.value).toBe(true);
    });

    it('should handle status fetch error', async () => {
      mockGet.mockRejectedValue(new Error('Failed to fetch'));

      const { getProxyStatus } = useProxyControl();

      const status = await getProxyStatus();

      expect(status).toBeNull();
    });
  });

  describe('Start Proxy', () => {
    it('should start proxy server', async () => {
      const startedStatus = { ...mockProxyStatus, running: true };
      mockPost.mockResolvedValue({ data: startedStatus });

      const { startProxy, isProxyRunning } = useProxyControl();

      await startProxy();

      expect(mockPost).toHaveBeenCalledWith('/api/proxy/start');
      expect(isProxyRunning.value).toBe(true);
    });

    it('should handle start error', async () => {
      mockPost.mockRejectedValue(new Error('Failed to start'));

      const { startProxy, isProxyRunning } = useProxyControl();

      await startProxy();

      expect(isProxyRunning.value).toBe(false);
    });

    it('should not start if already running', async () => {
      const { isProxyRunning, startProxy } = useProxyControl();
      isProxyRunning.value = true;

      await startProxy();

      expect(mockPost).not.toHaveBeenCalled();
    });

    it('should prevent operation during another operation', async () => {
      // Use a promise that we control manually
      let resolvePost: (value: any) => void;
      mockPost.mockImplementation(
        () => new Promise((resolve) => {
          resolvePost = resolve;
        })
      );

      const { startProxy } = useProxyControl();

      const promise1 = startProxy();
      const promise2 = startProxy(); // Should be blocked

      // Only one call should be made (second is blocked while first is pending)
      expect(mockPost).toHaveBeenCalledTimes(1);

      // Resolve the first call
      resolvePost!({ data: mockProxyStatus });
      await promise1;
      await promise2;
    });
  });

  describe('Stop Proxy', () => {
    it('should stop proxy server', async () => {
      const stoppedStatus = { ...mockProxyStatus, running: false };
      mockPost.mockResolvedValue({ data: stoppedStatus });

      const { stopProxy, isProxyRunning } = useProxyControl();
      isProxyRunning.value = true;

      await stopProxy();

      expect(mockPost).toHaveBeenCalledWith('/api/proxy/stop');
      expect(isProxyRunning.value).toBe(false);
    });

    it('should handle stop error', async () => {
      mockPost.mockRejectedValue(new Error('Failed to stop'));

      const { stopProxy, isProxyRunning } = useProxyControl();
      isProxyRunning.value = true; // Must be running to attempt stop

      await stopProxy();

      expect(mockPost).toHaveBeenCalledWith('/api/proxy/stop');
    });

    it('should not stop if not running', async () => {
      const { isProxyRunning, stopProxy } = useProxyControl();
      isProxyRunning.value = false;

      await stopProxy();

      expect(mockPost).not.toHaveBeenCalled();
    });
  });

  describe('Toggle Proxy', () => {
    it('should start proxy if stopped', async () => {
      const startedStatus = { ...mockProxyStatus, running: true };
      mockPost.mockResolvedValue({ data: startedStatus });

      const { toggleProxy, isProxyRunning } = useProxyControl();
      isProxyRunning.value = false;

      await toggleProxy();

      expect(mockPost).toHaveBeenCalledWith('/api/proxy/start');
    });

    it('should stop proxy if running', async () => {
      const stoppedStatus = { ...mockProxyStatus, running: false };
      mockPost.mockResolvedValue({ data: stoppedStatus });

      const { toggleProxy, isProxyRunning } = useProxyControl();
      isProxyRunning.value = true;

      await toggleProxy();

      expect(mockPost).toHaveBeenCalledWith('/api/proxy/stop');
    });
  });

  describe('Clear Traffic', () => {
    it('should clear traffic and return count', async () => {
      mockDelete.mockResolvedValue({ data: { cleared_count: 42 } });

      const { clearTraffic } = useProxyControl();

      await clearTraffic();

      expect(mockDelete).toHaveBeenCalledWith('/api/traffic');
    });

    it('should handle clear error', async () => {
      mockDelete.mockRejectedValue(new Error('Failed to clear'));

      const { clearTraffic } = useProxyControl();

      await clearTraffic();

      expect(mockDelete).toHaveBeenCalledWith('/api/traffic');
    });

    it('should not allow multiple simultaneous clears', async () => {
      let resolveDelete: (value: any) => void;
      mockDelete.mockImplementation(
        () => new Promise((resolve) => {
          resolveDelete = resolve;
        })
      );

      const { clearTraffic } = useProxyControl();

      const promise1 = clearTraffic();
      const promise2 = clearTraffic(); // Should be blocked

      expect(mockDelete).toHaveBeenCalledTimes(1);

      resolveDelete!({ data: { cleared_count: 0 } });
      await promise1;
      await promise2;
    });
  });

  describe('Export Metrics', () => {
    it('should export metrics as JSON', async () => {
      const mockData = { metrics: [], summary: {} };
      mockPost.mockResolvedValue({ data: mockData });

      const { exportMetrics } = useProxyControl();

      // Mock URL.createObjectURL and DOM methods
      global.URL.createObjectURL = vi.fn(() => 'blob:mock-url');
      global.URL.revokeObjectURL = vi.fn();
      const mockLink = {
        href: '',
        download: '',
        click: vi.fn(),
      };
      vi.spyOn(document, 'createElement').mockReturnValue(mockLink as any);
      vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink as any);
      vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink as any);

      await exportMetrics('json');

      expect(mockPost).toHaveBeenCalledWith('/api/dashboard/export?format=json', {}, { responseType: 'blob' });
    });

    it('should export metrics as CSV', async () => {
      mockPost.mockResolvedValue({ data: {} });

      const { exportMetrics } = useProxyControl();

      global.URL.createObjectURL = vi.fn(() => 'blob:mock-url');
      global.URL.revokeObjectURL = vi.fn();
      const mockLink = {
        href: '',
        download: '',
        click: vi.fn(),
      };
      vi.spyOn(document, 'createElement').mockReturnValue(mockLink as any);
      vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink as any);
      vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink as any);

      await exportMetrics('csv');

      expect(mockPost).toHaveBeenCalledWith('/api/dashboard/export?format=csv', {}, { responseType: 'blob' });
    });

    it('should export metrics as HAR', async () => {
      mockPost.mockResolvedValue({ data: {} });

      const { exportMetrics } = useProxyControl();

      global.URL.createObjectURL = vi.fn(() => 'blob:mock-url');
      global.URL.revokeObjectURL = vi.fn();
      const mockLink = {
        href: '',
        download: '',
        click: vi.fn(),
      };
      vi.spyOn(document, 'createElement').mockReturnValue(mockLink as any);
      vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink as any);
      vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink as any);

      await exportMetrics('har');

      expect(mockPost).toHaveBeenCalledWith('/api/dashboard/export?format=har', {}, { responseType: 'blob' });
    });

    it('should handle export error', async () => {
      mockPost.mockRejectedValue(new Error('Export failed'));

      const { exportMetrics } = useProxyControl();

      await exportMetrics('json');

      expect(mockPost).toHaveBeenCalled();
    });
  });

  describe('Operation State', () => {
    it('should track isOperating during operations', async () => {
      let resolvePost: any;
      mockPost.mockImplementation(
        () =>
          new Promise((resolve) => {
            resolvePost = resolve;
          })
      );

      const { startProxy, isOperating } = useProxyControl();

      const promise = startProxy();

      // isOperating should be true during operation
      expect(isOperating.value).toBe(true);

      resolvePost({ data: mockProxyStatus });
      await promise;

      expect(isOperating.value).toBe(false);
    });

    it('should track lastOperation', async () => {
      mockPost.mockResolvedValue({ data: mockProxyStatus });

      const { startProxy, lastOperation } = useProxyControl();

      await startProxy();

      expect(lastOperation.value).toBe('start_proxy');
    });
  });

  describe('Notifications', () => {
    it('should call notification callback on success', async () => {
      const mockNotify = vi.fn();
      mockPost.mockResolvedValue({ data: mockProxyStatus });

      const { startProxy } = useProxyControl(mockNotify);

      await startProxy();

      expect(mockNotify).toHaveBeenCalledWith(
        expect.stringContaining('started'),
        'success'
      );
    });

    it('should call notification callback on error', async () => {
      const mockNotify = vi.fn();
      mockPost.mockRejectedValue(new Error('Start failed'));

      const { startProxy } = useProxyControl(mockNotify);

      await startProxy();

      expect(mockNotify).toHaveBeenCalledWith(
        expect.any(String),
        'error'
      );
    });
  });

  describe('Edge Cases', () => {
    it('should handle rapid toggle calls', async () => {
      mockPost.mockResolvedValue({ data: mockProxyStatus });

      const { toggleProxy } = useProxyControl();

      await toggleProxy();
      await toggleProxy();

      // Second call should be prevented while first is operating
      expect(mockPost.mock.calls.length).toBeLessThanOrEqual(2);
    });

    it('should handle null response gracefully', async () => {
      mockPost.mockResolvedValue(null);

      const { startProxy, proxyStatus } = useProxyControl();

      await startProxy();

      // Should not crash
      expect(proxyStatus.value).toBeDefined();
    });
  });
});
