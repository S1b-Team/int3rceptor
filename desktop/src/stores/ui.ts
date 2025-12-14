import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const currentTab = ref('dashboard')

  function setTab(tab: string) {
    currentTab.value = tab
  }

  return {
    currentTab,
    setTab
  }
})
