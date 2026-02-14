<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type TrafficItem } from '../../api/client'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'
import FeatureCard from '../shared/FeatureCard.vue'
import MetricCard from '../shared/MetricCard.vue'
import ProgressRing from '../shared/ProgressRing.vue'
import { useDashboardActivity } from '../../composables/useDashboardActivity'

const stats = ref({
  requests: 0,
  memory: 0,
  connections: 0,
})

const recentRequests = ref<TrafficItem[]>([])

// Dashboard Activity
const {
  activities,
  isLoading: isLoadingActivity,
  error: activityError,
  fetchActivity,
  clearActivity,
  getLevelColor,
  getLevelBadge,
  formatTimestamp,
} = useDashboardActivity()

// Simple backend connection
let intervalId: number | undefined

async function fetchStats() {
  try {
    const response = await fetch('http://localhost:3000/api/stats')
    if (response.ok) {
      const data = await response.json()
      stats.value.requests = data.requests || 0
      stats.value.memory = data.memory || 0
      stats.value.connections = data.connections || 0
    }

    // Fetch recent traffic
    const traffic = await apiClient.getTraffic(5)
    recentRequests.value = traffic
  } catch (error) {
    // Backend not ready yet
  }
}

function openSettings() {
  // Navigate to settings
  console.log('Open settings')
}

function startProxy() {
  window.alert('Starting Proxy...')
}

onMounted(() => {
  fetchStats()
  intervalId = setInterval(fetchStats, 2000) as unknown as number
  // Fetch activity data
  fetchActivity()
})

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId)
  }
})
</script>

