<template>
  <div class="activity-chart">
    <!-- Chart Header -->
    <div class="chart-header">
      <h3 class="chart-title">📈 Request Activity (60s)</h3>
      <div class="chart-controls">
        <button
          v-for="period in ['1m', '5m', '15m']"
          :key="period"
          class="period-btn"
          :class="{ active: selectedPeriod === period }"
          @click="selectedPeriod = period"
        >
          {{ period }}
        </button>
      </div>
    </div>

    <!-- SVG Chart Container -->
    <div class="chart-container">
      <svg
        class="chart-svg"
        :viewBox="`0 0 ${chartWidth} ${chartHeight}`"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- Grid Lines -->
        <g class="grid-lines">
          <line
            v-for="i in 5"
            :key="`h-${i}`"
            :x1="0"
            :y1="(chartHeight / 5) * i"
            :x2="chartWidth"
            :y2="(chartHeight / 5) * i"
            stroke="rgba(255, 255, 255, 0.05)"
            stroke-width="1"
          />
          <line
            v-for="i in 12"
            :key="`v-${i}`"
            :x1="(chartWidth / 12) * i"
            :y1="0"
            :x2="(chartWidth / 12) * i"
            :y2="chartHeight"
            stroke="rgba(255, 255, 255, 0.05)"
            stroke-width="1"
          />
        </g>

        <!-- Y-Axis Labels -->
        <g class="y-axis-labels">
          <text
            v-for="i in 5"
            :key="`y-${i}`"
            :x="chartWidth - 8"
            :y="(chartHeight / 5) * (5 - i) + 4"
            text-anchor="end"
            font-size="10"
            fill="#606060"
          >
            {{ Math.round(maxValue * (i / 5)) }}
          </text>
        </g>

        <!-- Series Paths -->
        <g class="data-series">
          <!-- GET Requests (Cyan) -->
          <path
            v-if="getRequestsPath"
            :d="getRequestsPath"
            class="line get-requests"
            fill="none"
            stroke="#00d4ff"
            stroke-width="2"
            stroke-linejoin="round"
            stroke-linecap="round"
          />

          <!-- POST Requests (Magenta) -->
          <path
            v-if="postRequestsPath"
            :d="postRequestsPath"
            class="line post-requests"
            fill="none"
            stroke="#ff006e"
            stroke-width="2"
            stroke-linejoin="round"
            stroke-linecap="round"
          />

          <!-- Other Requests (Orange) -->
          <path
            v-if="otherRequestsPath"
            :d="otherRequestsPath"
            class="line other-requests"
            fill="none"
            stroke="#ffb800"
            stroke-width="2"
            stroke-linejoin="round"
            stroke-linecap="round"
          />
        </g>

        <!-- Data Points (circles) -->
        <g class="data-points">
          <circle
            v-for="(point, idx) in getRequests"
            :key="`get-${idx}`"
            :cx="(idx / (getRequests.length - 1 || 1)) * chartWidth"
            :cy="chartHeight - (point / maxValue) * chartHeight"
            r="3"
            fill="#00d4ff"
            opacity="0.6"
          />
          <circle
            v-for="(point, idx) in postRequests"
            :key="`post-${idx}`"
            :cx="(idx / (postRequests.length - 1 || 1)) * chartWidth"
            :cy="chartHeight - (point / maxValue) * chartHeight"
            r="3"
            fill="#ff006e"
            opacity="0.6"
          />
          <circle
            v-for="(point, idx) in otherRequests"
            :key="`other-${idx}`"
            :cx="(idx / (otherRequests.length - 1 || 1)) * chartWidth"
            :cy="chartHeight - (point / maxValue) * chartHeight"
            r="3"
            fill="#ffb800"
            opacity="0.6"
          />
        </g>
      </svg>
    </div>

    <!-- Legend -->
    <div class="chart-legend">
      <div class="legend-item">
        <span class="legend-color" style="background: #00d4ff;"></span>
        <span class="legend-label">GET</span>
        <span class="legend-value">{{ getRequests[getRequests.length - 1] ?? 0 }}/s</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #ff006e;"></span>
        <span class="legend-label">POST</span>
        <span class="legend-value">{{ postRequests[postRequests.length - 1] ?? 0 }}/s</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #ffb800;"></span>
        <span class="legend-label">Other</span>
        <span class="legend-value">{{ otherRequests[otherRequests.length - 1] ?? 0 }}/s</span>
      </div>
      <div class="legend-item">
        <span class="legend-label">Total</span>
        <span class="legend-value">
          {{ (getRequests[getRequests.length - 1] ?? 0) + (postRequests[postRequests.length - 1] ?? 0) + (otherRequests[otherRequests.length - 1] ?? 0) }}/s
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { TimeSeriesData } from '@/types/dashboard';

