<template>
    <div class="system-health-panel">
        <div class="panel-header">
            <h2 class="panel-title">🏥 System Health</h2>
            <div class="health-score">
                <span class="health-label">Overall</span>
                <span class="health-value" :class="overallHealthStatus"
                    >{{ overallHealthScore }}%</span
                >
            </div>
        </div>

        <!-- Health Gauges Grid -->
        <div class="gauges-grid">
            <div class="gauge-container">
                <ProgressRing
                    :value="cpuUsage"
                    label="CPU"
                    :size="140"
                    :stroke-width="5"
                    :status="cpuHealthStatus"
                />
            </div>

            <div class="gauge-container">
                <ProgressRing
                    :value="memoryUsage"
                    label="Memory"
                    :size="140"
                    :stroke-width="5"
                    :status="memoryHealthStatus"
                />
            </div>

            <div class="gauge-container">
                <ProgressRing
                    :value="diskUsage"
                    label="Disk"
                    :size="140"
                    :stroke-width="5"
                    :status="diskHealthStatus"
                />
            </div>

            <div class="gauge-container">
                <ProgressRing
                    :value="networkUsage"
                    label="Network"
                    :size="140"
                    :stroke-width="5"
                    :status="networkHealthStatus"
                />
            </div>
        </div>

        <!-- Connection Stats -->
        <div class="connection-stats">
            <div class="stat-item">
                <span class="stat-label">Active Connections</span>
                <span class="stat-value">{{ metrics?.active_connections ?? 0 }}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Uptime</span>
                <span class="stat-value">{{ formatUptime(metrics?.uptime_seconds ?? 0) }}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Proxy Status</span>
                <StatusBadge
                    :status="proxyStatus"
                    :label="proxyRunning ? 'Running' : 'Stopped'"
                    size="sm"
                />
            </div>
        </div>

        <!-- Health Alerts -->
        <div v-if="healthAlerts.length > 0" class="health-alerts">
            <div
                v-for="alert in healthAlerts"
                :key="alert.id"
                class="alert-item"
                :class="alert.severity"
            >
                <span class="alert-icon">{{ alert.icon }}</span>
                <div class="alert-content">
                    <div class="alert-title">{{ alert.title }}</div>
                    <div class="alert-message">{{ alert.message }}</div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import ProgressRing from "./ProgressRing.vue";
import StatusBadge from "./StatusBadge.vue";
import { formatUptime } from "@/utils/dashboard/formatters";
import { getHealthStatus } from "@/utils/dashboard/thresholds";
import type { SystemMetrics } from "@/types/dashboard";

interface Props {
    metrics: SystemMetrics | null;
    proxyRunning: boolean;
}

const props = withDefaults(defineProps<Props>(), {
    proxyRunning: true,
});

// CPU Usage (0-100)
const cpuUsage = computed(() => {
    return props.metrics?.cpu_percent ?? 0;
});

const cpuHealthStatus = computed(() => {
    return getHealthStatus(cpuUsage.value, { warning: 70, critical: 85 });
});

// Memory Usage (0-100)
const memoryUsage = computed(() => {
    if (!props.metrics) return 0;
    // Assuming memory_usage_mb / estimate of total (8GB typical)
    const estimatedTotalMb = 8192;
    return Math.round((props.metrics.memory_usage_mb / estimatedTotalMb) * 100);
});

const memoryHealthStatus = computed(() => {
    return getHealthStatus(memoryUsage.value, { warning: 70, critical: 85 });
});

// Disk Usage (0-100)
const diskUsage = computed(() => {
    if (!props.metrics) return 0;
    return props.metrics.disk_percent ?? 0;
});

const diskHealthStatus = computed(() => {
    return getHealthStatus(diskUsage.value, { warning: 75, critical: 90 });
});

// Network Usage (0-100) - calculated from current I/O
const networkUsage = computed(() => {
    if (!props.metrics) return 0;
    const maxIOPerSec = 1000; // MB/s max for visualization
    const currentIO = (props.metrics.bytes_in_sec + props.metrics.bytes_out_sec) / 1024 / 1024;
    return Math.min(Math.round((currentIO / maxIOPerSec) * 100), 100);
});

const networkHealthStatus = computed(() => {
    return getHealthStatus(networkUsage.value, { warning: 70, critical: 85 });
});

// Overall Health Score (average of all metrics)
const overallHealthScore = computed(() => {
    const scores = [cpuUsage.value, memoryUsage.value, diskUsage.value, networkUsage.value];
    const average = Math.round(scores.reduce((a, b) => a + b, 0) / scores.length);
    return Math.max(0, 100 - average); // Invert so lower usage = higher health
});

const overallHealthStatus = computed(() => {
    return getHealthStatus(100 - overallHealthScore.value, { warning: 70, critical: 85 });
});

const proxyStatus = computed(() => {
    return props.proxyRunning ? "healthy" : "critical";
});

