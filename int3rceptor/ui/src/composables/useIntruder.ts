/**
 * useIntruder Composable
 *
 * Owns the client-side orchestration of an Intruder attack against the API:
 * - GET /api/intruder/payloads - Load the built-in starter payload set
 * - POST /api/intruder/generate - Preview the requests that will be sent
 * - POST /api/intruder/start - Launch an attack
 * - POST /api/intruder/stop - Stop an in-flight attack
 * - GET /api/intruder/status - Poll { running, result_count }
 * - GET /api/intruder/results - Fetch the result rows
 * - DELETE /api/intruder/results - Clear stored results
 *
 * The composable keeps reactive state (running, results, resultCount) and drives
 * a status-based polling loop so the UI stops polling once the backend reports
 * the attack has drained.
 *
 * @example
 * const { startAttack, stopAttack, running, results } = useIntruder();
 * await startAttack({ template, config });
 */

import { ref, type Ref } from "vue";
import { api } from "./useApi";
import type {
    IntruderGenerateRequest,
    IntruderPayloadsResponse,
    IntruderResult,
    IntruderStatus,
} from "@/types";

// ==========================================
// Configuration Constants
// ==========================================

/** How often the status/results polling loop ticks while an attack runs. */
export const INTRUDER_POLL_INTERVAL_MS = 500;

// ==========================================
// Type Definitions
// ==========================================

/**
 * Notification callback function type
 */
export type NotificationCallback = (
    message: string,
    type: "success" | "error" | "warning" | "info",
) => void;

/**
 * Return type for the useIntruder composable
 */
