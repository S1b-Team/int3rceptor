import { ref, onUnmounted, type Ref } from 'vue'

interface ActivityEvent {
  timestamp: string
  type: 'request' | 'error' | 'warning' | 'info'
  message: string
  details?: any
}

interface UseDashboardActivityOptions {
  interval?: number
  limit?: number
}

export function useDashboardActivity(options: UseDashboardActivityOptions = {}) {
  const activityData = ref<ActivityEvent[]>([]) as Ref<ActivityEvent[]>
  const isLoading = ref(false) as Ref<boolean>
  const error = ref<Error | null>(null) as Ref<Error | null>
  const autoFetchTimer = ref<number | null>(null)

  const POLLING_INTERVAL = options.interval || 5000
  const FETCH_LIMIT = options.limit || 50

  const fetchActivity = async (): Promise<void> => {
    isLoading.value = true
    error.value = null

    try {
      const response = await fetch(`/api/dashboard/activity?limit=${FETCH_LIMIT}`)
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }
      
      const data: ActivityEvent[] = await response.json()
      activityData.value = data
    } catch (err) {
      error.value = err instanceof Error ? err : new Error('Failed to fetch activity data')
      console.error('Error fetching activity:', error.value)
    } finally {
      isLoading.value = false
    }
  }

  const startAutoFetch = (): void => {
    if (autoFetchTimer.value) {
      console.warn('Auto-fetch already running')
      return
    }

    fetchActivity() // Initial fetch
    
    autoFetchTimer.value = window.setInterval(() => {
      fetchActivity()
    }, POLLING_INTERVAL)
  }

  const stopAutoFetch = (): void => {
    if (autoFetchTimer.value) {
      clearInterval(autoFetchTimer.value)
      autoFetchTimer.value = null
    }
  }

  onUnmounted(() => {
    stopAutoFetch()
  })

  return {
    activityData,
    isLoading,
    error,
    startAutoFetch,
    stopAutoFetch,
    fetchActivity
  }
}
