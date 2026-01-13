<script setup lang="ts">
import { ref } from 'vue'
import Button from '../base/Button.vue'

const exportFormat = ref('har')
const exportScope = ref('all')
const includeBodies = ref(true)
const isExporting = ref(false)

function handleExport() {
  isExporting.value = true

  // Simulate export process
  setTimeout(() => {
    isExporting.value = false
    // Mock download
    const date = new Date().toISOString().split('T')[0]
    const filename = `int3rceptor_capture_${date}.${exportFormat.value}`
    alert(`Exported ${filename} successfully!`)
  }, 1500)
}
</script>

<template>
  <div class="h-full flex flex-col max-w-4xl mx-auto">
    <h2 class="text-2xl font-heading font-bold text-i3-cyan mb-6 flex items-center gap-3">
      <span class="text-3xl">📄</span> Export Data
    </h2>

    <div class="flex-1 overflow-auto pr-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">

        <!-- Format Selection -->
        <section class="panel bg-i3-bg-alt/50 h-full">
          <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">Export Format</h3>
          <div class="space-y-4">

            <label class="flex items-start gap-3 cursor-pointer group p-3 rounded border border-i3-border hover:border-i3-cyan transition-colors" :class="{ 'bg-i3-cyan/10 border-i3-cyan': exportFormat === 'har' }">
              <input v-model="exportFormat" type="radio" value="har" class="mt-1">
              <div>
                <div class="font-bold text-i3-text group-hover:text-i3-cyan">HTTP Archive (.har)</div>
                <div class="text-xs text-i3-text-secondary mt-1">Standard format compatible with Chrome DevTools, Charles, and other proxies.</div>
              </div>
            </label>

            <label class="flex items-start gap-3 cursor-pointer group p-3 rounded border border-i3-border hover:border-i3-orange transition-colors" :class="{ 'bg-i3-orange/10 border-i3-orange': exportFormat === 'json' }">
              <input v-model="exportFormat" type="radio" value="json" class="mt-1">
              <div>
                <div class="font-bold text-i3-text group-hover:text-i3-orange">Raw JSON (.json)</div>
                <div class="text-xs text-i3-text-secondary mt-1">Full internal representation of requests and responses. Best for programmatic analysis.</div>
              </div>
            </label>

            <label class="flex items-start gap-3 cursor-pointer group p-3 rounded border border-i3-border hover:border-i3-magenta transition-colors" :class="{ 'bg-i3-magenta/10 border-i3-magenta': exportFormat === 'csv' }">
              <input v-model="exportFormat" type="radio" value="csv" class="mt-1">
              <div>
                <div class="font-bold text-i3-text group-hover:text-i3-magenta">CSV Report (.csv)</div>
                <div class="text-xs text-i3-text-secondary mt-1">Summary of requests suitable for spreadsheets (Excel, Sheets).</div>
              </div>
            </label>

          </div>
        </section>

        <!-- Scope & Options -->
        <section class="panel bg-i3-bg-alt/50 h-full flex flex-col">
          <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">Options</h3>

          <div class="space-y-6 flex-1">
            <div class="space-y-2">
              <label class="text-sm text-i3-text-secondary font-bold">Scope</label>
              <select v-model="exportScope" class="w-full bg-white text-black border border-i3-border rounded p-2 focus:border-i3-cyan outline-none focus:ring-2 focus:ring-i3-cyan">
                <option value="all">All Captured Traffic</option>
                <option value="filtered">Current Filtered View</option>
                <option value="selected">Selected Requests Only</option>
              </select>
            </div>

            <div class="space-y-2">
              <label class="flex items-center gap-3 cursor-pointer group">
                <div class="relative">
                  <input v-model="includeBodies" type="checkbox" class="sr-only peer">
                  <div class="w-10 h-6 bg-i3-bg border border-i3-border rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:translate-x-4 peer-checked:bg-i3-cyan transition-all"></div>
                </div>
                <span class="text-i3-text group-hover:text-i3-cyan transition-colors">Include Response Bodies</span>
              </label>
              <p class="text-xs text-i3-text-muted ml-13">Uncheck to reduce file size significantly.</p>
            </div>
          </div>

          <div class="mt-auto pt-6">
             <div class="bg-i3-bg p-4 rounded border border-i3-border text-center">
                <div class="text-sm text-i3-text-secondary mb-1">Estimated File Size</div>
                <div class="text-2xl font-mono text-i3-cyan">~24.5 MB</div>
             </div>
          </div>
        </section>

      </div>
    </div>

    <!-- Actions -->
    <div class="mt-6 pt-4 border-t border-i3-border flex justify-end gap-4">
      <Button variant="secondary">Cancel</Button>
      <Button variant="primary" @click="handleExport" :disabled="isExporting">
        <span v-if="!isExporting" class="flex items-center gap-2">
          <span>⬇️</span> Export Data
        </span>
        <span v-else class="flex items-center gap-2">
          <span class="animate-spin">⏳</span> Exporting...
        </span>
      </Button>
    </div>
  </div>
</template>