<template>
  <div class="h-full flex flex-col gap-6 overflow-auto">
    <!-- Hero Section -->
    <div class="hero-section panel bg-gradient-cyan border-l-4 border-l-i3-cyan p-8">
      <h1 class="text-3xl font-heading font-bold text-i3-cyan mb-2">
        Welcome to INT3RCEPTOR
      </h1>
      <p class="text-lg text-i3-text-secondary">
        The most advanced HTTP/HTTPS intercepting proxy built with Rust 🦀
      </p>
    </div>

    <!-- Feature Cards Grid -->
    <div class="grid grid-cols-3 gap-6">
      <FeatureCard
        icon="⚡"
        title="HTTP/2 Support"
        description="Full multiplexing and server push"
        color="orange"
        :clickable="false"
      />
      <FeatureCard
        icon="🔌"
        title="WASM Plugins"
        description="Extend with any language"
        color="magenta"
        :clickable="false"
      />
      <FeatureCard
        icon="🚀"
        title="10x Faster"
        description="Rust-powered performance"
        color="cyan"
        :clickable="false"
      />
    </div>

    <!-- Metrics Grid -->
    <div class="grid grid-cols-3 gap-6">
      <MetricCard
        title="Requests/sec"
        :value="stats.requests"
        unit="req/s"
        icon="📊"
        :trend="{ direction: 'up', percent: 12 }"
        :threshold="{ warning: 100, critical: 200, max: 500 }"
      />
      <MetricCard
        title="Avg Response Time"
        :value="124"
        unit="ms"
        icon="⏱️"
        :trend="{ direction: 'down', percent: 8 }"
        :threshold="{ warning: 200, critical: 500, max: 1000 }"
      />
      <MetricCard
        title="Memory Usage"
        :value="stats.memory"
        unit="MB"
        icon="💾"
        :threshold="{ warning: 512, critical: 1024, max: 2048 }"
      />
      <MetricCard
        title="Active Connections"
        :value="stats.connections"
        unit="conn"
        icon="🔗"
        :threshold="{ warning: 50, critical: 100, max: 200 }"
      />
      <MetricCard
        title="Bytes/sec"
        :value="1250"
        unit="B/s"
        icon="📈"
        :trend="{ direction: 'stable', percent: 0 }"
      />
      <MetricCard
        title="Error Rate"
        :value="1.5"
        unit="%"
        icon="⚠️"
        :trend="{ direction: 'down', percent: 5 }"
        :threshold="{ warning: 3, critical: 5, max: 10 }"
      />
    </div>

    <!-- Two-Column Layout for Recent Requests and Activity Log -->
    <div class="flex-1 flex gap-6 min-h-0">
      <!-- Recent Requests -->
      <div class="flex-1 panel flex flex-col overflow-hidden">
        <div class="p-4 border-b border-i3-border">
          <h3 class="text-lg font-bold text-i3-cyan mb-4">Recent Requests</h3>
        </div>
        <div class="flex-1 overflow-auto">
          <table class="w-full text-sm text-left">
            <thead class="sticky top-0 bg-i3-bg-alt z-10 text-xs uppercase text-i3-text-muted border-b border-i3-border font-semibold">
              <tr>
                <th class="py-2">Method</th>
                <th class="py-2">URL</th>
                <th class="py-2">Status</th>
                <th class="py-2">Size</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-i3-border">
              <tr
                v-for="req in recentRequests"
                :key="req.id"
                class="hover:bg-i3-bg-alt/50 transition-colors cursor-pointer"
              >
                <td class="py-3">
                  <Badge :variant="req.method === 'GET' ? 'cyan' : req.method === 'POST' ? 'orange' : 'magenta'">
                    {{ req.method }}
                  </Badge>
                </td>
                <td class="py-3 text-i3-text truncate max-w-[200px]" :title="req.url">{{ req.url }}</td>
                <td class="py-3">
                  <Badge variant="cyan" outline>{{ req.status }}</Badge>
                </td>
                <td class="py-3 text-i3-text-secondary font-mono text-xs">{{ req.size }} B</td>
              </tr>
              <tr v-if="recentRequests.length === 0">
                <td colspan="4" class="py-8 text-center">
                  <div class="flex flex-col items-center justify-center text-i3-text-muted">
                    <div class="text-4xl mb-3 opacity-30">📡</div>
                    <p class="text-lg font-medium">No requests captured yet</p>
                    <p class="text-sm mt-2">Make sure your browser is configured to use the proxy.</p>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Activity Log -->
      <div class="flex-1 panel flex flex-col overflow-hidden">
        <div class="flex justify-between items-center p-4 border-b border-i3-border">
          <h3 class="text-lg font-bold text-i3-cyan">Activity Log</h3>
          <Button variant="secondary" size="sm" @click="clearActivity">
            🗑️ Clear
          </Button>
        </div>
        <div class="flex-1 overflow-auto">
          <div v-if="isLoadingActivity" class="flex items-center justify-center py-8 text-i3-text-muted">
            Loading activity...
          </div>
          <div v-else-if="activityError" class="flex items-center justify-center py-8 text-red-500">
            {{ activityError }}
          </div>
          <div v-else-if="activities.length === 0" class="flex items-center justify-center py-8 text-i3-text-muted">
            <div class="text-4xl mb-3 opacity-30">📋</div>
            <p>No activity recorded yet</p>
          </div>
          <div v-else class="space-y-2 p-4">
            <div
              v-for="activity in activities"
              :key="activity.timestamp"
              class="flex items-start gap-3 p-3 rounded hover:bg-i3-bg-alt/50 transition-colors border-l-2"
              :class="getLevelBadge(activity.level)"
            >
              <div class="flex-shrink-0 w-2 h-2 rounded-full mt-2" :class="getLevelBadge(activity.level)"></div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">
                    {{ activity.event_type }}
                  </span>
                  <span class="text-xs text-i3-text-muted">
                    {{ formatTimestamp(activity.timestamp) }}
                  </span>
                </div>
                <p class="text-sm text-i3-text break-words" :class="getLevelColor(activity.level)">
                  {{ activity.message }}
                </p>
                <div v-if="activity.details" class="mt-1 text-xs text-i3-text-muted">
                  <pre class="bg-i3-bg p-2 rounded overflow-x-auto">{{ JSON.stringify(activity.details, null, 2) }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Actions Bar -->
    <div class="panel p-4 flex gap-4">
      <Button variant="primary" class="flex-1 justify-center shadow-neon-cyan" @click="startProxy">
        <span class="mr-2">▶</span> Start Proxy
      </Button>
      <Button variant="secondary" class="flex-1 justify-center" @click="openSettings">
        <span class="mr-2">⚙️</span> Settings
      </Button>
      <Button variant="secondary" class="flex-1 justify-center">
        <span class="mr-2">📄</span> Export
      </Button>
    </div>
  </div>
</template>

<style scoped>
.hero-section {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.1), rgba(139, 92, 246, 0.1));
}

table tbody tr {
  transition: all 0.15s cubic-bezier(0.4, 0.0, 0.2, 1);
}

table tbody tr:hover {
  border-left: 3px solid var(--color-accent-cyan);
}
</style>
