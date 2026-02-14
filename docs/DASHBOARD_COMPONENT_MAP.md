# 🗺️ Dashboard Component Map - Visual Reference

**Purpose**: Quick visual reference for Dashboard component architecture  
**Status**: Complete Architecture Design  
**Last Updated**: 2025-01-20

---

## 📐 Component Tree Diagram

```
DashboardTab (Root)
│
├─── DashboardHeader
│    ├── StatusBadge (System Health)
│    ├── StatusBadge (Proxy Status)
│    ├── StatusBadge (WebSocket Status)
│    ├── TimeDisplay
│    └── QuickNavButtons
│
├─── MetricsGrid (Responsive 3-col)
│    ├── MetricCard (Requests/sec)
│    ├── MetricCard (Avg Response Time)
│    ├── MetricCard (Memory Usage)
│    ├── MetricCard (Active Connections)
│    ├── MetricCard (Bytes/sec)
│    └── MetricCard (Error Rate)
│
├─── DashboardPanels (Two-column flex)
│    ├── RecentActivityPanel
│    │   ├── ActivityTimeline
│    │   │   └── ActivityItem[] (virtual scroll)
│    │   └── TopEndpointsList
│    │       └── EndpointItem[]
│    │
│    └── SystemHealthPanel
│        ├── HealthGaugesGrid
│        │   ├── ProgressRing (CPU)
│        │   ├── ProgressRing (Memory)
│        │   ├── ProgressRing (Disk)
│        │   └── ProgressRing (Network)
│        ├── ConnectionGraph
│        └── UptimeDisplay
│
└─── QuickActionsBar
     ├── ProxyToggleButton
     ├── ExportButton
     ├── ClearCacheButton
     └── SettingsButton
```

---

## 🔄 Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    External Data Sources                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  REST API (1s poll)              WebSocket (real-time)     │
│  ├── /api/dashboard/metrics       ├── metrics_update       │
│  ├── /api/dashboard/health        ├── connection_change    │
│  ├── /api/dashboard/activity      ├── activity_log        │
│  ├── /api/dashboard/endpoints     └── health_change       │
│  └── /api/proxy/status                                     │
│                                                              │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         ▼                       ▼
    ┌──────────┐         ┌──────────────┐
    │ Composables (Caching & Batching)   │
    ├──────────┤         ├──────────────┤
    │useDashboardMetrics  │useDashboardWebSocket
    │useDashboardHealth   │useSystemStatus
    │useMetricUpdates     │useConnectionStats
    └──────┬──────────────┴──────┬───────┘
           │                     │
           └──────────┬──────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │   Reactive State (Refs)      │
        ├─────────────────────────────┤
        │ • metrics (SystemMetrics)    │
        │ • health (HealthStatus)      │
        │ • activity (ActivityEntry[]) │
        │ • timeSeries (Map)           │
        │ • proxyStatus (ProxyStatus)  │
        └──────────────┬───────────────┘
                       │
         ┌─────────────┼─────────────────────────┬──────────────┐
         │             │                         │              │
         ▼             ▼                         ▼              ▼
    ┌─────────┐  ┌───────────┐         ┌──────────┐      ┌──────────┐
    │Dashboard │  │  Metrics  │         │ Activity │      │  System  │
    │Header   │  │   Grid    │         │  Panel   │      │  Health  │
    └─────────┘  └────┬──────┘         └────┬─────┘      └────┬─────┘
                      │                      │                 │
                      ▼                      ▼                 ▼
                   ┌──────────────┐    ┌──────────────┐   ┌──────────────┐
                   │ MetricCard[] │    │Activity Item │   │ProgressRing[]
                   │ • Value      │    │ • Timestamp  │   │ • Value      │
                   │ • Trend      │    │ • Message    │   │ • Status     │
                   │ • Sparkline  │    │ • Type       │   │ • Label      │
                   └──────────────┘    └──────────────┘   └──────────────┘
