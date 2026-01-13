import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiClient, type PluginInfo } from '../api/client'

export const usePluginsStore = defineStore('plugins', () => {
  // State
  const plugins = ref<PluginInfo[]>([])
  const isLoading = ref(false)

  // Actions
  async function fetchPlugins() {
    isLoading.value = true
    try {
      const data = await apiClient.getPlugins()
      plugins.value = data
    } catch (error) {
      console.error('Failed to fetch plugins:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function reloadPlugin(name: string) {
    try {
      await apiClient.reloadPlugin(name)
      await fetchPlugins()
    } catch (error) {
      console.error('Failed to reload plugin:', error)
    }
  }

  async function togglePlugin(name: string, enabled: boolean) {
    try {
      await apiClient.togglePlugin(name, enabled)
      await fetchPlugins()
    } catch (error) {
      console.error('Failed to toggle plugin:', error)
    }
  }

  return {
    // State
    plugins,
    isLoading,

    // Actions
    fetchPlugins,
    reloadPlugin,
    togglePlugin,
  }
})
