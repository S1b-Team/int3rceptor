import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { apiClient } from '../api/client'

export const useStatsStore = defineStore('stats', () => {
  // State
  const requests = ref(0)
  const memory = ref(0)
  const connections = ref(0)
  const uptime = ref(0)
  const isLoading = ref(false)
  const lastUpdate = ref<Date | null>(null)

  // Computed
  const successRate = computed(() => {
    // TODO: Calculate from actual traffic data
    return 98.5
  })

  const avgResponseTime = computed(() => {
    // TODO: Calculate from actual traffic data
    return 124
  })

  const activePlugins = computed(() => {
    // TODO: Get from plugins store
    return 3
  })

  // Actions
  async function fetchStats() {
    isLoading.value = true
    try {
      const data = await apiClient.getStats()
      requests.value = data.requests || 0
      memory.value = data.memory || 0
      connections.value = data.connections || 0
      uptime.value = data.uptime || 0
      lastUpdate.value = new Date()
    } catch (error) {
      console.error('Failed to fetch stats:', error)
    } finally {
      isLoading.value = false
    }
  }

  // Auto-refresh stats every 2 seconds
  function startAutoRefresh() {
    fetchStats()
    setInterval(fetchStats, 2000)
  }

  return {
    // State
    requests,
    memory,
    connections,
    uptime,
    isLoading,
    lastUpdate,

    // Computed
    successRate,
    avgResponseTime,
    activePlugins,

    // Actions
    fetchStats,
    startAutoRefresh,
  }
})