```

---

## 📋 Component Specifications Matrix

| Component | Lines | Role | Renders | Updates | Reuses |
|-----------|-------|------|---------|---------|--------|
| **DashboardTab** | 150 | Orchestrator | 1 | Every 1s | N/A |
| **DashboardHeader** | 120 | Status bar | 1 | On event | StatusBadge×3 |
| **MetricsGrid** | 80 | Container | 6 | On update | MetricCard×6 |
| **MetricCard** | 90 | Display | 1-N | On change | Sparkline |
| **RecentActivityPanel** | 120 | Activity feed | 2 sections | On event | Timeline |
| **SystemHealthPanel** | 140 | Health gauges | 4 gauges | Every 5s | ProgressRing×4 |
| **QuickActionsBar** | 100 | Controls | 4 buttons | Static | - |
| **StatusBadge** | 60 | Status indicator | 1 | On change | - |
| **ProgressRing** | 80 | Circular progress | 1 | On update | - |
| **TimeSeriesChart** | 130 | Line chart | 1 | On append | SVG |
| **ConnectionGraph** | 110 | Connection viz | 1 | Every 5s | Canvas |
| **ActivityTimeline** | 100 | Timeline view | N | Virtual scroll | - |

**Total Codebase**: ~1,200 lines of Vue components

---

## 🎯 Props & Emit Contracts

### DashboardTab (Root)
```
Props: None (self-contained)
Emits: None (contains everything)
Provides: dashboard state via provide()
Scope: { metrics, health, activity, proxyStatus }
```

### DashboardHeader
```
Props:
  - proxyStatus: ProxyStatus
  - metricsHealth: 'healthy' | 'warning' | 'critical'
  - lastUpdate: number (timestamp)
  - wsConnected: boolean

Emits:
  - toggle-proxy: void
  - export: void
  - settings: void
```

### MetricsGrid
```
Props:
  - metrics: SystemMetrics | null
  - timeSeries?: Map<string, MetricPoint[]>
  - isLoading?: boolean

Emits: None
```

### MetricCard
```
Props:
  - title: string
  - value: number
  - unit: string
  - icon?: string
  - trend?: { direction: 'up'|'down'|'stable', percent: number }
  - threshold?: { warning, critical, max }
  - sparklineData?: MetricPoint[]
  - loading?: boolean

Emits: click: void
```

### RecentActivityPanel
```
Props:
  - activities: ActivityEntry[]
  - topEndpoints: TopEndpoint[]
  - isLoading?: boolean

Emits:
  - activity-click: ActivityEntry
  - endpoint-click: TopEndpoint
```

### SystemHealthPanel
```
Props:
  - health: HealthStatus
  - connectionStats: ConnectionStats
  - isLoading?: boolean

Emits: None
```

### QuickActionsBar
```
Props:
  - proxyStatus: ProxyStatus
  - canClear: boolean

Emits:
  - toggle-proxy: void
  - clear-traffic: void
  - export: 'json' | 'csv' | 'har'
  - settings: void
```

---

## 💾 State Management Strategy

### Global Dashboard State (Root Level)

```typescript
// Managed in DashboardTab via ref()
const state = {
  // Metrics (updated via useDashboardMetrics)
  metrics: SystemMetrics | null,
  timeSeries: Map<string, MetricPoint[]>,
  
  // Activity (updated via WebSocket)
  recentActivity: ActivityEntry[],
  topEndpoints: TopEndpoint[],
  
  // Health (updated via useSystemHealth)
  systemHealth: HealthStatus,
  
  // Status (updated via API)
  proxyStatus: ProxyStatus,
  connectionStats: ConnectionStats,
  
  // UI State
  isLoading: boolean,
  error: string | null,
  lastUpdate: number,
};
```

### Composable State (Encapsulated)

```typescript
// useDashboardMetrics.ts
{
  metrics: Ref<SystemMetrics | null>,
  isLoading: Ref<boolean>,
  error: Ref<string | null>,
  lastUpdate: Ref<number>,
}

// useDashboardWebSocket.ts
{
  isConnected: Ref<boolean>,
  lastEvent: Ref<DashboardEvent | null>,
  reconnectAttempts: Ref<number>,
}

// useSystemHealth.ts
{
  health: ComputedRef<HealthStatus>,
}
```

### Component-Level State

```typescript
// MetricCard
{
  statusClass: ComputedRef<string>,
  fillPercentage: ComputedRef<number>,
  sparklinePoints: ComputedRef<string>,
}

