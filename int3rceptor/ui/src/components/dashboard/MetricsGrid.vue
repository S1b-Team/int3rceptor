<template>
  <div class="metrics-grid">
    <!-- Requests/sec Card -->
    <MetricCard
      title="Requests/sec"
      :value="metrics?.requests_per_sec ?? 0"
      unit="/sec"
      icon="📊"
      :threshold="{ warning: 5000, critical: 10000, max: 15000 }"
      :trend="{ direction: 'up', percent: 12, timeWindow: '5m' }"
      :sparkline-data="timeSeries.get('requests_per_sec')?.dataPoints"
      :loading="isLoading"
    />

    <!-- Avg Response Time Card -->
    <MetricCard
      title="Avg Response Time"
      :value="metrics?.avg_response_time_ms ?? 0"
      unit="ms"
      icon="⏱️"
      :threshold="{ warning: 500, critical: 1000, max: 2000 }"
      :trend="{ direction: 'down', percent: 5, timeWindow: '5m' }"
      :sparkline-data="timeSeries.get('response_time')?.dataPoints"
      :loading="isLoading"
    />

    <!-- Memory Usage Card -->
    <MetricCard
      title="Memory Usage"
      :value="metrics?.memory_usage_mb ?? 0"
      unit="MB"
      icon="🧠"
      :threshold="{ warning: 300, critical: 400, max: 512 }"
      :sparkline-data="timeSeries.get('memory_usage')?.dataPoints"
      :loading="isLoading"
    />

    <!-- Active Connections Card -->
    <MetricCard
      title="Active Connections"
      :value="metrics?.active_connections ?? 0"
      unit=""
      icon="🔗"
      :threshold="{ warning: 100, critical: 150, max: 200 }"
      :sparkline-data="timeSeries.get('connections')?.dataPoints"
      :loading="isLoading"
    />

    <!-- Bytes/sec Card -->
    <MetricCard
      title="Network I/O"
      :value="(metrics?.bytes_in_sec ?? 0) + (metrics?.bytes_out_sec ?? 0)"
      unit="B/s"
      icon="📡"
      :trend="{ direction: 'up', percent: 8, timeWindow: '5m' }"
      :threshold="{ warning: 10000000, critical: 50000000, max: 100000000 }"
      :sparkline-data="timeSeries.get('bytes_transferred')?.dataPoints"
      :loading="isLoading"
    />

    <!-- Error Rate Card -->
    <MetricCard
      title="Error Rate"
      :value="metrics?.error_rate_percent ?? 0"
      unit="%"
      icon="⚠️"
      :threshold="{ warning: 1, critical: 5, max: 100 }"
      :sparkline-data="timeSeries.get('error_rate')?.dataPoints"
      :loading="isLoading"
    />
  </div>
</template>

<script setup lang="ts">
import { toRefs } from 'vue';
import type { SystemMetrics, TimeSeriesData } from '@/types/dashboard';
import MetricCard from './MetricCard.vue';

interface Props {
  metrics?: SystemMetrics | null;
  timeSeries?: Map<string, TimeSeriesData>;
  isLoading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  metrics: null,
  timeSeries: () => new Map(),
  isLoading: false,
});

const { metrics, timeSeries, isLoading } = toRefs(props);
</script>

<style scoped>
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
  padding: 20px;
  animation: fadeIn 0.3s ease-out;
}

/* Responsive breakpoints */
@media (max-width: 1920px) {
  .metrics-grid {
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
    padding: 20px;
  }
}

@media (max-width: 1366px) {
  .metrics-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    padding: 16px;
  }
}

@media (max-width: 768px) {
  .metrics-grid {
    grid-template-columns: 1fr;
    gap: 12px;
    padding: 12px;
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