// Health Alerts
const healthAlerts = computed(() => {
    const alerts: Array<{
        id: string;
        severity: "warning" | "critical";
        icon: string;
        title: string;
        message: string;
    }> = [];

    if (cpuUsage.value > 85) {
        alerts.push({
            id: "cpu-critical",
            severity: "critical",
            icon: "⚠️",
            title: "High CPU Usage",
            message: `CPU is at ${cpuUsage.value}%. Consider investigating heavy processes.`,
        });
    } else if (cpuUsage.value > 70) {
        alerts.push({
            id: "cpu-warning",
            severity: "warning",
            icon: "⚡",
            title: "Elevated CPU Usage",
            message: `CPU is at ${cpuUsage.value}%.`,
        });
    }

    if (memoryUsage.value > 85) {
        alerts.push({
            id: "memory-critical",
            severity: "critical",
            icon: "⚠️",
            title: "High Memory Usage",
            message: `Memory is at ${memoryUsage.value}%. Consider restarting services.`,
        });
    } else if (memoryUsage.value > 70) {
        alerts.push({
            id: "memory-warning",
            severity: "warning",
            icon: "💾",
            title: "Elevated Memory Usage",
            message: `Memory is at ${memoryUsage.value}%.`,
        });
    }

    if (diskUsage.value > 90) {
        alerts.push({
            id: "disk-critical",
            severity: "critical",
            icon: "⚠️",
            title: "Critical Disk Space",
            message: `Disk is ${diskUsage.value}% full. Free up space urgently.`,
        });
    } else if (diskUsage.value > 75) {
        alerts.push({
            id: "disk-warning",
            severity: "warning",
            icon: "📁",
            title: "Low Disk Space",
            message: `Disk is ${diskUsage.value}% full.`,
        });
    }

    if (!props.proxyRunning) {
        alerts.push({
            id: "proxy-offline",
            severity: "critical",
            icon: "🛑",
            title: "Proxy Offline",
            message: "The proxy service is not running.",
        });
    }

    return alerts;
});
</script>

<style scoped>
.system-health-panel {
    padding: 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(26, 26, 46, 0.5);
    color: #ffffff;
}

.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.panel-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #ffffff;
}

.health-score {
    display: flex;
    align-items: center;
    gap: 8px;
}

.health-label {
    font-size: 11px;
    text-transform: uppercase;
    color: #606060;
    font-weight: 600;
    letter-spacing: 0.5px;
}

.health-value {
    font-size: 20px;
    font-weight: 700;
    font-family: "Fira Code", monospace;
    padding: 4px 12px;
    border-radius: 4px;
    background: rgba(0, 212, 255, 0.1);
    color: #00d4ff;
    transition: all 0.3s ease;
}

.health-value.critical {
    background: rgba(255, 0, 110, 0.1);
    color: #ff006e;
}

.health-value.warning {
    background: rgba(255, 184, 0, 0.1);
    color: #ffb800;
}

/* Gauges Grid */
.gauges-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 20px;
    margin-bottom: 24px;
}

.gauge-container {
    display: flex;
    justify-content: center;
    align-items: center;
}

/* Connection Stats */
.connection-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 16px;
    padding: 16px;
    background: rgba(0, 212, 255, 0.05);
    border-radius: 6px;
    margin-bottom: 16px;
}

.stat-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.stat-label {
    font-size: 11px;
    text-transform: uppercase;
    color: #606060;
    font-weight: 600;
    letter-spacing: 0.5px;
}

.stat-value {
    font-size: 16px;
    font-weight: 700;
    color: #00d4ff;
    font-family: "Fira Code", monospace;
}

/* Health Alerts */
.health-alerts {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.alert-item {
    display: flex;
    gap: 12px;
    padding: 12px;
    border-radius: 6px;
    border-left: 3px solid;
    background: rgba(255, 255, 255, 0.05);
    transition: all 0.2s ease;
}

.alert-item.warning {
    background: rgba(255, 184, 0, 0.1);
    border-left-color: #ffb800;
}

.alert-item.critical {
    background: rgba(255, 0, 110, 0.1);
    border-left-color: #ff006e;
}

.alert-icon {
    font-size: 18px;
    flex-shrink: 0;
}

.alert-content {
    flex: 1;
    min-width: 0;
}

.alert-title {
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
    margin-bottom: 2px;
}

.alert-message {
    font-size: 12px;
    color: #a0a0a0;
    line-height: 1.4;
}

/* Responsive */
@media (max-width: 1366px) {
    .gauges-grid {
        grid-template-columns: repeat(2, 1fr);
    }
}

@media (max-width: 768px) {
    .gauges-grid {
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
    }

    .connection-stats {
        grid-template-columns: 1fr;
    }

    .panel-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
    }

    .health-score {
        width: 100%;
        justify-content: space-between;
    }
}
</style>
