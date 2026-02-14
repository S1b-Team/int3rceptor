# Phase 4 Task 1: Integration Guide - Using Real Metrics API

**Purpose**: Shows how to integrate the new `useDashboardMetrics` composable with real API calls into your DashboardTab component.

**Duration**: 30 minutes (integration only, implementation is complete)

---

## 📋 Overview

The `useDashboardMetrics` composable is now ready to be integrated into your dashboard components. This guide walks through:

1. Basic integration into DashboardTab
2. Handling loading and error states
3. Displaying metrics in cards
4. Monitoring update frequency
5. Manual refresh triggers

---

## 🔧 Step 1: Update DashboardTab Component

### Current State
Your DashboardTab likely has mock metrics or no metrics at all.

### Update Required
Import and use the new composable:

```vue
<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue';
import { useDashboardMetrics } from '@/composables/dashboard/useDashboardMetrics';
import MetricsGrid from '@/components/dashboard/MetricsGrid.vue';

// Initialize composable
const { 
  metrics, 
  isLoading, 
  error, 
  lastUpdated,
  timeSinceUpdate,
  startAutoFetch, 
  stopAutoFetch,
  fetchMetrics,
  clearError 
} = useDashboardMetrics();

// Start polling when component mounts
onMounted(() => {
  console.log('Dashboard mounted - starting metrics polling');
  startAutoFetch(1000); // Poll every 1 second
});

// Cleanup is automatic, but you can add explicit handler if needed
onBeforeUnmount(() => {
  console.log('Dashboard unmounting - stopping metrics polling');
  stopAutoFetch();
});
</script>

<template>
  <div class="dashboard-tab">
    <!-- Header -->
    <div class="dashboard-header">
      <h1>System Dashboard</h1>
      <button 
        @click="fetchMetrics" 
        :disabled="isLoading"
        class="refresh-btn"
      >
        {{ isLoading ? 'Fetching...' : 'Refresh' }}
      </button>
    </div>

    <!-- Error Alert -->
    <div v-if="error" class="error-alert">
      <div class="error-content">
        <span class="error-icon">⚠️</span>
        <span class="error-message">{{ error }}</span>
        <button @click="clearError" class="error-close">×</button>
      </div>
    </div>

    <!-- Loading Skeleton (first load) -->
    <div v-if="isLoading && !metrics" class="loading-skeleton">
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
    </div>

    <!-- Metrics Grid (main content) -->
    <MetricsGrid v-if="metrics" :metrics="metrics" />

    <!-- Last Update Info -->
    <div v-if="metrics" class="update-info">
      <small>Last updated: {{ new Date(lastUpdated).toLocaleTimeString() }}</small>
      <small>({{ (timeSinceUpdate / 1000).toFixed(1) }}s ago)</small>
    </div>
  </div>
</template>

<style scoped>
.dashboard-tab {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.refresh-btn {
  padding: 8px 16px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-alert {
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.error-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.error-icon {
  font-size: 20px;
}

.error-message {
  flex: 1;
  color: #721c24;
  font-weight: 500;
}

.error-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #721c24;
}

.loading-skeleton {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin: 24px 0;
}

.skeleton-card {
  height: 150px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
  border-radius: 8px;
}

@keyframes loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

.update-info {
  display: flex;
  gap: 16px;
  justify-content: flex-end;
  margin-top: 16px;
  color: #666;
  font-size: 12px;
}
</style>
```

---

## 🎯 Step 2: Create MetricsGrid Component (if not exists)

Your MetricsGrid should accept the metrics object:

