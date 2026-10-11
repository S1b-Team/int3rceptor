import { useIntruder, INTRUDER_POLL_INTERVAL_MS } from '@/composables/useIntruder';
import type { IntruderResult, IntruderStatus } from '@/types';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * Test suite for the useIntruder composable.
 */

vi.mock('@/composables/useApi', () => ({
  api: {
    post: vi.fn(),
    delete: vi.fn(),
    get: vi.fn(),
  },
}));

import { api } from '@/composables/useApi';

const sampleResults: IntruderResult[] = [
  {
    request_id: 0,
    payload: 'admin',
    status_code: 200,
    response_length: 1234,
    duration_ms: 42,
  },
  {
    request_id: 1,
    payload: 'root',
    status_code: 403,
    response_length: 99,
    duration_ms: 51,
  },
];

const runningStatus: IntruderStatus = { running: true, result_count: 1 };
const doneStatus: IntruderStatus = { running: false, result_count: 2 };

describe('useIntruder', () => {
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
    vi.useRealTimers();
  });

  describe('loadPayloads', () => {
    it('returns the payload list from the API', async () => {
      mockGet.mockResolvedValue({ data: { payloads: ['admin', 'root'] } });

      const { loadPayloads } = useIntruder();
      const payloads = await loadPayloads();

      expect(mockGet).toHaveBeenCalledWith('/api/intruder/payloads');
      expect(payloads).toEqual(['admin', 'root']);
    });

    it('returns an empty array on error', async () => {
      mockGet.mockRejectedValue(new Error('boom'));

      const { loadPayloads, lastError } = useIntruder();
      const payloads = await loadPayloads();

      expect(payloads).toEqual([]);
      expect(lastError.value).toBeTruthy();
    });
  });

  describe('getStatus', () => {
    it('syncs running and resultCount from the status endpoint', async () => {
      mockGet.mockResolvedValue({ data: runningStatus });

      const { getStatus, running, resultCount } = useIntruder();
      const status = await getStatus();

      expect(mockGet).toHaveBeenCalledWith('/api/intruder/status');
      expect(status).toEqual(runningStatus);
      expect(running.value).toBe(true);
      expect(resultCount.value).toBe(1);
    });
  });

  describe('refreshResults', () => {
    it('stores the fetched result rows', async () => {
      mockGet.mockResolvedValue({ data: sampleResults });

      const { refreshResults, results } = useIntruder();
      const rows = await refreshResults();

      expect(mockGet).toHaveBeenCalledWith('/api/intruder/results');
      expect(rows).toEqual(sampleResults);
      expect(results.value).toEqual(sampleResults);
    });
  });

  describe('generatePreview', () => {
    it('posts the request and returns generated request strings', async () => {
      mockPost.mockResolvedValue({ data: { requests: ['req-1', 'req-2'] } });

      const { generatePreview } = useIntruder();
      const preview = await generatePreview({
        template: 'GET /§x§ HTTP/1.1',
        config: {
          positions: [{ start: 0, end: 0, name: 'x' }],
          payloads: ['a', 'b'],
          attack_type: 'Sniper',
          options: { concurrency: 1, delay_ms: 0 },
        },
      });

      expect(mockPost).toHaveBeenCalledWith(
        '/api/intruder/generate',
        expect.objectContaining({ template: 'GET /§x§ HTTP/1.1' }),
      );
      expect(preview).toEqual(['req-1', 'req-2']);
    });
  });

  describe('startAttack', () => {
    it('posts start, resets results, and marks running', async () => {
      vi.useFakeTimers();
      mockPost.mockResolvedValue({});
      mockGet.mockResolvedValue({ data: runningStatus });

      const { startAttack, running, results } = useIntruder();
      const ok = await startAttack({
        template: 'GET /§x§ HTTP/1.1',
        config: {
          positions: [{ start: 0, end: 0, name: 'x' }],
          payloads: ['a'],
          attack_type: 'Sniper',
          options: { concurrency: 1, delay_ms: 0 },
        },
      });

      expect(ok).toBe(true);
      expect(mockPost).toHaveBeenCalledWith(
        '/api/intruder/start',
        expect.any(Object),
      );
      expect(running.value).toBe(true);
      expect(results.value).toEqual([]);
    });

    it('does not start a second attack while one is operating', async () => {
      let resolvePost: (value: any) => void = () => {};
      mockPost.mockImplementation(
        () => new Promise(resolve => {
          resolvePost = resolve;
        }),
      );

      const { startAttack } = useIntruder();
      const req = {
        template: 't',
        config: {
          positions: [{ start: 0, end: 0, name: 'x' }],
          payloads: ['a'],
          attack_type: 'Sniper' as const,
          options: { concurrency: 1, delay_ms: 0 },
        },
      };

      const p1 = startAttack(req);
      const p2 = startAttack(req);

      expect(mockPost).toHaveBeenCalledTimes(1);

      resolvePost({});
      await p1;
      await p2;
    });

    it('reports failure and leaves running false on error', async () => {
      mockPost.mockRejectedValue(new Error('nope'));

      const { startAttack, running, lastError } = useIntruder();
      const ok = await startAttack({
        template: 't',
        config: {
          positions: [{ start: 0, end: 0, name: 'x' }],
          payloads: ['a'],
          attack_type: 'Sniper',
          options: { concurrency: 1, delay_ms: 0 },
        },
      });

      expect(ok).toBe(false);
      expect(running.value).toBe(false);
      expect(lastError.value).toBeTruthy();
    });
  });

  describe('polling', () => {
    it('stops polling once the backend reports the attack is done', async () => {
      vi.useFakeTimers();
      mockPost.mockResolvedValue({});
      // start -> status running, then status done.
      mockGet
        .mockResolvedValueOnce({ data: runningStatus }) // tick 1 status
        .mockResolvedValueOnce({ data: sampleResults }) // tick 1 results
        .mockResolvedValueOnce({ data: doneStatus }) // tick 2 status
        .mockResolvedValueOnce({ data: sampleResults }); // tick 2 results

      const { startAttack, running } = useIntruder();
      await startAttack({
        template: 't',
        config: {
          positions: [{ start: 0, end: 0, name: 'x' }],
          payloads: ['a'],
          attack_type: 'Sniper',
          options: { concurrency: 1, delay_ms: 0 },
        },
      });

      // Tick once: still running.
      await vi.advanceTimersByTimeAsync(INTRUDER_POLL_INTERVAL_MS);
      expect(running.value).toBe(true);

      // Tick again: backend reports done -> polling stops.
      await vi.advanceTimersByTimeAsync(INTRUDER_POLL_INTERVAL_MS);
      expect(running.value).toBe(false);

      const callsAfterStop = mockGet.mock.calls.length;
      await vi.advanceTimersByTimeAsync(INTRUDER_POLL_INTERVAL_MS * 3);
      // No further polling after the loop stopped.
      expect(mockGet.mock.calls.length).toBe(callsAfterStop);
    });
  });

  describe('stopAttack', () => {
    it('posts stop and clears running when the backend has drained', async () => {
      mockPost.mockResolvedValue({});
      mockGet
        .mockResolvedValueOnce({ data: doneStatus }) // getStatus after stop
        .mockResolvedValueOnce({ data: sampleResults }); // final refreshResults

      const { stopAttack, running } = useIntruder();
      await stopAttack();

      expect(mockPost).toHaveBeenCalledWith('/api/intruder/stop');
      expect(running.value).toBe(false);
    });
  });

  describe('clearResults', () => {
    it('deletes results and resets local state', async () => {
      mockDelete.mockResolvedValue({});

      const { clearResults, results, resultCount } = useIntruder();
      results.value = [...sampleResults];
      resultCount.value = 2;

      await clearResults();

      expect(mockDelete).toHaveBeenCalledWith('/api/intruder/results');
      expect(results.value).toEqual([]);
      expect(resultCount.value).toBe(0);
    });
  });

  describe('notifications', () => {
    it('notifies on a successful payload load', async () => {
      const notify = vi.fn();
      mockGet.mockResolvedValue({ data: { payloads: ['admin'] } });

      const { loadPayloads } = useIntruder(notify);
      await loadPayloads();

      expect(notify).toHaveBeenCalledWith(
        expect.stringContaining('payload'),
        'success',
      );
    });
  });
});
