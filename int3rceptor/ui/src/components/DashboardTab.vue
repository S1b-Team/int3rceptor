<template>
    <div class="dashboard-tab">
        <!-- Header with status and controls -->
        <div class="dashboard-header">
            <div class="header-left">
                <h1 class="dashboard-title">📊 Dashboard</h1>
                <span class="subtitle">Real-time metrics & system health</span>
            </div>
            <div class="header-right">
                <div class="status-indicators">
                    <div class="status-item">
                        <span class="status-label">Proxy</span>
                        <span class="status-badge" :class="proxyRunning ? 'running' : 'stopped'">
                            {{ proxyRunning ? "● Running" : "○ Stopped" }}
                        </span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">WS</span>
                        <span
                            class="status-badge"
                            :class="wsConnected ? 'connected' : 'disconnected'"
                        >
                            {{ wsConnected ? "● Connected" : "○ Offline" }}
                        </span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Requests</span>
                        <span class="status-value"
                            >{{ metrics?.requests_per_sec.toFixed(0) ?? "0" }}/s</span
                        >
                    </div>
                    <div class="status-item">
                        <span class="status-label">Memory</span>
                        <span class="status-value"
                            >{{ metrics?.memory_usage_mb.toFixed(1) ?? "0" }}MB</span
                        >
                    </div>
                    <div class="status-item">
                        <span class="status-label">API</span>
                        <span
                            class="status-badge"
                            :class="{ 'api-live': !isLoading, 'api-loading': isLoading }"
                        >
                            {{ isLoading ? "⏳ Fetching..." : "✓ Live" }}
                        </span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Last Update</span>
                        <span class="status-value">{{ formatLastUpdate }}</span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Error Alert -->
        <div v-if="error && showErrorNotification" class="error-alert">
            <span class="error-icon">⚠️</span>
            <span class="error-message">{{ error }}</span>
            <button class="close-btn" @click="dismissError">✕</button>
        </div>

        <!-- Loading State -->
        <div v-if="isLoading && !metrics" class="loading-state">
            <div class="spinner"></div>
            <p>Loading dashboard metrics...</p>
        </div>

        <!-- Main Content -->
        <template v-else-if="metrics">
            <div class="dashboard-content">
                <!-- Metrics Grid -->
                <MetricsGrid :metrics="metrics" :time-series="timeSeries" :is-loading="isLoading" />

                <!-- System Health Panel -->
                <SystemHealthPanel :metrics="metrics" :proxy-running="proxyRunning" />

                <!-- Activity Chart -->
                <ActivityChart :time-series="timeSeries" />

                <!-- Connection Graph -->
                <ConnectionGraph :connections="connectionStats" :metrics="metrics" />

                <!-- Recent Activity Panel -->
                <RecentActivityPanel
                    :activities="recentActivity"
                    :max-display="10"
                    @refresh="refreshActivity"
                    @clear="clearActivity"
                    @action="(activity: any) => handleActivityAction(activity)"
                />
            </div>

            <!-- Quick Actions -->
            <div class="quick-actions">
                <button class="btn-primary" @click="toggleProxy" :disabled="isUpdating">
                    {{ proxyRunning ? "⏹️ Stop Proxy" : "▶️ Start Proxy" }}
                </button>
                <button class="btn-secondary" @click="clearTraffic" :disabled="isUpdating">
                    🗑️ Clear Traffic
                </button>
                <button class="btn-secondary" @click="handleRefresh" :disabled="isLoading">
                    🔄 Refresh
                </button>
                <button class="btn-secondary" @click="exportMetrics" :disabled="isUpdating">
                    📥 Export
                </button>
                <button class="btn-secondary" @click="openSettings">⚙️ Settings</button>
            </div>
        </template>

        <!-- Empty State -->
        <div v-else class="empty-state">
            <p>No metrics available</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { useDashboardMetrics } from "@/composables/dashboard/useDashboardMetrics";
import MetricsGrid from "./dashboard/MetricsGrid.vue";
import SystemHealthPanel from "./dashboard/SystemHealthPanel.vue";
import ActivityChart from "./dashboard/ActivityChart.vue";
import ConnectionGraph from "./dashboard/ConnectionGraph.vue";
import RecentActivityPanel from "./dashboard/RecentActivityPanel.vue";
import { formatTime } from "@/utils/dashboard/formatters";
import type {
    SystemMetrics,
    TimeSeriesData,
    ActivityEntry,
    ConnectionStats,
} from "@/types/dashboard";

// Composables
const {
    metrics,
    isLoading,
    error,
    lastUpdated,
    timeSinceUpdate,
    startAutoFetch,
    stopAutoFetch,
    clearError,
    fetchMetrics,
} = useDashboardMetrics();

// State
const wsConnected = ref(false);
const proxyRunning = ref(true);
const isUpdating = ref(false);
const showErrorNotification = ref(false);
let errorNotificationTimeout: ReturnType<typeof setTimeout> | null = null;