```vue
<script setup lang="ts">
import type { SystemMetrics } from '@/types/dashboard';
import MetricCard from './MetricCard.vue';

interface Props {
  metrics: SystemMetrics;
}

defineProps<Props>();

// Calculate trend (simplified - real version would track history)
const getTrend = (value: number, prev: number) => {
  if (!prev) return { direction: 'stable' as const, percent: 0 };
  const change = ((value - prev) / prev) * 100;
  return {
    direction: change > 0 ? 'up' as const : change < 0 ? 'down' as const : 'stable' as const,
    percent: Math.abs(change),
  };
};
</script>

<template>
  <div class="metrics-grid">
    <!-- Requests per second -->
    <MetricCard
      title="Requests/sec"
      :value="metrics.requests_per_sec"
      unit="/sec"
      icon="📊"
      :trend="{ direction: 'stable', percent: 0 }"
      :threshold="{ warning: 5000, critical: 10000, max: 15000 }"
    />

    <!-- Average Response Time -->
    <MetricCard
      title="Avg Response Time"
      :value="metrics.avg_response_time_ms"
      unit="ms"
      icon="⚡"
      :threshold="{ warning: 500, critical: 1000, max: 2000 }"
    />

    <!-- Memory Usage -->
    <MetricCard
      title="Memory Usage"
      :value="metrics.memory_usage_mb"
      unit="MB"
      icon="💾"
      :threshold="{ warning: 512, critical: 1024, max: 2048 }"
    />

    <!-- Active Connections -->
    <MetricCard
      title="Active Connections"
      :value="metrics.active_connections"
      unit=""
      icon="🔗"
      :threshold="{ warning: 100, critical: 200, max: 500 }"
    />

    <!-- Bytes In/Out -->
    <MetricCard
      title="Bytes In"
      :value="metrics.bytes_in_sec / 1024 / 1024"
      unit="MB/s"
      icon="📥"
      :threshold="{ warning: 50, critical: 100, max: 500 }"
    />

    <MetricCard
      title="Bytes Out"
      :value="metrics.bytes_out_sec / 1024 / 1024"
      unit="MB/s"
      icon="📤"
      :threshold="{ warning: 50, critical: 100, max: 500 }"
    />

    <!-- Error Rate -->
    <MetricCard
      title="Error Rate"
      :value="metrics.error_rate_percent"
      unit="%"
      icon="❌"
      :threshold="{ warning: 1, critical: 5, max: 10 }"
    />

    <!-- CPU Usage (if available) -->
    <MetricCard
      v-if="metrics.cpu_percent !== undefined"
      title="CPU Usage"
      :value="metrics.cpu_percent"
      unit="%"
      icon="⚙️"
      :threshold="{ warning: 50, critical: 80, max: 100 }"
    />

    <!-- Disk Usage (if available) -->
    <MetricCard
      v-if="metrics.disk_percent !== undefined"
      title="Disk Usage"
      :value="metrics.disk_percent"
      unit="%"
      icon="💿"
      :threshold="{ warning: 70, critical: 90, max: 100 }"
    />

    <!-- Uptime -->
    <MetricCard
      title="Uptime"
      :value="metrics.uptime_seconds / 3600"
      unit="hours"
      icon="⏱️"
    />
  </div>
</template>

<style scoped>
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  padding: 16px 0;
}

@media (max-width: 1024px) {
  .metrics-grid {
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  }
}

@media (max-width: 640px) {
  .metrics-grid {
    grid-template-columns: 1fr;
  }
}
</style>
```

---

## 📊 Step 3: Display Metrics in Template

### Simple Display
```vue
<div v-if="metrics" class="metrics-display">
  <div>Requests: {{ metrics.requests_per_sec.toFixed(2) }}/sec</div>
  <div>Memory: {{ metrics.memory_usage_mb.toFixed(1) }} MB</div>
  <div>Connections: {{ metrics.active_connections }}</div>
</div>
```

### With Loading/Error States
```vue
<!-- Loading state (first load) -->
<div v-if="isLoading && !metrics" class="state-loading">
  Loading metrics...
</div>

<!-- Error state -->
<div v-else-if="error" class="state-error">
  ⚠️ {{ error }}
  <button @click="clearError">Dismiss</button>
</div>

<!-- Success state -->
<div v-else-if="metrics" class="state-success">
  <MetricsGrid :metrics="metrics" />
</div>

<!-- Never reached (no data and not loading/error) -->
<div v-else class="state-empty">
  No data available
</div>
```

---

## 🔄 Step 4: Handle Update Frequency

### Show Update Status
```vue
<template>
  <div class="metrics-status">
    <!-- Active polling indicator -->
    <div :class="['poll-status', { 'poll-active': !isLoading }]">
      {{ isLoading ? '⌛ Fetching...' : '✓ Live' }}
    </div>

    <!-- Time since last update -->
    <div class="last-update">
      <small>
        Last: {{ formatTime(lastUpdated) }}
        ({{ (timeSinceUpdate / 1000).toFixed(1) }}s ago)
      </small>
    </div>

    <!-- Polling frequency indicator -->
    <div class="poll-frequency">
      <small>Updates every 1 second</small>
    </div>
  </div>
</template>

<script setup lang="ts">
const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString();
};
</script>

<style scoped>
.metrics-status {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  font-size: 12px;
}

.poll-status {
  padding: 4px 8px;
  background: #ff6b6b;
  color: white;
  border-radius: 2px;
}

.poll-status.poll-active {
  background: #51cf66;
}

.last-update,
.poll-frequency {
  color: #666;
}
</style>
```

---

## 🔧 Step 5: Add Manual Refresh Button

