import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useIntruderStore = defineStore('intruder', () => {
  const template = ref('')
  const targetUrl = ref('')

  function setTemplate(newTemplate: string, url: string = '') {
    template.value = newTemplate
    targetUrl.value = url
  }

  function clearTemplate() {
    template.value = ''
    targetUrl.value = ''
  }

  return {
    template,
    targetUrl,
    setTemplate,
    clearTemplate
  }
})
