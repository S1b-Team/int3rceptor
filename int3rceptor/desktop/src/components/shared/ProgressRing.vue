<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  value: number
  max?: number
  size?: number
  strokeWidth?: number
  color?: 'cyan' | 'magenta' | 'orange' | 'purple' | 'green'
  showLabel?: boolean
  label?: string
}

const props = withDefaults(defineProps<Props>(), {
  max: 100,
  size: 80,
  strokeWidth: 8,
  color: 'cyan',
  showLabel: false
})

const percentage = computed(() => {
  return Math.min((props.value / props.max) * 100, 100)
})

const radius = computed(() => {
  return (props.size - props.strokeWidth) / 2
})

const circumference = computed(() => {
  return 2 * Math.PI * radius.value
})

const strokeDasharray = computed(() => {
  return circumference.value
})

const strokeDashoffset = computed(() => {
  return circumference.value - (percentage.value / 100) * circumference.value
})

const colorClass = computed(() => {
  const colors = {
    cyan: 'text-i3-cyan',
    magenta: 'text-i3-magenta',
    orange: 'text-i3-orange',
    purple: 'text-i3-purple',
    green: 'text-green-500'
  }
  return colors[props.color]
})

const glowClass = computed(() => {
  const glows = {
    cyan: 'drop-shadow-cyan',
    magenta: 'drop-shadow-magenta',
    orange: 'drop-shadow-orange',
    purple: 'drop-shadow-purple',
    green: ''
  }
  return glows[props.color]
})
</script>

<template>
  <div class="progress-ring-container" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg
      class="progress-ring"
      :width="size"
      :height="size"
      viewBox="0 0 100 100"
    >
      <!-- Background Circle -->
      <circle
        class="progress-ring__circle-bg"
        :cx="50"
        :cy="50"
        :r="radius"
        fill="transparent"
        :stroke-width="strokeWidth"
      />

      <!-- Progress Circle -->
      <circle
        class="progress-ring__circle"
        :class="[colorClass, glowClass]"
        :cx="50"
        :cy="50"
        :r="radius"
        fill="transparent"
        :stroke-width="strokeWidth"
        :stroke-dasharray="strokeDasharray"
        :stroke-dashoffset="strokeDashoffset"
        stroke-linecap="round"
        transform="rotate(-90 50 50)"
      />
    </svg>

    <!-- Center Label -->
    <div v-if="showLabel" class="progress-ring__label">
      <span class="text-2xl font-bold text-i3-text font-mono">
        {{ Math.round(percentage) }}%
      </span>
      <span v-if="label" class="text-xs text-i3-text-muted block mt-1">
        {{ label }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.progress-ring-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.progress-ring {
  transform: rotate(-90deg);
}

.progress-ring__circle-bg {
  stroke: var(--color-bg-panel);
}

.progress-ring__circle {
  transition: stroke-dashoffset 0.5s ease-out;
}

.progress-ring__label {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
}

.drop-shadow-cyan {
  filter: drop-shadow(0 0 6px rgba(0, 212, 255, 0.4));
}

.drop-shadow-magenta {
  filter: drop-shadow(0 0 6px rgba(255, 0, 110, 0.4));
}

.drop-shadow-orange {
  filter: drop-shadow(0 0 6px rgba(255, 184, 0, 0.4));
}

.drop-shadow-purple {
  filter: drop-shadow(0 0 6px rgba(139, 92, 246, 0.4));
}
</style>