const timeSeries = ref<Map<string, TimeSeriesData>>(new Map());
const recentActivity = ref<
    Array<{
        id: string;
        timestamp: number;
        type: "request" | "error" | "warning" | "info";
        message: string;
        actionLabel?: string;
        details?: string;
    }>
>([]);
const connectionStats = ref<ConnectionStats>({
    active: 8,
    established: 6,
    closing: 1,
    failed: 0,
    total_lifetime: 150,
    avg_duration_ms: 2500,
    peak_connections: 45,
    peak_time: Date.now() - 3600000,
    concurrent_limit: 100,
});

// Computed properties
const formatLastUpdate = computed(() => {
    if (!lastUpdated.value) return "—";
    const ms = timeSinceUpdate.value;
    if (ms < 1000) return "now";
    if (ms < 60000) return `${(ms / 1000).toFixed(0)}s ago`;
    return `${(ms / 60000).toFixed(0)}m ago`;
});

// Lifecycle
onMounted(() => {
    startAutoFetch(1000); // Poll every 1 second

    // Mock activity data for development
    recentActivity.value = [
        {
            id: "1",
            timestamp: Date.now(),
            type: "request",
            message: "GET /api/users",
        },
        {
            id: "2",
            timestamp: Date.now() - 5000,
            type: "info",
            message: "Proxy started on port 8080",
        },
        {
            id: "3",
            timestamp: Date.now() - 10000,
            type: "warning",
            message: "High memory usage detected",
        },
        {
            id: "4",
            timestamp: Date.now() - 15000,
            type: "request",
            message: "POST /api/config",
        },
        {
            id: "5",
            timestamp: Date.now() - 20000,
            type: "info",
            message: "WebSocket connection established",
        },
    ];

    // Initialize time series data with mock data
    const mockTimeSeriesData: TimeSeriesData = {
        metric: "requests",
        unit: "req/s",
        dataPoints: Array.from({ length: 60 }, (_, i) => ({
            timestamp: Date.now() - (60 - i) * 1000,
            value: Math.random() * 100 + 50,
        })),
        current: 75,
        min: 10,
        max: 120,
        avg: 65,
        percentile_95: 110,
        percentile_99: 115,
    };

    timeSeries.value.set("requests", mockTimeSeriesData);
});

onBeforeUnmount(() => {
    stopAutoFetch();
});

// Methods
const dismissError = () => {
    clearError();
    showErrorNotification.value = false;
    if (errorNotificationTimeout) {
        clearTimeout(errorNotificationTimeout);
    }
};

const handleError = () => {
    showErrorNotification.value = true;
    if (errorNotificationTimeout) {
        clearTimeout(errorNotificationTimeout);
    }
    // Auto-dismiss error notification after 5 seconds
    errorNotificationTimeout = setTimeout(() => {
        showErrorNotification.value = false;
    }, 5000);
};

const handleRefresh = async () => {
    await fetchMetrics();
    console.log("✓ Metrics refreshed manually");
};

const toggleProxy = async () => {
    isUpdating.value = true;
    try {
        // TODO: Call API to toggle proxy
        // POST /api/proxy/start or POST /api/proxy/stop
        proxyRunning.value = !proxyRunning.value;
        console.log(`Proxy ${proxyRunning.value ? "started" : "stopped"}`);
    } catch (err) {
        console.error("Failed to toggle proxy:", err);
        error.value = "Failed to toggle proxy";
        handleError();
    } finally {
        isUpdating.value = false;
    }
};

const clearTraffic = async () => {
    isUpdating.value = true;
    try {
        // TODO: Call API to clear traffic
        // DELETE /api/traffic
        console.log("✓ Traffic cleared");
    } catch (err) {
        console.error("Failed to clear traffic:", err);
        error.value = "Failed to clear traffic";
        handleError();
    } finally {
        isUpdating.value = false;
    }
};

const clearActivity = () => {
    recentActivity.value = [];
    console.log("✓ Activity cleared");
};

const refreshActivity = () => {
    // TODO: Refetch activity from API (Task 2)
    // GET /api/dashboard/activity?limit=50&offset=0
    console.log("Refreshing activity...");
};

const handleActivityAction = (activity: {
    id: string;
    timestamp: number;
    type: "request" | "error" | "warning" | "info";
    message: string;
    actionLabel?: string;
    details?: string;
}) => {
    console.log("Activity action:", activity);
    // TODO: Handle activity-specific actions (e.g., view request details, navigate to request detail tab)
};

const exportMetrics = async () => {
    isUpdating.value = true;
    try {
        // TODO: Call API to export metrics
        // POST /api/dashboard/export?format=json
        console.log("✓ Metrics exported");
    } catch (err) {
        console.error("Failed to export metrics:", err);
        error.value = "Failed to export metrics";
        handleError();
    } finally {
        isUpdating.value = false;
    }
};

const openSettings = () => {
    // TODO: Open settings modal
    console.log("Opening settings...");
};
</script>

