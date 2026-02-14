<template>
    <div class="connection-graph-container">
        <!-- Header -->
        <div class="graph-header">
            <h3 class="graph-title">🔗 Connection Status</h3>
            <div class="header-stats">
                <span class="stat-badge">
                    <span class="stat-label">Active</span>
                    <span class="stat-value">{{ activeConnections }}</span>
                </span>
                <span class="stat-badge">
                    <span class="stat-label">Total</span>
                    <span class="stat-value">{{ totalConnections }}</span>
                </span>
            </div>
        </div>

        <!-- Connection Visualization -->
        <div class="graph-content">
            <svg class="connection-svg" viewBox="0 0 400 300" preserveAspectRatio="xMidYMid meet">
                <!-- Central Server Node -->
                <circle
                    class="node server-node"
                    cx="200"
                    cy="150"
                    r="30"
                    fill="rgba(0, 212, 255, 0.2)"
                    stroke="#00d4ff"
                    stroke-width="2"
                />
                <text
                    x="200"
                    y="155"
                    text-anchor="middle"
                    font-size="14"
                    fill="#00d4ff"
                    font-weight="bold"
                >
                    Server
                </text>

                <!-- Client Nodes (animated) -->
                <g v-for="(client, idx) in clientNodes" :key="`client-${idx}`" class="client-group">
                    <!-- Connection Line -->
                    <line
                        :x1="200"
                        :y1="150"
                        :x2="client.x"
                        :y2="client.y"
                        stroke="rgba(0, 212, 255, 0.3)"
                        stroke-width="1"
                        class="connection-line"
                    />

                    <!-- Client Node -->
                    <circle
                        :cx="client.x"
                        :cy="client.y"
                        r="15"
                        :fill="
                            client.active ? 'rgba(0, 212, 255, 0.3)' : 'rgba(160, 160, 160, 0.2)'
                        "
                        :stroke="client.active ? '#00d4ff' : '#606060'"
                        stroke-width="2"
                        class="client-node"
                    />

                    <!-- Activity Indicator -->
                    <circle
                        v-if="client.active"
                        :cx="client.x"
                        :cy="client.y"
                        r="20"
                        fill="none"
                        stroke="#00d4ff"
                        stroke-width="1"
                        opacity="0.5"
                        class="activity-pulse"
                    />
                </g>
            </svg>
        </div>

        <!-- Connection Details -->
        <div class="connection-details">
            <div class="detail-item">
                <span class="detail-label">Connection Rate</span>
                <span class="detail-value">{{ connectionRate }}/s</span>
            </div>
            <div class="detail-item">
                <span class="detail-label">Avg Duration</span>
                <span class="detail-value">{{ avgDuration }}ms</span>
            </div>
            <div class="detail-item">
                <span class="detail-label">Closed</span>
                <span class="detail-value">{{ closedConnections }}</span>
            </div>
            <div class="detail-item">
                <span class="detail-label">Failed</span>
                <span class="detail-value" :class="{ 'is-warning': failedConnections > 0 }">
                    {{ failedConnections }}
                </span>
            </div>
        </div>

        <!-- Connection List -->
        <div class="connection-list">
            <div class="list-header">
                <span class="list-title">Recent Connections</span>
                <span class="list-count">{{ recentConnections.length }}</span>
            </div>
            <div class="list-items">
                <div
                    v-for="conn in recentConnections.slice(0, 5)"
                    :key="conn.id"
                    class="list-item"
                    :class="{ 'is-active': conn.active }"
                >
                    <span class="connection-status" :class="{ 'is-active': conn.active }"></span>
                    <span class="connection-ip">{{ conn.ip }}</span>
                    <span class="connection-duration">{{ formatDuration(conn.duration) }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import type { ConnectionStats } from "@/types/dashboard";

interface Props {
    connections: ConnectionStats | null;
}

const props = defineProps<Props>();

// Sample data for visualization
interface ClientNode {
    x: number;
    y: number;
    active: boolean;
}

interface Connection {
    id: string;
    ip: string;
    active: boolean;
    duration: number;
}

// State
const clientNodes = ref<ClientNode[]>([]);
const recentConnections = ref<Connection[]>([]);
const animationInterval = ref<ReturnType<typeof setInterval> | null>(null);

// Computed
const activeConnections = computed(() => props.connections?.active ?? 0);
const totalConnections = computed(() => props.connections?.total_lifetime ?? 0);
const connectionRate = computed(() => {
    // Approximate rate based on total lifetime connections (would need real data)
    return ((props.connections?.total_lifetime ?? 0) / 60).toFixed(1);
});
const avgDuration = computed(() => (props.connections?.avg_duration_ms ?? 0).toFixed(0));
const closedConnections = computed(() =>
    Math.max(0, (props.connections?.total_lifetime ?? 0) - (props.connections?.active ?? 0)),
);
const failedConnections = computed(() => props.connections?.failed ?? 0);

// Initialize client nodes in a circle
onMounted(() => {
    const numClients = Math.min(activeConnections.value, 8);
    const radius = 100;

    clientNodes.value = Array.from({ length: numClients }, (_, idx) => {
        const angle = (idx / numClients) * Math.PI * 2;
        return {
            x: 200 + radius * Math.cos(angle),
            y: 150 + radius * Math.sin(angle),
            active: Math.random() > 0.3, // 70% chance active
        };
    });

    // Mock recent connections
    recentConnections.value = Array.from({ length: 5 }, (_, idx) => ({
        id: `conn-${idx}`,
        ip: `192.168.1.${100 + idx}`,
        active: idx < 2,
        duration: 1000 + idx * 500,
    }));

    // Animate connections
    animationInterval.value = setInterval(() => {
        clientNodes.value = clientNodes.value.map((node) => ({
            ...node,
            active: Math.random() > 0.4,
        }));

        recentConnections.value = recentConnections.value.map((conn) => ({
            ...conn,
            duration: conn.duration + (Math.random() * 100 - 50),
        }));
    }, 1000);
});

onBeforeUnmount(() => {
    if (animationInterval.value) {
        clearInterval(animationInterval.value);
    }
});

// Methods
const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${Math.round(ms)}ms`;
    const seconds = (ms / 1000).toFixed(1);
    return `${seconds}s`;
};
</script>

<style scoped>
.connection-graph-container {
    padding: 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(26, 26, 46, 0.5);
    display: flex;
    flex-direction: column;
    gap: 16px;
    color: #ffffff;
}

/* Header */
.graph-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.graph-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #ffffff;
}

.header-stats {
    display: flex;
    gap: 12px;
}

.stat-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    background: rgba(0, 212, 255, 0.1);
    border: 1px solid rgba(0, 212, 255, 0.2);
    border-radius: 4px;
}

.stat-label {
    font-size: 10px;
    color: #606060;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.3px;
}

.stat-value {
    font-size: 14px;
    color: #00d4ff;
    font-weight: 700;
    font-family: "Fira Code", monospace;
}

/* Graph Content */
.graph-content {
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(10, 10, 15, 0.5);
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 12px;
    min-height: 320px;
}

.connection-svg {
    width: 100%;
    height: 100%;
    max-width: 400px;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.server-node {
    transition: all 0.3s ease;
    cursor: pointer;
}

.server-node:hover {
    r: 35;
    filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.4));
}

.client-node {
    transition: all 0.3s ease;
    cursor: pointer;
}

.client-node:hover {
    r: 18;
}

.connection-line {
    transition: stroke-width 0.3s ease;
}

.activity-pulse {
    animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
    0% {
        r: 20;
        opacity: 0.5;
    }
    50% {
        r: 25;
        opacity: 0.2;
    }
    100% {
        r: 20;
        opacity: 0.5;
    }
}

/* Connection Details */
.connection-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 12px;
    padding: 12px;
    background: rgba(0, 212, 255, 0.05);
    border-radius: 6px;
    border: 1px solid rgba(0, 212, 255, 0.1);
}

.detail-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.detail-label {
    font-size: 10px;
    color: #606060;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.3px;
}

.detail-value {
    font-size: 16px;
    color: #00d4ff;
    font-weight: 700;
    font-family: "Fira Code", monospace;
}

.detail-value.is-warning {
    color: #ffb800;
}

/* Connection List */
.connection-list {
    padding: 12px;
    background: rgba(0, 212, 255, 0.05);
    border-radius: 6px;
    border: 1px solid rgba(0, 212, 255, 0.1);
}

.list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 12px;
    color: #a0a0a0;
}

.list-title {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.list-count {
    font-size: 11px;
    color: #606060;
    background: rgba(255, 255, 255, 0.05);
    padding: 2px 8px;
    border-radius: 3px;
}

.list-items {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.list-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-radius: 4px;
    background: rgba(160, 160, 160, 0.05);
    border: 1px solid rgba(160, 160, 160, 0.1);
    font-size: 12px;
    transition: all 0.2s ease;
}

.list-item.is-active {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.2);
}

.list-item:hover {
    background: rgba(0, 212, 255, 0.15);
    border-color: rgba(0, 212, 255, 0.3);
}

.connection-status {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #606060;
    flex-shrink: 0;
}

.connection-status.is-active {
    background: #00d4ff;
    box-shadow: 0 0 4px rgba(0, 212, 255, 0.5);
}

.connection-ip {
    flex: 1;
    color: #ffffff;
    font-family: "Fira Code", monospace;
    font-weight: 500;
}

.connection-duration {
    color: #a0a0a0;
    font-family: "Fira Code", monospace;
    font-size: 11px;
}

/* Responsive */
@media (max-width: 768px) {
    .connection-graph-container {
        padding: 16px;
    }

    .graph-content {
        min-height: 250px;
    }

    .connection-details {
        grid-template-columns: repeat(2, 1fr);
    }

    .graph-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
    }

    .header-stats {
        width: 100%;
        justify-content: space-between;
    }
}
</style>
