/**
 * useDashboardMetrics Composable
 *
 * Manages metrics polling from the API with intelligent caching and error handling.
 * - Fetches metrics from GET /api/dashboard/metrics every 1 second
 * - Implements exponential backoff retry strategy
 * - Provides loading, error, and last-updated state
 * - Auto-cleanup on component unmount
 *
 * @example
 * const { metrics, isLoading, error, startAutoFetch, stopAutoFetch } = useDashboardMetrics();
 * onMounted(() => startAutoFetch());
 * onBeforeUnmount(() => stopAutoFetch());
 */

import { ref, computed, onBeforeUnmount, type Ref, type ComputedRef } from "vue";
import { api } from "../useApi";
import type { SystemMetrics } from "@/types/dashboard";

// ==========================================
// Configuration Constants
// ==========================================

/** Polling interval in milliseconds - 1 second */
const POLL_INTERVAL_MS = 1000;

/** Maximum number of retry attempts before giving up */
const MAX_RETRIES = 3;

/** Initial retry delay in milliseconds - will be exponentially backed off */
const INITIAL_RETRY_DELAY = 1000;

/** Maximum backoff delay to prevent excessive waiting */
const MAX_BACKOFF_DELAY = 10000;

// ==========================================
// Return Type Interface
// ==========================================

/**
 * Return type for the useDashboardMetrics composable
 */
