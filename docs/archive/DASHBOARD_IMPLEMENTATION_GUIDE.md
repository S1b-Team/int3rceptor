# 🚀 Dashboard Implementation Guide

**Status**: Production-Ready Implementation Blueprint  
**Target**: High-performance, real-time metrics visualization  
**Estimated Time**: 40-60 hours of development  
**Last Updated**: 2025-01-20

---

## 📋 Quick Start Checklist

### Phase 1: Foundation (8 hours)
- [ ] Create base types in `src/types/dashboard.ts`
- [ ] Implement `useDashboardMetrics` composable
- [ ] Implement `useDashboardWebSocket` composable
- [ ] Setup CSS variables and theme tokens
- [ ] Create utility formatters (`src/utils/dashboard/formatters.ts`)

### Phase 2: Core Components (16 hours)
- [ ] `MetricCard.vue` - Reusable metric display
- [ ] `MetricsGrid.vue` - Responsive grid container
- [ ] `DashboardHeader.vue` - Status and time display
- [ ] `QuickActionsBar.vue` - Control buttons
- [ ] `StatusBadge.vue` - Status indicator component

### Phase 3: Advanced Components (16 hours)
- [ ] `ActivityChart.vue` - Time-series line chart
- [ ] `RecentActivityPanel.vue` - Activity feed
- [ ] `SystemHealthPanel.vue` - Health gauges
- [ ] `ProgressRing.vue` - Circular progress indicator
- [ ] `ConnectionGraph.vue` - Connection visualization

### Phase 4: Integration & Polish (8-12 hours)
- [ ] Update `DashboardTab.vue` with all components
- [ ] Performance optimization and testing
- [ ] Accessibility audit (WCAG 2.1 AA)
- [ ] Cross-browser testing
- [ ] Deploy and monitor

---

## 🏗️ Component Implementation Order

### 1. Base Types (`src/types/dashboard.ts`)

**Location**: `s1b-ecosystem/int3rceptor/ui/src/types/dashboard.ts`

```typescript
// Core metric types
export interface SystemMetrics {
  timestamp: number;
  requests_per_sec: number;
  avg_response_time_ms: number;
  memory_usage_mb: number;
  active_connections: number;
  bytes_in_sec: number;
  bytes_out_sec: number;
  error_rate_percent: number;
  cpu_percent: number;
  disk_percent: number;
  uptime_seconds: number;
}

export interface MetricPoint {
  timestamp: number;
  value: number;
  label?: string;
}

export interface TimeSeriesData {
  metric: string;
  unit: string;
  dataPoints: MetricPoint[];
  current: number;
  min: number;
  max: number;
  avg: number;
  percentile_95: number;
  percentile_99: number;
}

// Health status
export type HealthLevel = 'healthy' | 'warning' | 'critical';

export interface HealthStatus {
  overall: HealthLevel;
  cpu: HealthLevel;
  memory: HealthLevel;
  disk: HealthLevel;
  network: HealthLevel;
  database: HealthLevel;
  uptime: number;
  lastCheck: number;
}

// Activity & events
export interface ActivityEntry {
  id: string;
  timestamp: number;
  type: 'request' | 'error' | 'warning' | 'info';
  message: string;
  details?: Record<string, any>;
}

// Connection statistics
export interface ConnectionStats {
  active: number;
  established: number;
  closing: number;
  failed: number;
  total_lifetime: number;
  avg_duration_ms: number;
}

// Proxy status
export interface ProxyStatus {
  running: boolean;
  host: string;
  port: number;
  tls_enabled: boolean;
  start_time: number;
}

// Dashboard state
export interface DashboardState {
  metrics: SystemMetrics;
  connectionStats: ConnectionStats;
  recentActivity: ActivityEntry[];
  systemHealth: HealthStatus;
  proxyStatus: ProxyStatus;
  lastUpdated: number;
  isLoading: boolean;
  error: string | null;
}
```

---

### 2. Composables

#### 2a. `useDashboardMetrics()`

**Location**: `src/composables/dashboard/useDashboardMetrics.ts`