<style scoped>
.dashboard-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0a0a0f;
    color: #ffffff;
    font-family: "Inter", "Roboto", "Segoe UI", sans-serif;
    overflow-y: auto;
    gap: 20px;
    padding-bottom: 20px;
}

.dashboard-content {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 0 20px;
}

/* Header */
.dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: linear-gradient(180deg, #1a1a2e 0%, #0a0a0f 100%);
    gap: 20px;
}

.header-left {
    flex: 1;
}

.dashboard-title {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    color: #ffffff;
}

.subtitle {
    font-size: 12px;
    color: #a0a0a0;
    margin-top: 4px;
}

.header-right {
    flex: 1;
    display: flex;
    justify-content: flex-end;
}

.status-indicators {
    display: flex;
    gap: 20px;
    flex-wrap: wrap;
}

.status-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-end;
}

.status-label {
    font-size: 10px;
    text-transform: uppercase;
    color: #606060;
    font-weight: 600;
    letter-spacing: 0.5px;
}

.status-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 12px;
    transition: all 0.3s ease;
}

.status-badge.running,
.status-badge.connected,
.status-badge.api-live {
    color: #00d4ff;
    background: rgba(0, 212, 255, 0.1);
}

.status-badge.stopped,
.status-badge.disconnected,
.status-badge.api-loading {
    color: #ffb800;
    background: rgba(255, 184, 0, 0.1);
}

.status-value {
    font-size: 12px;
    font-weight: 600;
    color: #ffb800;
    font-family: "Fira Code", monospace;
}

/* Error Alert */
.error-alert {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    background: rgba(255, 0, 110, 0.1);
    border-bottom: 1px solid rgba(255, 0, 110, 0.2);
    color: #ff6b9d;
}

.error-icon {
    font-size: 16px;
    flex-shrink: 0;
}

.error-message {
    flex: 1;
    font-size: 13px;
}

.close-btn {
    background: transparent;
    border: none;
    color: #ff006e;
    cursor: pointer;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 3px;
    transition: all 0.2s;
}

.close-btn:hover {
    background: rgba(255, 0, 110, 0.2);
}

/* Loading State */
.loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 300px;
    gap: 16px;
}

.spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(0, 212, 255, 0.2);
    border-top-color: #00d4ff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.loading-state p {
    color: #a0a0a0;
    font-size: 14px;
}

/* Activity Section */
.activity-section {
    padding: 20px;
    margin: 0 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(26, 26, 46, 0.5);
}

.section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.section-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
}

.activity-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 200px;
    overflow-y: auto;
}

.activity-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    background: rgba(0, 212, 255, 0.05);
    border-radius: 4px;
    font-size: 12px;
}

.activity-time {
    color: #606060;
    font-family: "Fira Code", monospace;
    min-width: 60px;
}

.activity-type {
    padding: 2px 6px;
    border-radius: 3px;
    font-weight: 600;
    text-transform: uppercase;
    font-size: 10px;
    min-width: 60px;
}

.activity-type.request {
    background: rgba(0, 212, 255, 0.2);
    color: #00d4ff;
}

.activity-type.error {
    background: rgba(255, 0, 110, 0.2);
    color: #ff006e;
}

.activity-type.warning {
    background: rgba(255, 184, 0, 0.2);
    color: #ffb800;
}

.activity-type.info {
    background: rgba(139, 92, 246, 0.2);
    color: #8b5cf6;
}

.activity-message {
    flex: 1;
    color: #a0a0a0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: #606060;
    font-size: 14px;
}

/* Quick Actions */
.quick-actions {
    display: flex;
    gap: 12px;
    padding: 20px;
    flex-wrap: wrap;
}

.btn-primary,
.btn-secondary {
    padding: 10px 16px;
    border-radius: 4px;
    font-weight: 600;
    font-size: 13px;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.btn-primary {
    background: #00d4ff;
    color: #000;
}

.btn-primary:hover:not(:disabled) {
    background: #33ddff;
    box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
}

.btn-secondary {
    background: transparent;
    border: 1px solid rgba(0, 212, 255, 0.3);
    color: #00d4ff;
}

.btn-secondary:hover:not(:disabled) {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.6);
}

.btn-primary:disabled,
.btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* Responsive */
@media (max-width: 1366px) {
    .dashboard-header {
        flex-direction: column;
        align-items: flex-start;
    }

    .header-right {
        width: 100%;
        justify-content: flex-start;
    }

    .status-indicators {
        width: 100%;
        justify-content: flex-start;
    }
}

@media (max-width: 1366px) {
    .dashboard-content {
        padding: 0 16px;
    }
}

@media (max-width: 768px) {
    .dashboard-header {
        padding: 16px;
    }

    .dashboard-title {
        font-size: 20px;
    }

    .status-indicators {
        font-size: 12px;
        gap: 12px;
    }

    .dashboard-content {
        padding: 0 12px;
        gap: 16px;
    }

    .quick-actions {
        margin-left: 12px;
        margin-right: 12px;
    }
}
</style>
