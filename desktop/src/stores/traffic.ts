import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiClient, type TrafficItem } from '../api/client'

export const useTrafficStore = defineStore('traffic', () => {
  // State
  const items = ref<TrafficItem[]>([])
  const selectedItem = ref<TrafficItem | null>(null)
  const isLoading = ref(false)
  const filter = ref('')

  // Actions
  async function fetchTraffic(limit = 100) {
    isLoading.value = true
    try {
      const data = await apiClient.getTraffic(limit)
      items.value = data
    } catch (error) {
      console.error('Failed to fetch traffic:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function getRequest(id: string) {
    try {
      const data = await apiClient.getRequest(id)
      selectedItem.value = data
      return data
    } catch (error) {
      console.error('Failed to fetch request:', error)
      return null
    }
  }

  async function clearAll() {
    try {
      await apiClient.clearTraffic()
      items.value = []
      selectedItem.value = null
    } catch (error) {
      console.error('Failed to clear traffic:', error)
    }
  }

  function selectItem(item: TrafficItem) {
    selectedItem.value = item
  }

  // Auto-refresh traffic every 1 second
  function startAutoRefresh() {
    fetchTraffic()
    setInterval(fetchTraffic, 1000)
  }

  return {
    // State
    items,
    selectedItem,
    isLoading,
    filter,

    // Actions
    fetchTraffic,
    getRequest,
    clearAll,
    selectItem,
    startAutoRefresh,
  }
})