```typescript
import { ref, computed, onBeforeUnmount } from 'vue';
import { api } from '@/composables/useApi';
import type { SystemMetrics } from '@/types/dashboard';

const METRICS_ENDPOINT = '/api/dashboard/metrics';
const POLL_INTERVAL = 1000; // 1 second

export function useDashboardMetrics() {
  const metrics = ref<SystemMetrics | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const lastUpdate = ref<number>(0);
  
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  const fetchMetrics = async () => {
    isLoading.value = true;
    error.value = null;
    
    try {
      const response = await api.get<SystemMetrics>(METRICS_ENDPOINT);
      metrics.value = response.data;
      lastUpdate.value = Date.now();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch metrics';
      console.error('Metrics fetch error:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const startPolling = (interval: number = POLL_INTERVAL) => {
    // Fetch immediately
    fetchMetrics();
    
    // Then poll at interval
    if (pollInterval) clearInterval(pollInterval);
    pollInterval = setInterval(fetchMetrics, interval);
  };

  const stopPolling = () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  };

  const timeSinceUpdate = computed(() => {
    return lastUpdate.value ? Date.now() - lastUpdate.value : 0;
  });

  onBeforeUnmount(() => {
    stopPolling();
  });

  return {
    metrics,
    isLoading,
    error,
    lastUpdate,
    timeSinceUpdate,
    fetchMetrics,
    startPolling,
    stopPolling,
  };
}
```

#### 2b. `useDashboardWebSocket()`

**Location**: `src/composables/dashboard/useDashboardWebSocket.ts`

```typescript
import { ref, onMounted, onBeforeUnmount } from 'vue';
import type { SystemMetrics, ActivityEntry, HealthStatus, DashboardEvent } from '@/types/dashboard';

const WS_ENDPOINT = import.meta.env.VITE_WS_URL ?? 'ws://127.0.0.1:3000/ws';

export function useDashboardWebSocket() {
  const isConnected = ref(false);
  const lastEvent = ref<DashboardEvent | null>(null);
  const reconnectAttempts = ref(0);
  const maxReconnectAttempts = 5;
  
  let ws: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

  const connect = () => {
    if (ws?.readyState === WebSocket.OPEN) return;

    try {
      ws = new WebSocket(`${WS_ENDPOINT}?channel=dashboard`);

      ws.onopen = () => {
        isConnected.value = true;
        reconnectAttempts.value = 0;
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as DashboardEvent;
          lastEvent.value = data;
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };

      ws.onerror = (err) => {
        console.error('WebSocket error:', err);
      };

      ws.onclose = () => {
        isConnected.value = false;
        attemptReconnect();
      };
    } catch (err) {
      console.error('Failed to connect WebSocket:', err);
      attemptReconnect();
    }
  };

  const attemptReconnect = () => {
    if (reconnectAttempts.value >= maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    reconnectAttempts.value++;
    const delay = Math.pow(2, reconnectAttempts.value) * 1000;
    
    if (reconnectTimeout) clearTimeout(reconnectTimeout);
    reconnectTimeout = setTimeout(connect, delay);
  };

  const disconnect = () => {
    if (reconnectTimeout) clearTimeout(reconnectTimeout);
    if (ws) {
      ws.close();
      ws = null;
    }
    isConnected.value = false;
  };

  onMounted(() => {
    connect();
  });

  onBeforeUnmount(() => {
    disconnect();
  });

  return {
    isConnected,
    lastEvent,
    connect,
    disconnect,
  };
}
```

---

### 3. Utility Functions

#### 3a. Formatters (`src/utils/dashboard/formatters.ts`)

```typescript
/**
 * Format large numbers with appropriate units
 * 1000 → "1K", 1000000 → "1M", etc.
 */
export function formatNumber(num: number, decimals = 1): string {
  if (num === 0) return '0';
  
  const units = ['', 'K', 'M', 'B', 'T'];
  const exponent = Math.floor(Math.log10(Math.abs(num)) / 3);
  
  if (exponent === 0) return num.toFixed(0);
  
  const value = num / Math.pow(10, exponent * 3);
  return `${value.toFixed(decimals)}${units[exponent]}`;
}

/**
 * Format bytes to KB/MB/GB
 */
export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return '0 B';
  
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const exponent = Math.floor(Math.log10(bytes) / Math.log10(1024));
  
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(decimals)} ${units[exponent]}`;
}

/**
 * Format milliseconds to human-readable time
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  
  return `${(ms / 3600000).toFixed(1)}h`;
}

/**
 * Format uptime in seconds to human-readable format
 */
export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  const parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  
  return parts.join(' ') || '<1m';
}

/**
 * Format percentage with appropriate decimals
 */
