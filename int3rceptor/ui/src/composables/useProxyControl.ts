/**
 * useProxyControl Composable
 *
 * Manages proxy server control operations (start, stop, clear traffic, export).
 * - POST /api/proxy/start - Start the proxy server
 * - POST /api/proxy/stop - Stop the proxy server
 * - DELETE /api/traffic - Clear captured traffic
 * - POST /api/dashboard/export - Export metrics in various formats
 *
 * @example
 * const { startProxy, stopProxy, clearTraffic, exportMetrics } = useProxyControl();
 * await startProxy(); // Start proxy and show notification
 */

import { ref, type Ref } from "vue";
import { api } from "./useApi";
import type { ProxyStatus } from "@/types/dashboard";

// ==========================================
// Configuration Constants
// ==========================================

/** Request timeout for proxy control operations */
const OPERATION_TIMEOUT_MS = 10000; // 10 seconds for proxy operations

// ==========================================
// Type Definitions
// ==========================================

/**
 * Export format type
 */
export type ExportFormat = "json" | "csv" | "har";

/**
 * Result of a control operation
 */
interface OperationResult {
    success: boolean;
    message: string;
    data?: any;
}

/**
 * Notification callback function type
 */
export type NotificationCallback = (
    message: string,
    type: "success" | "error" | "warning" | "info",
) => void;

/**
 * Return type for the useProxyControl composable
 */
