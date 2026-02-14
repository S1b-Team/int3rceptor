import { ref, onMounted, onUnmounted } from 'vue'
import { apiClient, type DashboardActivity, type ActivityQuery, type ActivityLevel } from '../api/client'

export function useDashboardActivity() {
  const activities = ref<DashboardActivity[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  let intervalId: number | null = null

  async function fetchActivity(query?: ActivityQuery) {
    try {
      isLoading.value = true
      error.value = null
      activities.value = await apiClient.getDashboardActivity(query)
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch activity'
      console.error('Failed to fetch activity:', err)
    } finally {
      isLoading.value = false
    }
  }

  async function clearActivity() {
    try {
      isLoading.value = true
      error.value = null
      await apiClient.clearDashboardActivity()
      activities.value = []
    } catch (err: any) {
      error.value = err.message || 'Failed to clear activity'
      console.error('Failed to clear activity:', err)
    } finally {
      isLoading.value = false
    }
  }

  function startAutoRefresh(intervalMs = 5000) {
    fetchActivity()
    intervalId = setInterval(fetchActivity, intervalMs) as unknown as number
  }

  function stopAutoRefresh() {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  function getLevelColor(level: ActivityLevel): string {
    switch (level) {
      case 'error':
        return 'text-red-500'
      case 'warning':
        return 'text-yellow-500'
      case 'info':
        return 'text-blue-500'
      case 'success':
        return 'text-green-500'
    }
  }

  function getLevelBadge(level: ActivityLevel): string {
    switch (level) {
      case 'error':
        return 'bg-red-500'
      case 'warning':
        return 'bg-yellow-500'
      case 'info':
        return 'bg-blue-500'
      case 'success':
        return 'bg-green-500'
    }
  }

  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000)
    return date.toLocaleString()
  }

  onMounted(() => {
    // Auto-refresh is not started by default
    // Call startAutoRefresh() if needed
  })

  onUnmounted(() => {
    stopAutoRefresh()
  })

  return {
    activities,
    isLoading,
    error,
    fetchActivity,
    clearActivity,
    startAutoRefresh,
    stopAutoRefresh,
    getLevelColor,
    getLevelBadge,
    formatTimestamp,
  }
}