export function formatPercent(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format timestamp to readable time
 */
export function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

/**
 * Get CSS class based on health level
 */
export function getHealthClass(level: 'healthy' | 'warning' | 'critical'): string {
  const classMap = {
    healthy: 'status-healthy',
    warning: 'status-warning',
    critical: 'status-critical',
  };
  return classMap[level];
}

/**
 * Get color based on health level
 */
export function getHealthColor(level: 'healthy' | 'warning' | 'critical'): string {
  const colorMap = {
    healthy: '#00d4ff',   // Cyan
    warning: '#ffb800',   // Orange
    critical: '#ff006e',  // Magenta
  };
  return colorMap[level];
}
```

---

### 4. Base Components

#### 4a. MetricCard.vue

**Location**: `src/components/dashboard/MetricCard.vue`

```vue
<template>
  <div class="metric-card" :class="statusClass">
    <!-- Header with icon and title -->
    <div class="card-header">
      <span v-if="icon" class="icon">{{ icon }}</span>
      <h3 class="title">{{ title }}</h3>
      <span v-if="trend" class="trend" :class="trend.direction">
        <span class="arrow">{{ trend.direction === 'up' ? '↑' : '↓' }}</span>
        {{ formatPercent(trend.percent) }}
      </span>
    </div>

    <!-- Main metric value -->
    <div class="card-value">
      <span class="value">{{ formatMetricValue(value) }}</span>
      <span class="unit">{{ unit }}</span>
    </div>

    <!-- Optional sparkline chart -->
    <div v-if="sparklineData && sparklineData.length > 0" class="sparkline">
      <svg :viewBox="`0 0 ${sparklineData.length * 4} 40`" preserveAspectRatio="none">
        <polyline :points="sparklinePoints" />
      </svg>
    </div>

    <!-- Status bar for threshold-based metrics -->
    <div v-if="threshold" class="status-bar">
      <div class="bar-fill" :style="{ width: fillPercentage + '%' }"></div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-overlay">
      <div class="spinner"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { formatNumber, formatPercent } from '@/utils/dashboard/formatters';
import type { MetricPoint } from '@/types/dashboard';

interface Props {
  title: string;
  value: number;
  unit: string;
  icon?: string;
  trend?: {
    direction: 'up' | 'down' | 'stable';
    percent: number;
    timeWindow?: string;
  };
  threshold?: {
    warning: number;
    critical: number;
    max: number;
  };
  sparklineData?: MetricPoint[];
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  icon: '',
  loading: false,
});

// Determine status based on value and thresholds
const statusClass = computed(() => {
  if (!props.threshold) return 'status-normal';
  
  const { value } = props;
  const { critical, warning } = props.threshold;
  
  if (value >= critical) return 'status-critical';
  if (value >= warning) return 'status-warning';
  return 'status-healthy';
});

// Calculate fill percentage for progress bar
const fillPercentage = computed(() => {
  if (!props.threshold) return 0;
  return Math.min((props.value / props.threshold.max) * 100, 100);
});

// Generate sparkline SVG points
const sparklinePoints = computed(() => {
  if (!props.sparklineData || props.sparklineData.length === 0) return '';
  
  const data = props.sparklineData;
  const min = Math.min(...data.map(d => d.value));
  const max = Math.max(...data.map(d => d.value));
  const range = max - min || 1;
  
  return data
    .map((point, i) => {
      const x = i * 4;
      const y = 40 - ((point.value - min) / range) * 40;
      return `${x},${y}`;
    })
    .join(' ');
});

const formatMetricValue = (val: number): string => {
  // Format based on value magnitude
  if (Math.abs(val) >= 1000) return formatNumber(val, 1);
  if (Number.isInteger(val)) return val.toString();
  return val.toFixed(2);
};
</script>

<style scoped>
.metric-card {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 8px;
  padding: 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.metric-card:hover {
  border-color: rgba(0, 212, 255, 0.6);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.15);
  transform: translateY(-2px);
}

.metric-card.status-warning {
  border-color: rgba(255, 184, 0, 0.4);
}

.metric-card.status-warning:hover {
  border-color: rgba(255, 184, 0, 0.6);
  box-shadow: 0 0 20px rgba(255, 184, 0, 0.15);
}

.metric-card.status-critical {
  border-color: rgba(255, 0, 110, 0.4);
}