```vue
<template>
  <div class="refresh-controls">
    <!-- Manual refresh button -->
    <button 
      @click="handleRefresh"
      :disabled="isLoading"
      class="btn-refresh"
    >
      <span v-if="!isLoading">🔄 Refresh</span>
      <span v-else>⌛ Fetching...</span>
    </button>

    <!-- Auto-poll toggle -->
    <button 
      @click="toggleAutoFetch"
      :class="['btn-toggle', { active: isPolling }]"
    >
      {{ isPolling ? '⏸ Pause' : '▶ Resume' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useDashboardMetrics } from '@/composables/dashboard/useDashboardMetrics';

const { fetchMetrics, startAutoFetch, stopAutoFetch } = useDashboardMetrics();
const isPolling = ref(true);

const handleRefresh = async () => {
  await fetchMetrics();
  console.log('Manual refresh completed');
};

const toggleAutoFetch = () => {
  if (isPolling.value) {
    stopAutoFetch();
    isPolling.value = false;
  } else {
    startAutoFetch();
    isPolling.value = true;
  }
};
</script>

<style scoped>
.refresh-controls {
  display: flex;
  gap: 8px;
  margin: 16px 0;
}

button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-weight: 500;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-toggle.active {
  background: #51cf66;
  color: white;
  border-color: #51cf66;
}
</style>
```

---

## 🧪 Step 6: Testing the Integration

### 1. Verify Component Mounts
```typescript
// Add to your component
onMounted(() => {
  console.log('✓ Dashboard mounted');
  console.log('✓ Metrics polling started');
});

onBeforeUnmount(() => {
  console.log('✓ Dashboard unmounting');
  console.log('✓ Metrics polling stopped');
});
```

### 2. Check DevTools Network
1. Open DevTools (F12)
2. Network tab
3. Filter: `/api/dashboard/metrics`
4. Should see requests every 1 second
5. Status should be 200 OK

### 3. Monitor Metrics Updates
```typescript
// Watch for changes
watch(() => metrics.value, (newMetrics) => {
  console.log('Metrics updated:', {
    timestamp: newMetrics?.timestamp,
    requests_per_sec: newMetrics?.requests_per_sec,
  });
});
```

### 4. Test Error Scenarios
```typescript
// Simulate error by disabling network
// DevTools → Network → Offline
// Should see error message
// Should retry 3 times
// Then show "max retries exceeded"
```

---

## 🎯 Complete Working Example

Here's a minimal working DashboardTab:

```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import { useDashboardMetrics } from '@/composables/dashboard/useDashboardMetrics';

const { metrics, isLoading, error, startAutoFetch } = useDashboardMetrics();

onMounted(() => startAutoFetch());
</script>

<template>
  <div class="dashboard">
    <h1>Dashboard</h1>

    <!-- Error -->
    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <!-- Loading -->
    <div v-if="isLoading && !metrics" class="alert alert-info">
      Loading metrics...
    </div>

    <!-- Metrics -->
    <div v-if="metrics" class="metrics">
      <div class="metric">
        <span>Requests/sec:</span>
        <strong>{{ metrics.requests_per_sec.toFixed(2) }}</strong>
      </div>
      <div class="metric">
        <span>Memory:</span>
        <strong>{{ metrics.memory_usage_mb.toFixed(1) }} MB</strong>
      </div>
      <div class="metric">
        <span>Connections:</span>
        <strong>{{ metrics.active_connections }}</strong>
      </div>
      <div class="metric">
        <span>Response Time:</span>
        <strong>{{ metrics.avg_response_time_ms.toFixed(1) }} ms</strong>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  padding: 20px;
}

.alert {
  padding: 12px;
  margin: 16px 0;
  border-radius: 4px;
}

.alert-error {
  background: #f8d7da;
  color: #721c24;
}

.alert-info {
  background: #d1ecf1;
  color: #0c5460;
}

.metrics {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-top: 20px;
}

.metric {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
}

.metric span {
  font-weight: 500;
}

.metric strong {
  font-size: 18px;
  color: #007bff;
}
</style>
```

---

## 📚 Key Points to Remember

1. **Always call `startAutoFetch()` in `onMounted()`** - This starts polling
2. **Auto-cleanup works automatically** - But you can call `stopAutoFetch()` manually
3. **Error handling is built-in** - Display the `error` ref in UI
4. **isLoading indicates fetch in progress** - Use for loading states/skeletons
5. **timeSinceUpdate is computed** - Updates every millisecond
6. **lastUpdated is the timestamp** - Convert with `new Date(lastUpdated).toLocaleTimeString()`

---

## 🚀 Next Steps

After integrating into DashboardTab:

1. **Test thoroughly** - All error scenarios, loading states, updates
2. **Monitor performance** - Check DevTools for memory leaks
3. **Move to Task 2** - Fetch real activity data
4. **Add WebSocket** - Task 5 for real-time push updates

---

**Status**: Ready to integrate ✅  
**Estimated Time**: 30 minutes  
**Difficulty**: Easy (copy-paste and adapt)