interface UseProxyControlReturn {
    isProxyRunning: Ref<boolean>;
    isOperating: Ref<boolean>;
    lastOperation: Ref<string | null>;
    proxyStatus: Ref<ProxyStatus | null>;
    startProxy: () => Promise<void>;
    stopProxy: () => Promise<void>;
    toggleProxy: () => Promise<void>;
    clearTraffic: () => Promise<void>;
    exportMetrics: (format: ExportFormat) => Promise<void>;
    getProxyStatus: () => Promise<ProxyStatus | null>;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing proxy server control operations
 *
 * Handles:
 * - Starting and stopping the proxy server
 * - Clearing captured traffic
 * - Exporting metrics in various formats
 * - Error handling and user notifications
 * - Operation state tracking
 *
 * @param {NotificationCallback} [onNotify] - Optional callback for notifications
 * @returns {UseProxyControlReturn} Proxy control state and methods
 */
export function useProxyControl(onNotify?: NotificationCallback): UseProxyControlReturn {
    // ==========================================
    // State
    // ==========================================

    /** Whether the proxy is currently running */
    const isProxyRunning = ref(false);

    /** Whether an operation is in progress */
    const isOperating = ref(false);

    /** Last operation performed */
    const lastOperation = ref<string | null>(null);

    /** Current proxy status information */
    const proxyStatus = ref<ProxyStatus | null>(null);

    // ==========================================
    // Helper Functions
    // ==========================================

    /**
     * Show notification to user
     *
     * @param {string} message - Notification message
     * @param {string} type - Notification type (success, error, warning, info)
     */
    const notify = (
        message: string,
        type: "success" | "error" | "warning" | "info" = "info",
    ): void => {
        console.log(`[useProxyControl] ${type.toUpperCase()}: ${message}`);

        if (onNotify) {
            onNotify(message, type);
        }
    };

    /**
     * Handle operation error with logging and notification
     *
     * @param {string} operation - Operation name
     * @param {any} error - Error object
     * @returns {OperationResult} Operation result with error details
     */
    const handleError = (operation: string, error: any): OperationResult => {
        let errorMsg = `${operation} failed`;

        if (error instanceof Error) {
            errorMsg = error.message;
        } else if (typeof error === "string") {
            errorMsg = error;
        }

        console.error(`[useProxyControl] ${operation} error:`, error);
        notify(errorMsg, "error");

        return {
            success: false,
            message: errorMsg,
        };
    };

    // ==========================================
    // Core Functions
    // ==========================================

    /**
     * Get current proxy status
     *
     * Fetches the latest proxy status from the API without modifying state
     *
     * @returns {Promise<ProxyStatus | null>} Current proxy status or null if error
     */
    const getProxyStatus = async (): Promise<ProxyStatus | null> => {
        try {
            const response = await api.get<ProxyStatus>("/api/proxy/status");

            if (response.data) {
                proxyStatus.value = response.data;
                isProxyRunning.value = response.data.running;

                if (import.meta.env.DEV) {
                    console.debug("[useProxyControl] Proxy status fetched", {
                        running: response.data.running,
                        port: response.data.port,
                    });
                }

                return response.data;
            }
        } catch (err) {
            handleError("Get proxy status", err);
        }

        return null;
    };

    /**
     * Start the proxy server
     *
     * Sends POST /api/proxy/start request
     * Updates isProxyRunning state on success
     *
     * @throws Will not throw; errors are handled with notifications
     */
    const startProxy = async (): Promise<void> => {
        if (isOperating.value) {
            console.warn("[useProxyControl] Operation already in progress");
            return;
        }

        if (isProxyRunning.value) {
            notify("Proxy is already running", "warning");
            return;
        }

        isOperating.value = true;
        lastOperation.value = "start_proxy";

        try {
            const response = await api.post<ProxyStatus>("/api/proxy/start");

            if (response.data) {
                proxyStatus.value = response.data;
                isProxyRunning.value = true;

                notify(
                    `Proxy started on ${response.data.host}:${response.data.port}`,
                    "success",
                );

                if (import.meta.env.DEV) {
                    console.debug("[useProxyControl] Proxy started successfully", {
                        host: response.data.host,
                        port: response.data.port,
                        tls_enabled: response.data.tls_enabled,
                    });
                }
            }
        } catch (err) {
            isProxyRunning.value = false;
            handleError("Start proxy", err);
        } finally {
            isOperating.value = false;
        }
    };

    /**
     * Stop the proxy server
     *
     * Sends POST /api/proxy/stop request
     * Updates isProxyRunning state on success
     *
     * @throws Will not throw; errors are handled with notifications
     */
    const stopProxy = async (): Promise<void> => {
        if (isOperating.value) {
            console.warn("[useProxyControl] Operation already in progress");
            return;
        }

        if (!isProxyRunning.value) {
            notify("Proxy is not running", "warning");
            return;
        }

        isOperating.value = true;
        lastOperation.value = "stop_proxy";

        try {
            const response = await api.post<ProxyStatus>("/api/proxy/stop");

            if (response.data) {
                proxyStatus.value = response.data;
                isProxyRunning.value = false;

                notify("Proxy stopped successfully", "success");

                if (import.meta.env.DEV) {
                    console.debug("[useProxyControl] Proxy stopped successfully");
                }
            }
        } catch (err) {
            isProxyRunning.value = true;
            handleError("Stop proxy", err);
        } finally {
            isOperating.value = false;
        }
    };

    /**
     * Toggle proxy server (start if stopped, stop if running)
     *
     * Convenience method for toggle button UI patterns
     *
     * @throws Will not throw; errors are handled with notifications
     */
    const toggleProxy = async (): Promise<void> => {
        if (isProxyRunning.value) {
            await stopProxy();
        } else {
            await startProxy();
        }
    };

    /**
     * Clear all captured traffic from the proxy
     *
     * Sends DELETE /api/traffic request
     * Shows notification with count of cleared requests
     *
     * @throws Will not throw; errors are handled with notifications
     */
    const clearTraffic = async (): Promise<void> => {
        if (isOperating.value) {
            console.warn("[useProxyControl] Operation already in progress");
            return;
        }

        isOperating.value = true;
        lastOperation.value = "clear_traffic";

        try {
            const response = await api.delete<{ cleared_count: number }>("/api/traffic");

            if (response.data) {
                const count = response.data.cleared_count;
                notify(`Cleared ${count} captured requests`, "success");

                if (import.meta.env.DEV) {
                    console.debug("[useProxyControl] Traffic cleared successfully", {
                        cleared_count: count,
                    });
                }
            }
        } catch (err) {
            handleError("Clear traffic", err);
        } finally {
            isOperating.value = false;
        }
    };

    /**
     * Export metrics and traffic data in specified format
     *
     * Sends POST /api/dashboard/export?format=FORMAT request
     * Triggers file download in browser
     *
     * Supported formats:
     * - json: JSON format with metrics and traffic
     * - csv: CSV spreadsheet format
     * - har: HTTP Archive format (standard for web traffic)
     *
     * @param {ExportFormat} format - Export format (json, csv, or har)
     * @throws Will not throw; errors are handled with notifications
     */
    const exportMetrics = async (format: ExportFormat = "json"): Promise<void> => {
        if (isOperating.value) {
            console.warn("[useProxyControl] Operation already in progress");
            return;
        }

        if (!["json", "csv", "har"].includes(format)) {
            notify(`Invalid export format: ${format}`, "warning");
            return;
        }

        isOperating.value = true;
        lastOperation.value = `export_${format}`;

        try {
            // Use Blob API for file downloads
            const response = await api.post(
                `/api/dashboard/export?format=${format}`,
                {},
                {
                    responseType: "blob",
                },
            );

            if (response.data) {
                // Create blob and trigger download
                const blob = new Blob([response.data], {
                    type: getMimeType(format),
                });
                const url = URL.createObjectURL(blob);
                const link = document.createElement("a");
                link.href = url;
                link.download = `metrics-${Date.now()}.${getFileExtension(format)}`;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                URL.revokeObjectURL(url);

                notify(`Metrics exported as ${format.toUpperCase()}`, "success");

                if (import.meta.env.DEV) {
                    console.debug("[useProxyControl] Metrics exported successfully", {
                        format: format,
                        fileSize: blob.size,
                    });
                }
            }
        } catch (err) {
            handleError(`Export ${format}`, err);
        } finally {
            isOperating.value = false;
        }
    };

    // ==========================================
    // Utility Functions
    // ==========================================

    /**
     * Get MIME type for export format
     *
     * @param {ExportFormat} format - Export format
     * @returns {string} MIME type
     */
    const getMimeType = (format: ExportFormat): string => {
        const mimeTypes: Record<ExportFormat, string> = {
            json: "application/json",
            csv: "text/csv",
            har: "application/json",
        };
        return mimeTypes[format] || "application/octet-stream";
    };

    /**
     * Get file extension for export format
     *
     * @param {ExportFormat} format - Export format
     * @returns {string} File extension
     */
    const getFileExtension = (format: ExportFormat): string => {
        const extensions: Record<ExportFormat, string> = {
            json: "json",
            csv: "csv",
            har: "har",
        };
        return extensions[format] || "bin";
    };

    // ==========================================
    // Return Interface
    // ==========================================

    return {
        // State
        isProxyRunning,
        isOperating,
        lastOperation,
        proxyStatus,

        // Methods
        startProxy,
        stopProxy,
        toggleProxy,
        clearTraffic,
        exportMetrics,
        getProxyStatus,
    };
}
