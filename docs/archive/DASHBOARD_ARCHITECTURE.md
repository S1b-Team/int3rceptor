# 🎯 Dashboard Tab Architecture Proposal

**Document Version**: 1.0  
**Status**: Architecture Design  
**Target**: Production-ready implementation  
**Last Updated**: 2025-01-20

---

## 📋 Table of Contents

1. [Executive Summary](#executive-summary)
2. [Component Architecture](#component-architecture)
3. [Data Model & Types](#data-model--types)
4. [Component Specifications](#component-specifications)
5. [Integration Points](#integration-points)
6. [Performance Optimization](#performance-optimization)
7. [Security Considerations](#security-considerations)
8. [Testing Strategy](#testing-strategy)

---

## Executive Summary

The Dashboard tab serves as the **central monitoring hub** for Int3rceptor, providing real-time metrics, system health indicators, and quick-access controls. The architecture emphasizes:

- **Real-time Updates**: WebSocket-driven metrics with <50ms latency
- **Modular Design**: Reusable card components for different metric types
- **Performance**: Efficient rendering with minimal re-renders
- **Accessibility**: WCAG 2.1 AA compliance
- **Responsiveness**: Adapts gracefully from 1280px → 3440px displays

---

## Component Architecture

### High-Level Structure

```
DashboardTab (Main Container)
├── DashboardHeader
│   ├── ConnectionStatus
│   └── TimeSeriesMetrics
├── MetricsGrid
│   ├── MetricCard (Requests/sec)
│   ├── MetricCard (Avg Response Time)
│   ├── MetricCard (Memory Usage)
│   ├── MetricCard (Active Connections)
│   ├── MetricCard (Bytes In/Out)
│   └── MetricCard (Error Rate)
├── RecentActivityPanel
│   ├── ActivityTimeline
│   └── TopEndpoints
├── SystemHealthPanel
│   ├── HealthGauge (CPU)
│   ├── HealthGauge (Memory)
│   ├── HealthGauge (Disk)
│   └── ConnectionGraph
└── QuickActionsBar
    ├── ProxyControlButton
    ├── ExportButton
    └── SettingsButton
```

### Component Tree Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      DashboardTab (Main)                         │
│  • Manages dashboard state & WebSocket connection                │
│  • Handles metric subscriptions & unsubscriptions                │
│  • Coordinates data updates across all child components          │
└──────────────┬──────────────────────────────────────────────────┘
               │
       ┌───────┴───────┐
       │               │
       ▼               ▼
┌────────────────┐ ┌─────────────────────────────────┐
│ DashboardHeader│ │ MetricsGrid (Responsive)        │
│                │ │                                 │
│ • Status pills │ │ ┌──────────┐ ┌──────────┐      │
│ • Time sync    │ │ │MetricCard│ │MetricCard│ ...  │
│ • Quick links  │ │ │          │ │          │      │
│                │ │ └──────────┘ └──────────┘      │
└────────────────┘ │                                 │
                   │ GridItem: auto-fit columns      │
                   │ Gap: 16px | Padding: 20px       │
                   └─────────────────────────────────┘
       ┌───────────┬──────────────┬────────────────┐
       ▼           ▼              ▼                ▼
    ┌──────┐  ┌────────┐  ┌──────────┐  ┌──────────┐
    │Recent│  │System  │  │ Activity │  │ Quick    │
    │Activity  │Health  │  │ Export   │  │ Actions  │
    └──────┘  └────────┘  └──────────┘  └──────────┘
```

### Module Organization

```
src/
├── components/
│   ├── DashboardTab.vue                 # Main container (150 lines)
│   ├── dashboard/
│   │   ├── DashboardHeader.vue           # Top section (120 lines)
│   │   ├── MetricsGrid.vue               # Responsive grid (80 lines)
│   │   ├── MetricCard.vue                # Reusable card (90 lines)
│   │   ├── RecentActivityPanel.vue       # Activity feed (120 lines)
│   │   ├── SystemHealthPanel.vue         # Health gauges (140 lines)
│   │   └── QuickActionsBar.vue           # Control buttons (100 lines)
│   │
│   └── shared/
│       ├── TimeSeriesChart.vue           # Line/area chart (130 lines)
│       ├── ConnectionGraph.vue           # Connection visualizer (110 lines)
│       ├── ActivityTimeline.vue          # Timeline view (100 lines)
│       ├── StatusBadge.vue               # Status indicator (60 lines)
│       └── ProgressRing.vue              # Circular progress (80 lines)
│
├── composables/
│   ├── useDashboardMetrics.ts            # Metrics aggregation
│   ├── useDashboardWebSocket.ts          # WebSocket management
│   ├── useMetricUpdates.ts               # Update batching
│   └── useSystemHealth.ts                # Health calculations
│
├── utils/
│   ├── formatters.ts                     # Number/time formatting
│   ├── metrics.ts                        # Metric calculations
│   ├── chart-generators.ts               # Chart data factories
│   └── thresholds.ts                     # Health thresholds
│
└── types/
    └── dashboard.ts                      # Dashboard-specific types
```

---

## Data Model & Types

### Dashboard Types

```typescript
// src/types/dashboard.ts

// ==========================================
// REAL-TIME METRICS
// ==========================================

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

// ==========================================
// DASHBOARD STATE
// ==========================================

export interface DashboardState {
  metrics: SystemMetrics;
  timeSeries: {
    requestsPerSec: TimeSeriesData;
    responseTime: TimeSeriesData;
    memoryUsage: TimeSeriesData;
    bytesTransferred: TimeSeriesData;
  };
  connectionStats: ConnectionStats;
  recentActivity: ActivityEntry[];
  systemHealth: HealthStatus;
  proxyStatus: ProxyStatus;
  lastUpdated: number;
  isLoading: boolean;
  error: string | null;
}

// ==========================================
// DASHBOARD METRICS DETAILS
// ==========================================

export interface MetricCardProps {
  title: string;
  value: number;
  unit: string;
  icon: string;
  trend?: {
    direction: 'up' | 'down' | 'stable';
    percent: number;
    timeWindow: string; // e.g., "5m", "1h"
  };
  threshold?: {
    warning: number;
    critical: number;
    current: number;
  };
  sparklineData?: MetricPoint[];
  onClick?: () => void;
  loading?: boolean;
}

// ==========================================
// CONNECTION STATISTICS
// ==========================================

export interface ConnectionStats {
  active: number;
  established: number;
  closing: number;
  failed: number;
  total_lifetime: number;
  avg_duration_ms: number;
  peak_connections: number;
  peak_time: number;
  concurrent_limit: number;
}

export interface ConnectionBreakdown {
  protocol: {
    http: number;
    https: number;
    websocket: number;
  };
  state: {
    connecting: number;
    open: number;
    closing: number;
    closed: number;
  };
}

// ==========================================
// ACTIVITY & EVENTS
// ==========================================

export interface ActivityEntry {
  id: string;
  timestamp: number;
  type: 'request' | 'error' | 'warning' | 'info';
  message: string;
  details?: Record<string, any>;
  relatedId?: string;
}

export interface TopEndpoint {
  url: string;
  host: string;
  requests_count: number;
  total_bytes: number;
  avg_response_time: number;
  error_count: number;
  last_seen: number;
}

// ==========================================
// HEALTH STATUS
// ==========================================

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

export interface HealthGaugeData {
  label: string;
  current: number;
  max: number;
  unit: string;
  status: HealthLevel;
  trend?: number; // percent change
}

// ==========================================
// PROXY CONTROL
// ==========================================

export interface ProxyStatus {
  running: boolean;
  host: string;
  port: number;
  tls_enabled: boolean;
  intercept_https: boolean;
  start_time: number;
  certificates_generated: number;
}

export interface ProxyConfig {
  port: number;
  host: string;
  tls_enabled: boolean;
  intercept_https: boolean;
  max_connections: number;
  request_timeout_ms: number;
  idle_timeout_ms: number;
}

// ==========================================
// DASHBOARD EVENTS
// ==========================================

export type DashboardEvent =
  | { type: 'metrics_update'; payload: SystemMetrics }
  | { type: 'connection_change'; payload: ConnectionStats }
  | { type: 'activity_log'; payload: ActivityEntry }
  | { type: 'health_change'; payload: HealthStatus }
  | { type: 'proxy_status'; payload: ProxyStatus }
  | { type: 'error'; payload: { code: string; message: string } };

export interface MetricsUpdate {
  id: string;
  timestamp: number;
  changes: Partial<SystemMetrics>;
  source: 'polling' | 'websocket' | 'manual';
}
```

---

## Component Specifications

### 1. DashboardTab (Main Container)

**Purpose**: Root component managing all dashboard state and WebSocket subscriptions

**Props**: None

**Emits**: None

**State**:
- `metrics: SystemMetrics` - Current system metrics
- `timeSeries: Map<string, TimeSeriesData>` - Historical metric data
- `wsConnected: boolean` - WebSocket connection status
- `autoRefreshEnabled: boolean` - Auto-update toggle
- `refreshInterval: number` - Update interval (default: 1000ms)

**Lifecycle**:
```
onMounted():
  1. Connect WebSocket (/ws/dashboard)
  2. Fetch initial metrics (/api/dashboard/metrics)
  3. Subscribe to metric channels
  4. Start auto-refresh interval (1s)

onUnmounted():
  1. Disconnect WebSocket
  2. Clear intervals
  3. Unsubscribe from all channels
```

**Key Methods**:
```typescript
// Subscribe to metric stream
subscribeToMetrics(channels: string[]): void

// Handle WebSocket messages
handleMetricUpdate(event: DashboardEvent): void

// Batch updates for performance
batchMetricUpdates(updates: MetricsUpdate[]): void

// Export metrics as JSON/CSV
exportMetrics(format: 'json' | 'csv'): Promise<Blob>

// Pause/Resume auto-refresh
toggleAutoRefresh(): void
```

**Size**: ~150 lines

---

### 2. DashboardHeader

**Purpose**: Display connection status, time sync, and quick navigation

**Props**:
```typescript
interface Props {
  proxyStatus: ProxyStatus;
  metricsHealth: HealthLevel;
  lastUpdate: number;
  wsConnected: boolean;
}
```

**Key Features**:
- Status pills (Connected/Disconnected/Waiting)
- Server time display (synced via WebSocket)
- Quick links to other tabs
- Update timestamp indicator
- Proxy on/off toggle

**Template Structure**:
```vue
<div class="dashboard-header">
  <!-- Status Bar -->
  <div class="status-bar">
    <StatusBadge :status="metricsHealth" label="System Health" />
    <StatusBadge :status="proxyStatus.running ? 'healthy' : 'warning'" 
                 label="Proxy" />
    <StatusBadge :status="wsConnected ? 'healthy' : 'critical'" 
                 label="WebSocket" />
  </div>
  
  <!-- Time Sync -->
  <div class="time-sync">
    <span class="label">Server Time:</span>
    <span class="time">{{ formatTime(lastUpdate) }}</span>
    <span class="indicator" :class="{ synced: isSynced }">
      {{ isSynced ? '✓ Synced' : '⏳ Syncing...' }}
    </span>
  </div>
  
  <!-- Quick Actions -->
  <div class="quick-actions">
    <button @click="toggleProxy" class="btn-proxy">
      {{ proxyStatus.running ? 'Stop' : 'Start' }} Proxy
    </button>
    <button @click="openSettings" class="btn-icon">⚙️</button>
  </div>
</div>
```

**Size**: ~120 lines

---

### 3. MetricsGrid

**Purpose**: Responsive grid layout for metric cards

**Props**:
```typescript
interface Props {
  metrics: SystemMetrics;
  timeSeries: Map<string, TimeSeriesData>;
  isLoading?: boolean;
}
```

**Layout**:
- Desktop (1920px+): 3 columns
- Laptop (1366px): 2 columns
- Tablet (768px): 1 column

**Grid CSS**:
```css
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
  padding: 20px;
  
  @media (max-width: 1366px) {
    grid-template-columns: repeat(2, 1fr);
  }
  
  @media (max-width: 768px) {
    grid-template-columns: 1fr;
  }
}
```

**Cards Displayed**:
1. **Requests/sec** - Current throughput
2. **Avg Response Time** - Request latency
3. **Memory Usage** - Process memory
4. **Active Connections** - Concurrent sessions
5. **Bytes In/Out** - Network traffic
6. **Error Rate** - Error percentage

**Size**: ~80 lines

---

### 4. MetricCard

**Purpose**: Reusable card component for displaying individual metrics

**Props**:
```typescript
interface Props {
  title: string;
  value: number;
  unit: string;
  icon?: string;
  trend?: TrendData;
  threshold?: ThresholdData;
  sparklineData?: MetricPoint[];
  loading?: boolean;
  onClick?: () => void;
}

interface TrendData {
  direction: 'up' | 'down' | 'stable';
  percent: number;
  timeWindow: string;
}

interface ThresholdData {
  warning: number;
  critical: number;
  current: number;
}
```

**Template**:
```vue
<div class="metric-card" :class="statusClass" @click="onClick">
  <!-- Header -->
  <div class="card-header">
    <span class="icon">{{ icon }}</span>
    <h3 class="title">{{ title }}</h3>
    <span v-if="trend" class="trend" :class="trend.direction">
      {{ trend.direction === 'up' ? '↑' : '↓' }} {{ trend.percent }}%
    </span>
  </div>
  
  <!-- Value Display -->
  <div class="card-value">
    <span class="value">{{ formatNumber(value) }}</span>
    <span class="unit">{{ unit }}</span>
  </div>
  
  <!-- Sparkline -->
  <div v-if="sparklineData" class="sparkline">
    <TimeSeriesChart :data="sparklineData" height="40px" />
  </div>
  
  <!-- Status Bar -->
  <div v-if="threshold" class="status-bar">
    <div class="threshold-bar">
      <div class="fill" :style="{ width: percentage }"></div>
    </div>
    <span class="label">{{ threshold.current }} / {{ threshold.max }}</span>
  </div>
</div>
```

**Styling**:
```css
.metric-card {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 8px;
  padding: 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  
  &:hover {
    border-color: rgba(0, 212, 255, 0.6);
    box-shadow: 0 0 20px rgba(0, 212, 255, 0.15);
  }
  
  &.warning {
    border-color: rgba(255, 184, 0, 0.6);
  }
  
  &.critical {
    border-color: rgba(255, 0, 110, 0.6);
  }
}
```

**Size**: ~90 lines

---

### 5. RecentActivityPanel

**Purpose**: Display recent request activity and top endpoints

**Props**:
```typescript
interface Props {
  activities: ActivityEntry[];
  topEndpoints: TopEndpoint[];
  isLoading?: boolean;
}
```

**Sections**:
1. **Activity Feed** - Last 10 activities with timestamps
2. **Top Endpoints** - Most frequently accessed URLs

**Template**:
```vue
<div class="recent-activity-panel">
  <h3 class="panel-title">Recent Activity</h3>
  
  <div class="activity-timeline">
    <ActivityTimeline :items="activities" />
  </div>
  
  <div class="divider"></div>
  
  <h3 class="panel-title">Top Endpoints</h3>
  <div class="endpoints-list">
    <div v-for="endpoint in topEndpoints" 
         :key="endpoint.url" 
         class="endpoint-item">
      <div class="endpoint-info">
        <span class="host">{{ endpoint.host }}</span>
        <span class="url">{{ endpoint.url }}</span>
      </div>
      <div class="endpoint-stats">
        <span class="count">{{ endpoint.requests_count }} reqs</span>
        <span class="time">{{ endpoint.avg_response_time }}ms</span>
      </div>
    </div>
  </div>
</div>
```

**Size**: ~120 lines

---

### 6. SystemHealthPanel

**Purpose**: Display system health status with gauges and indicators

**Props**:
```typescript
interface Props {
  health: HealthStatus;
  connectionStats: ConnectionStats;
  isLoading?: boolean;
}
```

**Gauges**:
- CPU Usage
- Memory Usage
- Disk Usage
- Network Health

**Template**:
```vue
<div class="system-health-panel">
  <h3 class="panel-title">System Health</h3>
  
  <div class="health-grid">
    <ProgressRing v-for="gauge in gauges"
                  :key="gauge.label"
                  :value="gauge.current"
                  :max="gauge.max"
                  :status="gauge.status"
                  :label="gauge.label" />
  </div>
  
  <div class="connections-summary">
    <h4>Connections</h4>
    <ConnectionGraph :stats="connectionStats" />
  </div>
  
  <div class="uptime">
    <span class="label">Uptime:</span>
    <span class="value">{{ formatUptime(health.uptime) }}</span>
  </div>
</div>
```

**Size**: ~140 lines

---

### 7. QuickActionsBar

**Purpose**: Quick access to common operations

**Actions**:
1. **Proxy Control** - Start/Stop proxy
2. **Export Metrics** - Download as JSON/CSV
3. **Clear Cache** - Clear captured requests
4. **Settings** - Open settings panel

**Template**:
```vue
<div class="quick-actions-bar">
  <button @click="toggleProxy" class="btn-primary">
    {{ proxyStatus.running ? 'Stop Proxy' : 'Start Proxy' }}
  </button>
  <button @click="exportMetrics" class="btn-secondary">
    Export
  </button>
  <button @click="clearCache" class="btn-secondary">
    Clear Cache
  </button>
  <button @click="openSettings" class="btn-icon">⚙️</button>
</div>
```

**Size**: ~100 lines

---

## Integration Points

### WebSocket Events

The dashboard subscribes to these WebSocket channels:

```typescript
// 1. Real-time metrics (every 1s)
ws://127.0.0.1:3000/ws?channel=metrics
Payload: SystemMetrics

// 2. Connection events
ws://127.0.0.1:3000/ws?channel=connections
Payload: ConnectionStats

// 3. Activity log
ws://127.0.0.1:3000/ws?channel=activity
Payload: ActivityEntry

// 4. Health updates
ws://127.0.0.1:3000/ws?channel=health
Payload: HealthStatus
```

### REST API Endpoints

```
GET /api/dashboard/metrics
  • Returns: SystemMetrics
  • Cache: 1 second
  • Rate limit: 10/sec

GET /api/dashboard/timeseries?metric=requests_per_sec&window=5m
  • Returns: TimeSeriesData[]
  • Intervals: 5s, 1m, 5m, 15m, 1h

GET /api/dashboard/health
  • Returns: HealthStatus
  • Cache: 5 seconds

GET /api/dashboard/activity?limit=20
  • Returns: ActivityEntry[]
  • Ordered: newest first

GET /api/dashboard/endpoints?limit=10&sort=requests
  • Returns: TopEndpoint[]

POST /api/proxy/start
  • Body: ProxyConfig
  • Returns: ProxyStatus

POST /api/proxy/stop
  • Returns: ProxyStatus

POST /api/dashboard/export?format=json|csv
  • Returns: Blob
  • Content-Type: application/json | text/csv
```

---

## Performance Optimization

### 1. Rendering Optimization

**Virtual Scrolling**: For activity lists > 100 items
```typescript
import { computed } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';

const virtualizer = useVirtualizer({
  count: activities.value.length,
  getScrollElement: () => scrollElementRef.value,
  estimateSize: () => 60,
  overscan: 10,
});
```

**Lazy Loading**: Defer non-critical components
```vue
<template>
  <Suspense>
    <template #default>
      <SystemHealthPanel :health="health" />
    </template>
    <template #fallback>
      <div class="skeleton">Loading health...</div>
    </template>
  </Suspense>
</template>
```

### 2. Update Batching

```typescript
// useDashboardMetrics.ts
const batchSize = 50;
const batchWindow = 100; // ms

function batchMetricUpdates(updates: MetricsUpdate[]) {
  pendingUpdates.push(...updates);
  
  if (!batchTimeout.value) {
    batchTimeout.value = setTimeout(() => {
      const batch = pendingUpdates.splice(0, batchSize);
      applyBatch(batch);
      batchTimeout.value = null;
    }, batchWindow);
  }
}
```

### 3. WebSocket Optimization

```typescript
// useDashboardWebSocket.ts
// Subscribe to only needed channels
const channels = computed(() => {
  return [
    'metrics',
    activePanel.value === 'health' ? 'health' : null,
    activePanel.value === 'activity' ? 'activity' : null,
  ].filter(Boolean);
});

// Unsubscribe from unused channels
watch(channels, (newChannels, oldChannels) => {
  const removed = oldChannels.filter(c => !newChannels.includes(c));
  removed.forEach(channel => ws.unsubscribe(channel));
  
  const added = newChannels.filter(c => !oldChannels.includes(c));
  added.forEach(channel => ws.subscribe(channel));
});
```

### 4. Memoization

```typescript
// Cache expensive computations
const healthLevel = computed(() => {
  return calculateHealthLevel(metrics.value);
}, { cacheKey: 'health-level' });

const trendPercent = computed(() => {
  return calculateTrend(current, previous, timeWindow);
});
```

### 5. Debouncing & Throttling

```typescript
// Debounce chart updates
const chartUpdateDebounce = debounce((data) => {
  chart.update(data);
}, 500);

// Throttle scroll events
const handleScroll = throttle(() => {
  updateVisibleItems();
}, 16); // 60fps
```

### 6. CSS Optimization

```css
/* Use CSS containment for isolated components */
.metric-card {
  contain: layout style paint;
  will-change: transform;
}

/* Reduce repaints with transforms */
.sparkline {
  transform: translateZ(0);
  backface-visibility: hidden;
}

/* GPU acceleration */
.dashboard-grid {
  transform: translateZ(0);
}
```

---

## Security Considerations

### 1. Data Sanitization

```typescript
// sanitizers.ts
export function sanitizeActivityEntry(entry: ActivityEntry): ActivityEntry {
  return {
    ...entry,
    message: sanitizeHtml(entry.message),
    details: entry.details ? sanitizeObject(entry.details) : undefined,
  };
}
```

### 2. CSRF Protection

```typescript
// useApi.ts
const csrfToken = ref<string>('');

onMounted(async () => {
  const { data } = await axios.get('/api/csrf-token');
  csrfToken.value = data.token;
});

// Include in all requests
api.defaults.headers.common['X-CSRF-Token'] = csrfToken.value;
```

### 3. Rate Limiting

```typescript
// Rate limit API calls
const createRateLimiter = (limit: number, window: number) => {
  let count = 0;
  let resetTime = Date.now() + window;
  
  return {
    allowed: (): boolean => {
      if (Date.now() > resetTime) {
        count = 0;
        resetTime = Date.now() + window;
      }
      return ++count <= limit;
    },
  };
};

const metricsLimiter = createRateLimiter(10, 1000);
```

### 4. XSS Prevention

```typescript
// Use v-text for untrusted content
<span v-text="unsafeContent"></span>

// Use sanitized HTML
<div v-html="sanitizeHtml(htmlContent)"></div>

// Escape in attributes
:title="escapeAttr(tooltip)"
```

### 5. Content Security Policy

```html
<!-- In index.html -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               script-src 'self' 'nonce-{random}'; 
               style-src 'self' 'nonce-{random}'; 
               img-src 'self' data:; 
               connect-src 'self' ws: wss:">
```

---

## Testing Strategy

### Unit Tests

```typescript
// tests/dashboard/MetricCard.spec.ts
import { mount } from '@vue/test-utils';
import MetricCard from '@/components/dashboard/MetricCard.vue';

describe('MetricCard', () => {
  it('displays metric value correctly', () => {
    const wrapper = mount(MetricCard, {
      props: {
        title: 'Requests/sec',
        value: 1234.5,
        unit: '/sec',
      },
    });
    
    expect(wrapper.text()).toContain('1234.5');
    expect(wrapper.text()).toContain('/sec');
  });
  
  it('applies critical status when value exceeds threshold', () => {
    const wrapper = mount(MetricCard, {
      props: {
        title: 'Memory',
        value: 900,
        unit: 'MB',
        threshold: { warning: 700, critical: 800, max: 1000 },
      },
    });
    
    expect(wrapper.classes()).toContain('critical');
  });
  
  it('emits click event when clicked', async () => {
    const wrapper = mount(MetricCard, {
      props: { title: 'Test', value: 100, unit: 'ms' },
    });
    
    await wrapper.trigger('click');
    expect(wrapper.emitted('click')).toHaveLength(1);
  });
});
```

### Integration Tests

```typescript
// tests/dashboard/DashboardTab.spec.ts
import { mount, flushPromises } from '@vue/test-utils';
import DashboardTab from '@/components/DashboardTab.vue';
import * as api from '@/composables/useApi';

describe('DashboardTab Integration', () => {
  it('loads metrics on mount', async () => {
    const mockMetrics = {
      requests_per_sec: 1000,
      memory_usage_mb: 150,
      // ...
    };
    
    vi.spyOn(api, 'getMetrics').mockResolvedValue(mockMetrics);
    
    const wrapper = mount(DashboardTab);
    await flushPromises();
    
    expect(wrapper.vm.metrics).toEqual(mockMetrics);
  });
  
  it('updates metrics on WebSocket message', async () => {
    const wrapper = mount(DashboardTab);
    
    // Simulate WebSocket message
    wrapper.vm.ws.emit('message', {
      data: JSON.stringify({
        type: 'metrics',
        payload: { requests_per_sec: 2000 },
      }),
    });
    
    await wrapper.vm.$nextTick();
    expect(wrapper.vm.metrics.requests_per_sec).toBe(2000);
  });
  
  it('unsubscribes from WebSocket on unmount', () => {