// RecentActivityPanel
{
  selectedActivity: Ref<ActivityEntry | null>,
  filteredEndpoints: ComputedRef<TopEndpoint[]>,
}
```

---

## 🔌 API Integration Points

### REST Endpoints Called

```
┌─────────────────────────────────────────────────────────┐
│            API Endpoints Used by Dashboard              │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ GET /api/dashboard/metrics (1s poll)                   │
│   ├── Calls from: useDashboardMetrics()                │
│   ├── Returns: SystemMetrics                           │
│   └── Cache: 1s (inline in composable)                 │
│                                                         │
│ GET /api/dashboard/health (5s poll)                    │
│   ├── Calls from: useSystemHealth()                    │
│   ├── Returns: HealthStatus                            │
│   └── Cache: 5s                                        │
│                                                         │
│ GET /api/dashboard/activity?limit=10 (on mount)        │
│   ├── Calls from: onMounted in DashboardTab            │
│   ├── Returns: ActivityEntry[]                         │
│   └── Updates: Via WebSocket after                     │
│                                                         │
│ GET /api/dashboard/endpoints?limit=5 (5s)              │
│   ├── Calls from: loadDashboardData()                  │
│   ├── Returns: TopEndpoint[]                           │
│   └── Sort: by requests_count DESC                     │
│                                                         │
│ GET /api/proxy/status (1s poll)                        │
│   ├── Calls from: loadProxyStatus()                    │
│   ├── Returns: ProxyStatus                             │
│   └── Cache: 1s                                        │
│                                                         │
│ POST /api/proxy/start (on button click)                │
│   ├── Body: ProxyConfig                                │
│   ├── Returns: ProxyStatus                             │
│   └── After: Refetch status                            │
│                                                         │
│ POST /api/proxy/stop (on button click)                 │
│   ├── Returns: ProxyStatus                             │
│   └── After: Refetch status                            │
│                                                         │
│ POST /api/dashboard/export?format=json|csv             │
│   ├── Returns: Blob                                    │
│   └── Trigger: Download via URL.createObjectURL       │
│                                                         │
│ DELETE /api/traffic (on clear button click)            │
│   ├── Returns: { cleared_count: number }               │
│   └── After: Refetch metrics                           │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### WebSocket Channels Subscribed

```
┌─────────────────────────────────────────────────────────┐
│     WebSocket Channels (ws://127.0.0.1:3000/ws)        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ /ws?channel=metrics (every 1s)                         │
│   Payload: { type: 'metrics', payload: SystemMetrics } │
│   ├── Updates: metrics in DashboardTab                 │
│   └── Updates: MetricCard components                   │
│                                                         │
│ /ws?channel=activity (on new request)                  │
│   Payload: { type: 'activity', payload: ActivityEntry }│
│   └── Prepends: new entry to recentActivity[]          │
│                                                         │
│ /ws?channel=health (every 5s)                          │
│   Payload: { type: 'health', payload: HealthStatus }   │
│   └── Updates: systemHealth in DashboardTab            │
│                                                         │
│ /ws?channel=connections (on change)                    │
│   Payload: { type: 'connections', payload: Stats }     │
│   └── Updates: connectionStats                         │
│                                                         │
│ /ws?channel=proxy (on status change)                   │
│   Payload: { type: 'proxy', payload: ProxyStatus }     │
│   └── Updates: proxyStatus                             │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 📊 Data Update Frequency Matrix

| Data Source | Endpoint/Channel | Frequency | Priority | Batches |
|-------------|-----------------|-----------|----------|---------|
| Metrics | REST poll | 1s | High | 50 updates |
| Health | WebSocket | 5s | Medium | On change |
| Activity | WebSocket | Event | High | Prepend only |
| Proxy Status | REST poll | 1s | High | 1 per poll |
| Endpoints | REST poll | 5s | Low | 10 items |
| Connections | WebSocket | Event | Medium | On change |

---

## ⚡ Performance Optimization Techniques

### 1. Rendering Optimization
```typescript
// Virtual scrolling for activity lists > 50 items
import { useVirtualizer } from '@tanstack/vue-virtual';