.metric-card.status-critical:hover {
  border-color: rgba(255, 0, 110, 0.6);
  box-shadow: 0 0 20px rgba(255, 0, 110, 0.15);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.icon {
  font-size: 20px;
  line-height: 1;
}

.title {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #a0a0a0;
  flex: 1;
}

.trend {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
}

.trend.up {
  color: #00d4ff;
  background: rgba(0, 212, 255, 0.1);
}

.trend.down {
  color: #ff006e;
  background: rgba(255, 0, 110, 0.1);
}

.trend.stable {
  color: #ffb800;
  background: rgba(255, 184, 0, 0.1);
}

.arrow {
  font-weight: 700;
}

.card-value {
  margin-bottom: 16px;
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.value {
  font-size: 32px;
  font-weight: 700;
  color: #ffffff;
  font-family: 'Fira Code', monospace;
  letter-spacing: -1px;
}

.unit {
  font-size: 13px;
  color: #606060;
  font-weight: 500;
}

.sparkline {
  height: 40px;
  margin-bottom: 12px;
  opacity: 0.6;
}

.sparkline svg {
  width: 100%;
  height: 100%;
}

.sparkline polyline {
  fill: none;
  stroke: #00d4ff;
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.status-bar {
  height: 4px;
  background: rgba(0, 212, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
  margin-top: 8px;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #00d4ff, #8b5cf6);
  border-radius: 2px;
  transition: width 0.3s ease-out;
}

.metric-card.status-warning .bar-fill {
  background: linear-gradient(90deg, #ffb800, #ff9900);
}

.metric-card.status-critical .bar-fill {
  background: linear-gradient(90deg, #ff006e, #ff4500);
}

.loading-overlay {
  position: absolute;
  inset: 0;
  background: rgba(10, 10, 15, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  backdrop-filter: blur(2px);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(0, 212, 255, 0.2);
  border-top-color: #00d4ff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

---

#### 4b. MetricsGrid.vue

**Location**: `src/components/dashboard/MetricsGrid.vue`

```vue
<template>
  <div class="metrics-grid">
    <MetricCard
      v-for="metric in metricsList"
      :key="metric.id"
      :title="metric.title"
      :value="metric.value"
      :unit="metric.unit"
      :icon="metric.icon"
      :trend="metric.trend"
      :threshold="metric.threshold"
      :sparkline-data="metric.sparklineData"
      :loading="isLoading"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import MetricCard from './MetricCard.vue';
import type { SystemMetrics, MetricPoint } from '@/types/dashboard';

interface Props {
  metrics: SystemMetrics | null;
  timeSeries?: Map<string, MetricPoint[]>;
  isLoading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  timeSeries: () => new Map(),
  isLoading: false,
});

const metricsList = computed(() => {
  if (!props.metrics) return [];

  return [
    {
      id: 'requests-per-sec',
      title: 'Requests/sec',
      value: props.metrics.requests_per_sec,
      unit: '/sec',
      icon: '📊',
      trend: {
        direction: 'up' as const,
        percent: 12,
        timeWindow: '5m',
      },
      threshold: {
        warning: 5000,
        critical: 10000,
        max: 15000,
      },
      sparklineData: props.timeSeries.get('requests_per_sec'),
    },
    {
      id: 'avg-response-time',
      title: 'Avg Response Time',
      value: props.metrics.avg_response_time_ms,
      unit: 'ms',
      icon: '⏱️',
      trend: {
        direction: 'down' as const,
        percent: 5,
        timeWindow: '5m',
      },
      threshold: {
        warning: 500,
        critical: 1000,
        max: 2000,
      },
      sparklineData: props.timeSeries.get('response_time'),
    },
    {
      id: 'memory-usage',
      title: 'Memory Usage',
      value: props.metrics.memory_usage_mb,
      unit: 'MB',
      icon: '🧠',
      threshold: {
        warning: 300,
        critical: 400,
        max: 512,
      },
      sparklineData: props.timeSeries.get('memory_usage'),
    },
    {
      id: 'active-connections',
      title: 'Active Connections',
      value: props.metrics.active_connections,
      unit: '',
      icon: '🔗',
      threshold: {
        warning: 100,
        critical: 150,
        max: 200,
      },
    },
    {
      id: 'bytes-transferred',
      title: 'Bytes/sec',
      value: props.metrics.bytes_in_sec + props.metrics.bytes_out_sec,
      unit: 'B/s',
      icon: '📡',
      trend: {
        direction: 'up' as const,
        percent: 8,
        timeWindow: '5m',
      },
    },
    {
      id: 'error-rate',
      title: 'Error Rate',
      value: props.metrics.error_rate_percent,
      unit: '%',
      icon: '⚠️',
      trend: {
        direction: 'stable' as const,
        percent: 0,
      },
      threshold: {
        warning: 1,
        critical: 5,
        max: 100,
      },
    },
  ];
});
</script>

<style scoped>
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
  padding: 20px;
  animation: fadeIn 0.3s ease-out;
}

@media (max-width: 1366px) {
  .metrics-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .metrics-grid {
    grid-template-columns: 1fr;
    padding: 12px;
    gap: 12px;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
```

---

## 📊 Complete DashboardTab Implementation

**Location**: `src/components/DashboardTab.vue`

```vue
<template>
  <div class="dashboard-tab">
    <!-- Header with status and controls -->
    <DashboardHeader
      :proxy-status="proxyStatus"
      :metrics-health="systemHealth.overall"
      :last-update="lastUpdate"
      :ws-connected="wsConnected"
      @toggle-proxy="toggleProxy"
      @export="exportMetrics"
    />

    <!-- Error alert -->
    <div v-if="error" class="error-alert">
      <span class="error-icon">⚠️</span>
      <span class="error-message">{{ error }}</span>
      <button @click="dismissError" class="close-btn">✕</button>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading && !metrics" class="loading-state">
      <div class="spinner"></div>
      <p>Loading dashboard metrics...</p>
    </div>

    <!-- Main content -->
    <template v-else-if="metrics">
      <!-- Metrics Grid -->
      <MetricsGrid
        :metrics="metrics"
        :time-series="timeSeries"
        :is-loading="isLoading"
      />

      <!-- Activity and Health -->
      <div class="dashboard-panels">
        <RecentActivityPanel
          :activities="recentActivity"
          :top-endpoints="topEndpoints"
          class="panel"
        />
        <SystemHealthPanel
          :health="systemHealth"
          :connection-stats="connectionStats"
          class="panel"
        />
      </div>

      <!-- Quick Actions -->
      <QuickActionsBar
        :proxy-status="proxyStatus"
        :can-clear="metrics.total_requests > 0"
        @toggle-proxy="toggleProxy"
        @clear-traffic="clearTraffic"
        @export="exportMetrics"
      />
    </template>

    <!-- Empty state -->
    <div v-else class="empty-state">
      <p>No metrics available</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useDashboardMetrics } from '@/composables/dashboard/useDashboardMetrics';
import { useDashboardWebSocket } from '@/composables/dashboard/useDashboardWebSocket';
import { useApi } from '@/composables/useApi';
import DashboardHeader from './dashboard/DashboardHeader.vue';
import MetricsGrid from './dashboard/MetricsGrid.vue';
import RecentActivityPanel from './dashboard/RecentActivityPanel.vue';
import SystemHealthPanel from './dashboard/SystemHealthPanel.vue';
import QuickActionsBar from './dashboard/QuickActionsBar.vue';
import type {
  SystemMetrics,
  ConnectionStats,
  ActivityEntry,
  HealthStatus,
  ProxyStatus,
  MetricPoint,
} from '@/types/dashboard';

// Composables
const { metrics, isLoading, error, startPolling, stopPolling } =
  useDashboardMetrics();
const { isConnected: wsConnected, lastEvent } = useDashboardWebSocket();

// API
const { api } = useApi();

// State
const timeSeries = ref<Map<string, MetricPoint[]>>(new Map());
const recentActivity = ref<ActivityEntry[]>([]);
const topEndpoints = ref<any[]>([]);
const connectionStats = ref<ConnectionStats>({
  active: 0,
  established: 0,
  closing: 0,
  failed: 0,
  total_lifetime: 0,
  avg_duration_ms: 0,
});
const systemHealth = ref<HealthStatus>({
  overall: 'healthy',
  cpu: 'healthy',
  memory: 'healthy',
  disk: 'healthy',
  network: 'healthy',
  database: 'healthy',
  uptime: 0,
  lastCheck: 0,
});
const proxyStatus = ref<ProxyStatus>({
  running: false,
  host: 'localhost',
  port: 8080,
  tls_enabled: false,
  start_time: 0,
});

const lastUpdate = computed(() => Date.now());
const dismissError = () => (error.value = null);

// Load additional dashboard data
const loadDashboardData = async () => {
  try {
    const [health, activity, endpoints, proxy] = await Promise.all([
      api.get('/api/dashboard/health'),
      api.get('/api/dashboard/activity?limit=10'),
      api.get('/api/dashboard/endpoints?limit=5'),
      api.get('/api/proxy/status'),
    ]);

    systemHealth.value = health.data;