interface Props {
  timeSeries: Map<string, TimeSeriesData> | null;
}

const props = defineProps<Props>();

// State
const selectedPeriod = ref('1m');
const chartWidth = 600;
const chartHeight = 250;

// Sample data - in production, this would come from props.timeSeries
const getRequests = ref<number[]>([
  10, 15, 12, 18, 22, 20, 25, 28, 26, 32, 35, 30, 28, 25, 22, 20, 18, 16, 14, 12,
  10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 38, 36, 34, 32,
  30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2, 4, 6, 8, 10, 12,
]);

const postRequests = ref<number[]>([
  5, 8, 6, 10, 12, 11, 14, 16, 15, 18, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,
  5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35, 33, 31, 29, 27,
  25, 23, 21, 19, 17, 15, 13, 11, 9, 7, 5, 3, 1, 3, 5, 7, 9, 11, 13, 15,
]);

const otherRequests = ref<number[]>([
  3, 5, 4, 6, 8, 7, 9, 11, 10, 12, 14, 12, 10, 8, 6, 4, 2, 1, 0, 1,
  2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 16, 15, 14, 13,
  12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 7,
]);

// Computed
const maxValue = computed(() => {
  const allValues = [...getRequests.value, ...postRequests.value, ...otherRequests.value];
  return Math.max(...allValues, 50);
});

const getRequestsPath = computed(() => {
  if (getRequests.value.length === 0) return '';
  return generatePath(getRequests.value);
});

const postRequestsPath = computed(() => {
  if (postRequests.value.length === 0) return '';
  return generatePath(postRequests.value);
});

const otherRequestsPath = computed(() => {
  if (otherRequests.value.length === 0) return '';
  return generatePath(otherRequests.value);
});

// Helper function to generate SVG path
function generatePath(data: number[]): string {
  if (data.length < 2) return '';

  const points = data.map((value, idx) => {
    const x = (idx / (data.length - 1)) * chartWidth;
    const y = chartHeight - (value / maxValue.value) * chartHeight;
    return `${x},${y}`;
  });

  return `M${points.join(' L')}`;
}

// Watch for updates (in production, would listen to WebSocket)
watch(
  () => props.timeSeries,
  (newTimeSeries) => {
    if (newTimeSeries) {
      // Update getRequests, postRequests, otherRequests from timeSeries
      // This is where real data integration would happen
    }
  }
);
</script>

<style scoped>
.activity-chart {
  padding: 20px;
  background: rgba(26, 26, 46, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #ffffff;
}

.chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.chart-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.chart-controls {
  display: flex;
  gap: 8px;
}

.period-btn {
  padding: 6px 12px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #a0a0a0;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.period-btn:hover {
  border-color: rgba(0, 212, 255, 0.5);
  color: #00d4ff;
}

.period-btn.active {
  background: rgba(0, 212, 255, 0.2);
  border-color: #00d4ff;
  color: #00d4ff;
}

.chart-container {
  width: 100%;
  margin-bottom: 16px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 6px;
  padding: 12px;
  min-height: 280px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chart-svg {
  width: 100%;
  height: auto;
  filter: drop-shadow(0 0 4px rgba(0, 212, 255, 0.1));
}

.grid-lines line {
  shape-rendering: crispEdges;
}

.y-axis-labels text {
  font-family: 'Fira Code', monospace;
}

.data-series {
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.line {
  opacity: 0.8;
  transition: opacity 0.2s ease;
}

.line:hover {
  opacity: 1;
  stroke-width: 3;
}

.data-points circle {
  transition: all 0.2s ease;
  cursor: pointer;
}

.data-points circle:hover {
  r: 5;
  opacity: 1;
  filter: drop-shadow(0 0 4px currentColor);
}

/* Legend */
.chart-legend {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.legend-label {
  font-size: 12px;
  color: #a0a0a0;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.legend-value {
  font-size: 12px;
  color: #00d4ff;
  font-weight: 700;
  font-family: 'Fira Code', monospace;
  margin-left: 4px;
}

/* Responsive */
@media (max-width: 768px) {
  .activity-chart {
    padding: 16px;
  }

  .chart-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .chart-legend {
    gap: 12px;
  }

  .chart-container {
    min-height: 200px;
  }
}
</style>
