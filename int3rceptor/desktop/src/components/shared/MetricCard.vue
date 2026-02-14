<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  title: string
  value: number | string
  unit?: string
  icon?: string
  trend?: {
    direction: 'up' | 'down' | 'stable'
    percent: number
  }
  threshold?: {
    warning: number
    critical: number
    max: number
  }
  loading?: boolean
  sparklineData?: number[]
}

const props = withDefaults(defineProps<Props>(), {
  unit: '',
  loading: false
})

const emit = defineEmits<{
  click: []
}>()

const statusClass = computed(() => {
  if (props.loading) return 'status-loading'
  if (!props.threshold) return ''

  const value = typeof props.value === 'number' ? props.value : 0
  const percent = (value / props.threshold.max) * 100

  if (percent >= props.threshold.critical / props.threshold.max * 100) {
    return 'status-critical'
  } else if (percent >= props.threshold.warning / props.threshold.max * 100) {
    return 'status-warning'
  }
  return 'status-normal'
})

const fillPercentage = computed(() => {
  if (!props.threshold || typeof props.value !== 'number') return 0
  return Math.min((props.value / props.threshold.max) * 100, 100)
})

const sparklinePoints = computed(() => {
  if (!props.sparklineData || props.sparklineData.length < 2) return ''

  const width = 100
  const height = 30
  const data = props.sparklineData
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1

  const points = data.map((value, index) => {
    const x = (index / (data.length - 1)) * width
    const y = height - ((value - min) / range) * height
    return `${x},${y}`
  }).join(' ')

  return points
})

const trendIcon = computed(() => {
  if (!props.trend) return ''
  switch (props.trend.direction) {
    case 'up': return '↑'
    case 'down': return '↓'
    case 'stable': return '→'
    default: return ''
  }
})

const trendColor = computed(() => {
  if (!props.trend) return ''
  switch (props.trend.direction) {
    case 'up': return 'text-i3-cyan'
    case 'down': return 'text-i3-magenta'
    case 'stable': return 'text-i3-text-muted'
    default: return ''
  }
})
</script>

<template>
  <div
    class="panel p-5 transition-all duration-300 hover:border-i3-cyan/50 cursor-pointer group"
    :class="statusClass"
    @click="emit('click')"
  >
    <!-- Header -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <span v-if="icon" class="text-lg text-i3-text-muted group-hover:text-i3-cyan transition-colors">
          {{ icon }}
        </span>
        <span class="text-sm text-i3-text-secondary font-medium uppercase tracking-wider">
          {{ title }}
        </span>
      </div>

      <!-- Trend Indicator -->
      <div v-if="trend && !loading" class="flex items-center gap-1">
        <span :class="['text-xs font-bold', trendColor]">
          {{ trendIcon }} {{ trend.percent }}%
        </span>
      </div>
    </div>

    <!-- Value Display -->
    <div class="flex items-baseline gap-2 mb-3">
      <span v-if="loading" class="text-3xl font-bold text-i3-text-muted animate-pulse">
        ...
      </span>
      <span v-else class="text-3xl font-bold text-i3-text font-mono">
        {{ value }}
      </span>
      <span v-if="unit" class="text-sm text-i3-text-muted">
        {{ unit }}
      </span>
    </div>

    <!-- Progress Bar -->
    <div v-if="threshold && !loading" class="relative">
      <div class="h-1.5 bg-i3-bg rounded-full overflow-hidden">
        <div
          class="h-full transition-all duration-500"
          :class="{
            'bg-i3-cyan': statusClass === 'status-normal',
            'bg-i3-orange': statusClass === 'status-warning',
            'bg-i3-magenta': statusClass === 'status-critical'
          }"
          :style="{ width: `${fillPercentage}%` }"
        ></div>
      </div>
    </div>

    <!-- Sparkline -->
    <div v-if="sparklineData && sparklineData.length >= 2 && !loading" class="mt-3">
      <svg width="100%" height="30" class="overflow-visible">
        <polyline
          :points="sparklinePoints"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="text-i3-cyan/50"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.status-normal {
  border-left: 3px solid var(--color-accent-cyan);
}

.status-warning {
  border-left: 3px solid var(--color-accent-orange);
}

.status-critical {
  border-left: 3px solid var(--color-accent-magenta);
}

.status-loading {
  opacity: 0.6;
}
</style>