const virtualizer = useVirtualizer({
  count: recentActivity.value.length,
  getScrollElement: () => container.value,
  estimateSize: () => 60,
  overscan: 10,  // Render 10 extra items for smoothness
});
```

### 2. Update Batching
```typescript
// Batch metric updates to reduce re-renders
const batchMetrics = (updates: MetricsUpdate[]) => {
  pendingUpdates.push(...updates);
  
  if (!batchScheduled) {
    batchScheduled = true;
    requestAnimationFrame(() => {
      applyBatch(pendingUpdates);
      pendingUpdates = [];
      batchScheduled = false;
    });
  }
};
```

### 3. Memoization
```typescript
// Cache expensive computations
const healthLevel = computed(() => {
  return calculateHealthLevel(metrics.value);
}, { cacheKey: 'health-level' });
```

### 4. Debouncing
```typescript
// Debounce search/filter inputs
const debouncedFilter = debounce((term) => {
  updateActivityFilter(term);
}, 300);
```

### 5. Code Splitting
```typescript
// Lazy load non-critical components
const SystemHealthPanel = defineAsyncComponent(() =>
  import('./SystemHealthPanel.vue')
);
```

---

## 🎨 Styling Architecture

### CSS Cascade Structure

```
global.css
├── CSS Variables (--color-*, --shadow-*, --transition-*)
├── Reset & Normalization
└── Global Animations (@keyframes)

dashboard.module.css
├── .dashboard-tab
├── .metrics-grid
├── .metric-card
│   ├── .metric-card:hover
│   ├── .metric-card.status-warning
│   └── .metric-card.status-critical
├── .dashboard-panels
├── .panel
└── .error-alert

dashboard/
├── MetricCard.module.css
├── MetricsGrid.module.css
├── DashboardHeader.module.css
├── SystemHealthPanel.module.css
├── RecentActivityPanel.module.css
└── QuickActionsBar.module.css

shared/
├── tokens.css (colors, spacing, shadows)
├── animations.css (@keyframes)
└── typography.css (fonts, sizes)
```

### CSS Variables Reference

```css
:root {
  /* Colors */
  --color-bg-primary: #0a0a0f;
  --color-bg-secondary: #1a1a2e;
  --color-border-default: #2a2a3e;
  
  --color-cyan: #00d4ff;
  --color-magenta: #ff006e;
  --color-orange: #ffb800;
  --color-purple: #8b5cf6;
  
  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.2);
  --shadow-cyan: 0 0 20px rgba(0, 212, 255, 0.15);
  
  /* Transitions */
  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-normal: 300ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 500ms cubic-bezier(0.4, 0, 0.2, 1);
  
  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 20px;
  --spacing-xl: 24px;
  
  /* Typography */
  --font-mono: 'Fira Code', 'JetBrains Mono', monospace;
  --font-sans: 'Inter', 'Segoe UI', sans-serif;
}
```

---

## 🧪 Testing Coverage Map

### Unit Tests

```
composables/
├── __tests__/
│   ├── useDashboardMetrics.spec.ts (8 tests)
│   ├── useDashboardWebSocket.spec.ts (6 tests)
│   └── useSystemHealth.spec.ts (5 tests)

utils/
├── __tests__/
│   ├── formatters.spec.ts (12 tests)
│   └── thresholds.spec.ts (6 tests)
```

### Component Tests

```
components/
├── __tests__/
│   ├── dashboard/
│   │   ├── MetricCard.spec.ts (8 tests)
│   │   ├── MetricsGrid.spec.ts (6 tests)
│   │   ├── DashboardHeader.spec.ts (7 tests)
│   │   ├── RecentActivityPanel.spec.ts (5 tests)
│   │   ├── SystemHealthPanel.spec.ts (6 tests)
│   │   └── QuickActionsBar.spec.ts (5 tests)
│   └── DashboardTab.spec.ts (10 integration tests)
```

**Total Test Cases**: ~80 tests  
**Target Coverage**: >90% line coverage

---

## 🚀 Build & Deployment

### Bundle Size Targets

```
Component Breakdown:
├── DashboardTab.vue (15KB)
├── MetricCard.vue (12KB)
├── MetricsGrid.vue (8KB)
├── DashboardHeader.vue (10KB)
├── RecentActivityPanel.vue (12KB)
├── SystemHealthPanel.vue (14KB)
├── QuickActionsBar.vue (8KB)
├── Composables (18KB)
├── Utils (8KB)
└── CSS (12KB)

