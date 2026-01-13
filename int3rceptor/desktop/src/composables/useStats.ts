import { ref, onMounted, onUnmounted } from 'vue'
import { apiClient } from '../api/client'

export function useStats() {
  const requests = ref(0)
  const memory = ref(0)
  const connections = ref(0)
  const isLoading = ref(false)
  let intervalId: number | null = null

  async function fetchStats() {
    try {
      const data = await apiClient.getStats()
      requests.value = data.requests || 0
      memory.value = data.memory || 0
      connections.value = data.connections || 0
    } catch (error) {
      console.error('Failed to fetch stats:', error)
    }
  }

  function startAutoRefresh() {
    fetchStats()
    intervalId = setInterval(fetchStats, 2000) as unknown as number
  }

  function stopAutoRefresh() {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  onMounted(() => {
    startAutoRefresh()
  })

  onUnmounted(() => {
    stopAutoRefresh()
  })

  return {
    requests,
    memory,
    connections,
    isLoading,
    fetchStats,
    startAutoRefresh,
    stopAutoRefresh,
  }
}
