<template>
  <div class="progress-ring-container">
    <!-- SVG Circle -->
    <svg class="progress-ring-svg" :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
      <!-- Background circle -->
      <circle
        class="progress-ring-background"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        fill="none"
        :stroke-width="strokeWidth"
      />
      <!-- Progress circle -->
      <circle
        class="progress-ring-progress"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        fill="none"
        :stroke-width="strokeWidth"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="strokeOffset"
        :style="{ '--status-color': statusColor }"
      />
    </svg>

    <!-- Center content -->
    <div class="progress-ring-content">
      <div class="progress-ring-value">{{ value }}%</div>
      <div class="progress-ring-label">{{ label }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  value: number; // 0-100
  label: string;
  size?: number;
  strokeWidth?: number;
  status?: 'healthy' | 'warning' | 'critical';
}

const props = withDefaults(defineProps<Props>(), {
  size: 120,
  strokeWidth: 4,
  status: 'healthy',
});

// Computed properties
const radius = computed(() => (props.size - props.strokeWidth) / 2);
const circumference = computed(() => 2 * Math.PI * radius.value);
const strokeOffset = computed(() => circumference.value - (props.value / 100) * circumference.value);

const statusColor = computed(() => {
  switch (props.status) {
    case 'critical':
      return '#ff006e';
    case 'warning':
      return '#ffb800';
    case 'healthy':
    default:
      return '#00d4ff';
  }
});
</script>

<style scoped>
.progress-ring-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.progress-ring-svg {
  transform: rotate(-90deg);
}

.progress-ring-background {
  stroke: rgba(255, 255, 255, 0.1);
}

.progress-ring-progress {
  stroke: var(--status-color, #00d4ff);
  transition: stroke-dashoffset 0.5s ease, stroke 0.3s ease;
  stroke-linecap: round;
}

.progress-ring-content {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.progress-ring-value {
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
  font-family: 'Fira Code', monospace;
}

.progress-ring-label {
  font-size: 11px;
  color: #a0a0a0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}
</style>