Total: ~117KB (uncompressed)
Gzipped: ~35KB (goal: <50KB)
```

### Performance Checklist

- [ ] First Contentful Paint: <1.5s
- [ ] Time to Interactive: <3s
- [ ] Lighthouse Score: >90
- [ ] Core Web Vitals Pass
- [ ] Zero Layout Shifts (CLS)
- [ ] Smooth 60fps Animations
- [ ] Memory Usage: <50MB

---

## 📚 File Structure (Final)

```
src/
├── components/
│   ├── DashboardTab.vue                          # 150 lines
│   └── dashboard/                                 # Component subfolder
│       ├── DashboardHeader.vue                    # 120 lines
│       ├── MetricsGrid.vue                        # 80 lines
│       ├── MetricCard.vue                         # 90 lines
│       ├── RecentActivityPanel.vue                # 120 lines
│       ├── SystemHealthPanel.vue                  # 140 lines
│       ├── QuickActionsBar.vue                    # 100 lines
│       ├── StatusBadge.vue                        # 60 lines
│       ├── ProgressRing.vue                       # 80 lines
│       ├── TimeSeriesChart.vue                    # 130 lines
│       ├── ConnectionGraph.vue                    # 110 lines
│       └── ActivityTimeline.vue                   # 100 lines
│
├── composables/
│   └── dashboard/
│       ├── useDashboardMetrics.ts                 # Polling logic
│       ├── useDashboardWebSocket.ts               # WS management
│       ├── useSystemHealth.ts                     # Health calcs
│       └── useMetricUpdates.ts                    # Update batching
│
├── utils/
│   └── dashboard/
│       ├── formatters.ts                          # Number/time formats
│       ├── thresholds.ts                          # Health thresholds
│       └── validators.ts                          # Data validation
│
├── types/
│   └── dashboard.ts                               # Type definitions
│
└── styles/
    └── dashboard/
        ├── dashboard.module.css                   # Main styles
        ├── tokens.css                             # Design tokens
        └── animations.css                         # Keyframes
```

---

## ✅ Implementation Checklist

### Phase 1: Foundation
- [ ] Create `types/dashboard.ts`
- [ ] Create composables (`useDashboardMetrics.ts`, etc.)
- [ ] Create utility functions (`formatters.ts`, etc.)
- [ ] Setup CSS variables in `styles/tokens.css`

### Phase 2: Base Components
- [ ] `MetricCard.vue` - Core reusable component
- [ ] `MetricsGrid.vue` - Grid container
- [ ] `StatusBadge.vue` - Status indicator
- [ ] `ProgressRing.vue` - Circular progress

### Phase 3: Container Components
- [ ] `DashboardHeader.vue` - Top section
- [ ] `RecentActivityPanel.vue` - Activity feed
- [ ] `SystemHealthPanel.vue` - Health gauges
- [ ] `QuickActionsBar.vue` - Action buttons

### Phase 4: Advanced Features
- [ ] `TimeSeriesChart.vue` - Line chart
- [ ] `ConnectionGraph.vue` - Connection viz
- [ ] `ActivityTimeline.vue` - Timeline view

### Phase 5: Integration
- [ ] Update `DashboardTab.vue` root component
- [ ] Wire all API endpoints
- [ ] Wire WebSocket subscriptions
- [ ] Add error handling & loading states

### Phase 6: Polish & Testing
- [ ] Write unit tests (80+ tests)
- [ ] Performance optimization
- [ ] Accessibility audit (WCAG 2.1 AA)
- [ ] Cross-browser testing
- [ ] Deploy & monitor

---

## 🔗 Cross-References

- **Architecture**: [DASHBOARD_ARCHITECTURE.md](./DASHBOARD_ARCHITECTURE.md)
- **Implementation Guide**: [DASHBOARD_IMPLEMENTATION_GUIDE.md](./DASHBOARD_IMPLEMENTATION_GUIDE.md)
- **API Spec**: [API.md](./API.md)
- **UI Design**: [UI_DESIGN_SPEC.md](./UI_DESIGN_SPEC.md)

---

**Ready to implement? Start with Phase 1!**