interface UseDashboardMetricsReturn {
    metrics: Ref<SystemMetrics | null>;
    isLoading: Ref<boolean>;
    error: Ref<string | null>;
    lastUpdated: Ref<number>;
    timeSinceUpdate: ComputedRef<number>;
    fetchMetrics: () => Promise<void>;
    startAutoFetch: (intervalMs?: number) => void;
    stopAutoFetch: () => void;
    clearError: () => void;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing dashboard metrics polling and state
 *
 * Handles:
 * - API calls to fetch system metrics from GET /api/dashboard/metrics
 * - Exponential backoff retry on failure with jitter
 * - Automatic polling at configurable interval (default 1 second)
 * - Error state management with clear error capability
 * - Automatic cleanup on component unmount
 *
 * @returns {UseDashboardMetricsReturn} Reactive metrics state and control functions
 */
export function useDashboardMetrics(): UseDashboardMetricsReturn {
    // ==========================================
    // State
    // ==========================================

    /** Current metrics data from API */
    const metrics = ref<SystemMetrics | null>(null);

    /** Loading state indicator for current request */
    const isLoading = ref(false);

    /** Error message, null if no error */
    const error = ref<string | null>(null);

    /** Timestamp of last successful update (milliseconds) */
    const lastUpdated = ref(0);

    // ==========================================
    // Internal Tracking
    // ==========================================

    /** Active polling interval ID */
    let pollInterval: ReturnType<typeof setInterval> | null = null;

    /** Current retry attempt count */
    let retryCount = 0;

    /** Pending retry timeout ID */
    let retryTimeout: ReturnType<typeof setTimeout> | null = null;

    // ==========================================
    // Computed Properties
    // ==========================================

    /**
     * Computed: Milliseconds elapsed since last successful update
     * Returns 0 if never updated
     */
    const timeSinceUpdate = computed((): number => {
        if (!lastUpdated.value) return 0;
        return Date.now() - lastUpdated.value;
    });

    // ==========================================
    // Helper Functions
    // ==========================================

    /**
     * Calculate exponential backoff delay with jitter
     * Formula: min(initialDelay * 2^retryCount, maxDelay) * jitter
     * Jitter prevents thundering herd problem
     *
     * @param attempt - Current retry attempt number (0-based)
     * @returns Delay in milliseconds
     */
    const calculateBackoffDelay = (attempt: number): number => {
        const exponentialDelay = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
        const cappedDelay = Math.min(exponentialDelay, MAX_BACKOFF_DELAY);
        // Add random jitter (0.5 to 1.0x multiplier)
        const jitter = cappedDelay * (0.5 + Math.random());
        return Math.floor(jitter);
    };

    // ==========================================
    // Core Functions
    // ==========================================

    /**
     * Fetch metrics from the API endpoint
     *
     * Implements:
     * - API call to GET /api/dashboard/metrics
     * - Error handling with exponential backoff retry
     * - State updates for loading, error, and metrics
     * - Prevents overlapping requests
     *
     * @throws Will not throw; errors are captured in error ref
     */
    const fetchMetrics = async (): Promise<void> => {
        // Prevent overlapping concurrent requests
        if (isLoading.value) {
            return;
        }

        isLoading.value = true;
        error.value = null;

        try {
            // Make API call to dashboard metrics endpoint
            const response = await api.get<SystemMetrics>("/api/dashboard/metrics");

            // Successfully retrieved metrics
            if (response.data) {
                metrics.value = response.data;
                lastUpdated.value = Date.now();
                retryCount = 0; // Reset retry counter on success

                if (import.meta.env.DEV) {
                    console.debug("[useDashboardMetrics] Metrics fetched successfully", {
                        requests_per_sec: response.data.requests_per_sec.toFixed(2),
                        active_connections: response.data.active_connections,
                        memory_usage_mb: response.data.memory_usage_mb.toFixed(1),
                    });
                }
            }
        } catch (err) {
            // Determine error message
            let errorMsg = "Failed to fetch metrics";

            if (err instanceof Error) {
                errorMsg = err.message;
            }

            error.value = errorMsg;

            console.warn("[useDashboardMetrics] Fetch failed", {
                error: errorMsg,
                attempt: retryCount + 1,
                maxRetries: MAX_RETRIES,
            });

            // Implement exponential backoff retry
            if (retryCount < MAX_RETRIES) {
                const delayMs = calculateBackoffDelay(retryCount);
                retryCount++;

                if (import.meta.env.DEV) {
                    console.debug(
                        `[useDashboardMetrics] Retrying in ${delayMs}ms (attempt ${retryCount}/${MAX_RETRIES})`,
                    );
                }

                retryTimeout = setTimeout(fetchMetrics, delayMs);
            } else {
                console.error(
                    "[useDashboardMetrics] Max retries exceeded, stopping retry attempts",
                );
                error.value = `${errorMsg} (max retries exceeded)`;
            }
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * Start automatic metrics polling
     *
     * Behavior:
     * - Fetches metrics immediately
     * - Sets up recurring polling at specified interval
     * - Does nothing if already polling
     * - Clears any pending retries when starting
     *
     * @param {number} [intervalMs=POLL_INTERVAL_MS] - Polling interval in milliseconds
     */
    const startAutoFetch = (intervalMs = POLL_INTERVAL_MS): void => {
        // Prevent duplicate polling
        if (pollInterval !== null) {
            console.warn(
                "[useDashboardMetrics] Auto-fetch already running, ignoring start request",
            );
            return;
        }

        if (import.meta.env.DEV) {
            console.debug(
                `[useDashboardMetrics] Starting auto-fetch with ${intervalMs}ms interval`,
            );
        }

        // Clear any pending retries
        if (retryTimeout !== null) {
            clearTimeout(retryTimeout);
            retryTimeout = null;
            retryCount = 0;
        }

        // Fetch immediately
        fetchMetrics();

        // Then set up recurring interval
        pollInterval = setInterval(fetchMetrics, intervalMs);
    };

    /**
     * Stop automatic metrics polling
     *
     * Behavior:
     * - Clears the polling interval
     * - Cancels any pending retries
     * - Resets retry counter
     * - Safe to call multiple times
     */
    const stopAutoFetch = (): void => {
        if (pollInterval !== null) {
            clearInterval(pollInterval);
            pollInterval = null;

            if (import.meta.env.DEV) {
                console.debug("[useDashboardMetrics] Auto-fetch stopped");
            }
        }

        if (retryTimeout !== null) {
            clearTimeout(retryTimeout);
            retryTimeout = null;
        }

        retryCount = 0;
    };

    /**
     * Clear the current error state
     *
     * Useful for dismissing error messages in the UI
     */
    const clearError = (): void => {
        error.value = null;
    };

    // ==========================================
    // Lifecycle Management
    // ==========================================

    /**
     * Cleanup on component unmount
     * Ensures all timers and intervals are cleared to prevent memory leaks
     */
    onBeforeUnmount(() => {
        stopAutoFetch();
    });

    // ==========================================
    // Return Interface
    // ==========================================

    return {
        // State
        metrics,
        isLoading,
        error,
        lastUpdated,
        timeSinceUpdate,

        // Methods
        fetchMetrics,
        startAutoFetch,
        stopAutoFetch,
        clearError,
    };
}
