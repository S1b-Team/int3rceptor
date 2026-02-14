import { vi, beforeEach, afterEach } from 'vitest';

/**
 * Global test setup
 * Configures mocks and utilities for all tests
 */

// ==========================================
// Mock localStorage
// ==========================================

const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

// ==========================================
// Mock window.location
// ==========================================

delete (window as any).location;
window.location = {
  protocol: 'http:',
  host: 'localhost:5173',
  href: 'http://localhost:5173',
  pathname: '/',
  search: '',
  hash: '',
  reload: vi.fn(),
} as any;

// ==========================================
// Mock WebSocket
// ==========================================

export class MockWebSocket {
  url: string;
  readyState = WebSocket.CONNECTING;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: Event) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    // Simulate connection after a tick
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 0);
  }

  send(data: string) {
    // Mock send implementation
  }

  close() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.(new Event('close'));
  }

  simulateMessage(data: string) {
    this.onmessage?.(new MessageEvent('message', { data }));
  }

  simulateError() {
    this.onerror?.(new Event('error'));
  }
}

(window as any).WebSocket = MockWebSocket;

// ==========================================
// Global test hooks
// ==========================================

beforeEach(() => {
  // Clear all timers before each test
  vi.clearAllTimers();
  // Clear all mocks
  vi.clearAllMocks();
});

afterEach(() => {
  // Cleanup after each test
  vi.clearAllTimers();
  localStorageMock.clear();
});

// ==========================================
// Test utilities
// ==========================================

export const sleep = (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms));

export const flushPromises = () =>
  new Promise((resolve) => setTimeout(resolve, 0));

export const advanceTimersByTime = (ms: number) => {
  vi.advanceTimersByTime(ms);
};

export const runAllTimers = () => {
  vi.runAllTimers();
};

// Setup test environment
vi.useFakeTimers();
