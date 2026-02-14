<template>
  <div class="metric-card" :class="statusClass" @click="handleClick">
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

    <!-- Loading overlay -->
    <div v-if="loading" class="loading-overlay">
      <div class="spinner"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { formatPercent, formatMetricValue } from '@/utils/dashboard/formatters';
import type { MetricPoint, MetricCardProps } from '@/types/dashboard';

interface Props extends MetricCardProps {
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
  onClick?: () => void;
}

const props = withDefaults(defineProps<Props>(), {
  icon: '',
  loading: false,
});

/**
 * Determine status class based on value and thresholds
 * Returns 'status-healthy', 'status-warning', or 'status-critical'
 */
const statusClass = computed(() => {
  if (!props.threshold) return 'status-healthy';

  const { value } = props;
  const { critical, warning } = props.threshold;

  if (value >= critical) return 'status-critical';
  if (value >= warning) return 'status-warning';
  return 'status-healthy';
});

/**
 * Calculate fill percentage for progress bar
 * Clamped to 0-100%
 */
const fillPercentage = computed(() => {
  if (!props.threshold) return 0;
  return Math.min((props.value / props.threshold.max) * 100, 100);
});

/**
 * Generate SVG points for sparkline chart
 * Maps data points to SVG coordinate space
 */
const sparklinePoints = computed(() => {
  if (!props.sparklineData || props.sparklineData.length === 0) return '';

  const data = props.sparklineData;
  const values = data.map(d => d.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;

  return data
    .map((point, i) => {
      const x = i * 4;
      const y = 40 - ((point.value - min) / range) * 40;
      return `${x},${y}`;
    })
    .join(' ');
});

/**
 * Handle card click event
 */
const handleClick = () => {
  if (props.onClick) {
    props.onClick();
  }
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
  cursor: pointer;
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
  flex-shrink: 0;
}

.title {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #a0a0a0;
  flex: 1;
  font-family: 'Inter', 'Roboto', sans-serif;
}

.trend {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
  flex-shrink: 0;
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
  font-family: 'Fira Code', 'JetBrains Mono', monospace;
  letter-spacing: -1px;
  line-height: 1;
}

.unit {
  font-size: 13px;
  color: #606060;
  font-weight: 500;
  font-family: 'Inter', 'Roboto', sans-serif;
  text-transform: uppercase;
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
  z-index: 10;
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
  to {
    transform: rotate(360deg);
  }
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .metric-card {
    padding: 16px;
  }

  .value {
    font-size: 28px;
  }

  .title {
    font-size: 11px;
  }

  .unit {
    font-size: 12px;
  }
}
</style>