interface UseIntruderReturn {
    running: Ref<boolean>;
    isOperating: Ref<boolean>;
    results: Ref<IntruderResult[]>;
    resultCount: Ref<number>;
    lastError: Ref<string | null>;
    loadPayloads: () => Promise<string[]>;
    getStatus: () => Promise<IntruderStatus | null>;
    refreshResults: () => Promise<IntruderResult[]>;
    generatePreview: (request: IntruderGenerateRequest) => Promise<string[]>;
    startAttack: (request: IntruderGenerateRequest) => Promise<boolean>;
    stopAttack: () => Promise<void>;
    clearResults: () => Promise<void>;
    stopPolling: () => void;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing Intruder attacks.
 *
 * @param {NotificationCallback} [onNotify] - Optional callback for notifications
 * @returns {UseIntruderReturn} Intruder state and methods
 */
export function useIntruder(onNotify?: NotificationCallback): UseIntruderReturn {
    // ==========================================
    // State
    // ==========================================

    /** Whether an attack is currently sending (mirrors the backend status). */
    const running = ref(false);

    /** Whether a start/stop/clear operation is in progress. */
    const isOperating = ref(false);

    /** Latest result rows fetched from the API. */
    const results = ref<IntruderResult[]>([]);

    /** Result count reported by the status endpoint. */
    const resultCount = ref(0);

    /** Last error message, if any. */
    const lastError = ref<string | null>(null);

    /** Handle for the active polling timer. */
    let pollTimer: ReturnType<typeof setInterval> | null = null;

    /**
     * Identity for the current polling run. Bumped whenever polling starts or
     * stops so in-flight callbacks from an older run are ignored and cannot
     * clear a newer attack's timer.
     */
    let pollGeneration = 0;

    // ==========================================
    // Helper Functions
    // ==========================================

    const notify = (
        message: string,
        type: "success" | "error" | "warning" | "info" = "info",
    ): void => {
        if (onNotify) {
            onNotify(message, type);
        }
    };

    const handleError = (operation: string, error: unknown): string => {
        let message = `${operation} failed`;
        if (error instanceof Error) {
            message = error.message;
        } else if (typeof error === "string") {
            message = error;
        }
        lastError.value = message;
        console.error(`[useIntruder] ${operation} error:`, error);
        notify(message, "error");
        return message;
    };

    // ==========================================
    // Core Functions
    // ==========================================

    /**
     * Load the built-in starter payloads from the API.
     *
     * @returns {Promise<string[]>} Payload strings (empty array on failure)
     */
    const loadPayloads = async (): Promise<string[]> => {
        try {
            const { data } = await api.get<IntruderPayloadsResponse>(
                "/api/intruder/payloads",
            );
            const payloads = data?.payloads ?? [];
            notify(`Loaded ${payloads.length} common payloads`, "success");
            return payloads;
        } catch (err) {
            handleError("Load payloads", err);
            return [];
        }
    };

    /**
     * Fetch the current attack status and sync reactive state.
     *
     * @returns {Promise<IntruderStatus | null>} Status snapshot or null on error
     */
    const getStatus = async (): Promise<IntruderStatus | null> => {
        try {
            const { data } = await api.get<IntruderStatus>(
                "/api/intruder/status",
            );
            if (data) {
                running.value = data.running;
                resultCount.value = data.result_count;
            }
            return data;
        } catch (err) {
            handleError("Get status", err);
            return null;
        }
    };

    /**
     * Fetch the latest result rows.
     *
     * @returns {Promise<IntruderResult[]>} Result rows (empty array on failure)
     */
    const refreshResults = async (): Promise<IntruderResult[]> => {
        try {
            const { data } = await api.get<IntruderResult[]>(
                "/api/intruder/results",
            );
            results.value = data ?? [];
            return results.value;
        } catch (err) {
            handleError("Fetch results", err);
            return results.value;
        }
    };

    /**
     * Preview the requests an attack would generate without sending them.
     *
     * @param {IntruderGenerateRequest} request - Template + config
     * @returns {Promise<string[]>} Generated request strings (empty on failure)
     */
    const generatePreview = async (
        request: IntruderGenerateRequest,
    ): Promise<string[]> => {
        try {
            const { data } = await api.post<{ requests: string[] }>(
                "/api/intruder/generate",
                request,
            );
            return data?.requests ?? [];
        } catch (err) {
            handleError("Generate preview", err);
            return [];
        }
    };

    /**
     * Start an attack and begin polling status/results.
     *
     * @param {IntruderGenerateRequest} request - Template + config
     * @returns {Promise<boolean>} True if the attack started
     */
    const startAttack = async (
        request: IntruderGenerateRequest,
    ): Promise<boolean> => {
        if (isOperating.value || running.value) {
            notify(
                running.value
                    ? "An attack is already running"
                    : "An Intruder operation is already in progress",
                "warning",
            );
            return false;
        }

        isOperating.value = true;
        lastError.value = null;

        try {
            await api.post("/api/intruder/start", request);
            running.value = true;
            results.value = [];
            resultCount.value = 0;
            notify("Attack started", "success");
            startPolling();
            return true;
        } catch (err) {
            running.value = false;
            handleError("Start attack", err);
            return false;
        } finally {
            isOperating.value = false;
        }
    };

    /**
     * Stop an in-flight attack. Polling keeps running until the backend reports
     * the attack has drained, at which point the loop clears `running`.
     */
    const stopAttack = async (): Promise<void> => {
        try {
            await api.post("/api/intruder/stop");
            notify("Stop requested", "info");
            // Refresh immediately; the polling loop will clear `running` once the
            // backend finishes draining in-flight probes.
            await getStatus();
            if (!running.value) {
                stopPolling();
                await refreshResults();
            }
        } catch (err) {
            handleError("Stop attack", err);
        }
    };

    /**
     * Clear stored results on the backend and locally.
     */
    const clearResults = async (): Promise<void> => {
        try {
            await api.delete("/api/intruder/results");
            results.value = [];
            resultCount.value = 0;
            notify("Results cleared", "success");
        } catch (err) {
            handleError("Clear results", err);
        }
    };

    // ==========================================
    // Polling
    // ==========================================

    /**
     * Start the status-driven polling loop. On each tick it pulls status and
     * results; once the backend reports the attack is no longer running it does
     * a final results refresh and stops polling.
     */
    const startPolling = (): void => {
        stopPolling();
        const generation = ++pollGeneration;
        pollTimer = setInterval(async () => {
            try {
                const { data: status } = await api.get<IntruderStatus>(
                    "/api/intruder/status",
                );
                // Stale run: a newer start/stop already took over. Skip applying
                // state so an older response cannot clobber the new attack.
                if (generation !== pollGeneration) {
                    return;
                }
                if (status) {
                    running.value = status.running;
                    resultCount.value = status.result_count;
                }

                const { data: rows } = await api.get<IntruderResult[]>(
                    "/api/intruder/results",
                );
                if (generation !== pollGeneration) {
                    return;
                }
                results.value = rows ?? [];

                if (status && !status.running) {
                    stopPolling();
                }
            } catch (err) {
                if (generation !== pollGeneration) {
                    return;
                }
                handleError("Poll attack", err);
            }
        }, INTRUDER_POLL_INTERVAL_MS);
    };

    /**
     * Stop the polling loop if it is active. Also invalidates in-flight poll
     * callbacks so they cannot tear down a later run.
     */
    const stopPolling = (): void => {
        pollGeneration += 1;
        if (pollTimer !== null) {
            clearInterval(pollTimer);
            pollTimer = null;
        }
    };

    // ==========================================
    // Return Interface
    // ==========================================

    return {
        running,
        isOperating,
        results,
        resultCount,
        lastError,
        loadPayloads,
        getStatus,
        refreshResults,
        generatePreview,
        startAttack,
        stopAttack,
        clearResults,
        stopPolling,
    };
}
