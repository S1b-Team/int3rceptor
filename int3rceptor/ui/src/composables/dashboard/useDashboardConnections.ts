/**
 * useDashboardConnections Composable
 *
 * Manages connection statistics fetching from the API with intelligent error handling.
 * - Fetches connection stats from GET /api/dashboard/connections
 * - Implements exponential backoff retry strategy (same as metrics)
 * - Provides loading, error, and connection state
 * - Auto-cleanup on component unmount
 *
 * @example
 * const { connections, isLoading, error, startAutoFetch, stopAutoFetch } = useDashboardConnections();
 * onMounted(() => startAutoFetch());
 * onBeforeUnmount(() => stopAutoFetch());
 */

import { ref, computed, onBeforeUnmount, type Ref, type ComputedRef } from "vue";
import { api } from "../useApi";
import type { ConnectionStats, ConnectionBreakdown } from "@/types/dashboard";

// ==========================================
// Configuration Constants
// ==========================================

/** Polling interval in milliseconds - 5 seconds (same as activity) */
const POLL_INTERVAL_MS = 5000;

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
 * Return type for the useDashboardConnections composable
 */
interface UseDashboardConnectionsReturn {
    connections: Ref<ConnectionStats | null>;
    breakdown: Ref<ConnectionBreakdown | null>;
    isLoading: Ref<boolean>;
    error: Ref<string | null>;
    lastUpdated: Ref<number>;
    timeSinceUpdate: ComputedRef<number>;
    utilizationPercent: ComputedRef<number>;
    fetchConnections: () => Promise<void>;
    startAutoFetch: (intervalMs?: number) => void;
    stopAutoFetch: () => void;
    clearError: () => void;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing dashboard connection statistics polling and state
 *
 * Handles:
 * - API calls to fetch connection stats from GET /api/dashboard/connections
 * - Exponential backoff retry on failure with jitter
 * - Automatic polling at configurable interval (default 5 seconds)
 * - Error state management with clear error capability
 * - Automatic cleanup on component unmount
 * - Calculates connection pool utilization percentage
 *
 * @returns {UseDashboardConnectionsReturn} Reactive connection state and control functions
 */
export function useDashboardConnections(): UseDashboardConnectionsReturn {
    // ==========================================
    // State
    // ==========================================

    /** Current connection statistics from API */
    const connections = ref<ConnectionStats | null>(null);

    /** Connection breakdown by protocol and state */
    const breakdown = ref<ConnectionBreakdown | null>(null);

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

    /**
     * Computed: Connection pool utilization percentage
     * Calculated as (active / concurrent_limit) * 100
     * Returns 0 if no data
     */
    const utilizationPercent = computed((): number => {
        if (!connections.value) return 0;

        const { active, concurrent_limit } = connections.value;

        if (concurrent_limit === 0) return 0;

        return Math.round((active / concurrent_limit) * 100);
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
     * Fetch connection statistics from the API endpoint
     *
     * Implements:
     * - API call to GET /api/dashboard/connections
     * - Error handling with exponential backoff retry
     * - State updates for loading, error, and connections
     * - Prevents overlapping requests
     *
     * @throws Will not throw; errors are captured in error ref
     */
    const fetchConnections = async (): Promise<void> => {
        // Prevent overlapping concurrent requests
        if (isLoading.value) {
            return;
        }

        isLoading.value = true;
        error.value = null;

        try {
            // Make API call to dashboard connections endpoint
            const response = await api.get<{
                stats: ConnectionStats;
                breakdown?: ConnectionBreakdown;
            }>("/api/dashboard/connections");

            // Successfully retrieved connection data
            if (response.data) {
                connections.value = response.data.stats;
                breakdown.value = response.data.breakdown || null;
                lastUpdated.value = Date.now();
                retryCount = 0; // Reset retry counter on success

                if (import.meta.env.DEV) {
                    console.debug("[useDashboardConnections] Connections fetched successfully", {
                        active: response.data.stats.active,
                        established: response.data.stats.established,
                        utilization: utilizationPercent.value + "%",
                    });
                }
            }
        } catch (err) {
            // Determine error message
            let errorMsg = "Failed to fetch connections";

            if (err instanceof Error) {
                errorMsg = err.message;
            }

            error.value = errorMsg;

            console.warn("[useDashboardConnections] Fetch failed", {
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
                        `[useDashboardConnections] Retrying in ${delayMs}ms (attempt ${retryCount}/${MAX_RETRIES})`,
                    );
                }

                retryTimeout = setTimeout(fetchConnections, delayMs);
            } else {
                console.error(
                    "[useDashboardConnections] Max retries exceeded, stopping retry attempts",
                );
                error.value = `${errorMsg} (max retries exceeded)`;
            }
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * Start automatic connection statistics polling
     *
     * Behavior:
     * - Fetches connection stats immediately
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
                "[useDashboardConnections] Auto-fetch already running, ignoring start request",
            );
            return;
        }

        if (import.meta.env.DEV) {
            console.debug(
                `[useDashboardConnections] Starting auto-fetch with ${intervalMs}ms interval`,
            );
        }

        // Clear any pending retries
        if (retryTimeout !== null) {
            clearTimeout(retryTimeout);
            retryTimeout = null;
            retryCount = 0;
        }

        // Fetch immediately
        fetchConnections();

        // Then set up recurring interval
        pollInterval = setInterval(fetchConnections, intervalMs);
    };

    /**
     * Stop automatic connection statistics polling
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
                console.debug("[useDashboardConnections] Auto-fetch stopped");
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
        connections,
        breakdown,
        isLoading,
        error,
        lastUpdated,
        timeSinceUpdate,
        utilizationPercent,

        // Methods
        fetchConnections,
        startAutoFetch,
        stopAutoFetch,
        clearError,
    };
}
