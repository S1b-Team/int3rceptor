<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { apiClient } from '../../api/client'
import { useIntruderStore } from '../../stores/intruder'
import { useUiStore } from '../../stores/ui'
import Badge from '../base/Badge.vue'

const intruderStore = useIntruderStore()
const uiStore = useUiStore()

interface TrafficItem {
  id: string
  method: string
  url: string
  status: number
  size: number
  duration: number
  timestamp: string
}

const items = ref<TrafficItem[]>([])
const selectedId = ref<string | null>(null)
const selectedRequest = ref<any>(null)
const isLoading = ref(false)
let intervalId: number | undefined

// Fetch traffic from backend
async function fetchTraffic() {
  try {
    const data = await apiClient.getTraffic(50)
    items.value = data || []
  } catch (error) {
    console.error('Failed to fetch traffic', error)
  }
}

async function selectRequest(id: string) {
  selectedId.value = id
  selectedRequest.value = null // Clear previous
  try {
    const details = await apiClient.getRequest(id)
    selectedRequest.value = details
  } catch (error) {
    console.error('Failed to fetch request details', error)
  }
}

function sendToIntruder() {
  if (!selectedRequest.value) return

  const req = selectedRequest.value
  // Construct raw request string
  let raw = `${req.method} ${req.path || req.url} ${req.http_version || 'HTTP/1.1'}\n`

  if (req.headers) {
    for (const [key, value] of Object.entries(req.headers)) {
      raw += `${key}: ${value}\n`
    }
  }

  raw += '\n'

  if (req.body) {
    // If body is object, stringify it, otherwise use as is
    const bodyStr = typeof req.body === 'string' ? req.body : JSON.stringify(req.body, null, 2)
    raw += bodyStr
  }

  intruderStore.setTemplate(raw, req.url)
  uiStore.setTab('intruder')
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

onMounted(() => {
  fetchTraffic()
  intervalId = setInterval(fetchTraffic, 1000) as unknown as number
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Toolbar -->
    <div class="flex items-center gap-4 mb-4 p-2 bg-i3-bg-alt border border-i3-border rounded">
      <div class="relative flex-1">
        <span class="absolute left-3 top-1/2 -translate-y-1/2 text-i3-text-muted">🔍</span>
        <input
          type="text"
          placeholder="Filter requests (method, url, status)..."
          class="w-full bg-i3-bg border border-i3-border rounded py-1.5 pl-9 pr-4 text-sm focus:border-i3-cyan focus:outline-none transition-colors"
        >
      </div>
      <button class="p-2 hover:bg-i3-bg rounded text-i3-text-secondary hover:text-i3-cyan transition-colors" title="Clear">
        🗑️
      </button>
      <button class="p-2 hover:bg-i3-bg rounded text-i3-text-secondary hover:text-i3-cyan transition-colors" title="Pause">
        ⏸️
      </button>
    </div>

    <!-- Split View: List & Details -->
    <div class="flex-1 flex gap-4 overflow-hidden">

      <!-- Request List -->
      <div class="flex-1 panel p-0 overflow-hidden flex flex-col">
        <div class="overflow-auto flex-1">
          <table class="w-full text-sm text-left border-collapse">
            <thead class="sticky top-0 bg-i3-bg-alt z-10 text-xs uppercase text-i3-text-muted font-semibold">
              <tr>
                <th class="p-3 border-b border-i3-border w-16">#</th>
                <th class="p-3 border-b border-i3-border w-24">Method</th>
                <th class="p-3 border-b border-i3-border">URL</th>
                <th class="p-3 border-b border-i3-border w-20">Status</th>
                <th class="p-3 border-b border-i3-border w-24 text-right">Size</th>
                <th class="p-3 border-b border-i3-border w-24 text-right">Time</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-i3-border">
              <tr
                v-for="(req, index) in items"
                :key="req.id"
                @click="selectRequest(req.id)"
                :class="[
                  'cursor-pointer transition-colors hover:bg-i3-bg-panel',
                  selectedId === req.id ? 'bg-i3-cyan/10 border-l-2 border-l-i3-cyan' : 'border-l-2 border-l-transparent'
                ]"
              >
                <td class="p-3 text-i3-text-muted font-mono text-xs">{{ index + 1 }}</td>
                <td class="p-3">
                  <Badge :variant="req.method === 'GET' ? 'cyan' : req.method === 'POST' ? 'orange' : req.method === 'DELETE' ? 'magenta' : 'gray'">
                    {{ req.method }}
                  </Badge>
                </td>
                <td class="p-3 text-i3-text truncate max-w-[300px]" :title="req.url">{{ req.url }}</td>
                <td class="p-3">
                  <span :class="req.status < 300 ? 'text-i3-cyan' : req.status < 400 ? 'text-i3-orange' : 'text-i3-magenta'">
                    {{ req.status }}
                  </span>
                </td>
                <td class="p-3 text-right text-i3-text-secondary font-mono text-xs">{{ formatSize(req.size) }}</td>
                <td class="p-3 text-right text-i3-text-secondary font-mono text-xs">{{ formatDuration(req.duration) }}</td>
              </tr>

              <!-- Empty State -->
              <tr v-if="items.length === 0">
                <td colspan="6" class="p-8 text-center text-i3-text-muted">
                  <div class="mb-2 text-2xl">📡</div>
                  No traffic captured yet.<br>
                  <span class="text-xs opacity-70">Make sure your browser is configured to use the proxy.</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="p-2 border-t border-i3-border bg-i3-bg-alt text-xs text-i3-text-muted flex justify-between">
          <span>{{ items.length }} requests</span>
          <span>Auto-scroll: ON</span>
        </div>
      </div>

      <!-- Request Details (Placeholder for now) -->
      <div v-if="selectedId" class="w-1/3 panel flex flex-col">
        <div class="border-b border-i3-border pb-3 mb-3 flex justify-between items-center">
          <h3 class="font-bold text-i3-cyan">Request Details</h3>
          <div class="flex gap-2">
            <button @click="sendToIntruder" class="text-xs bg-i3-bg hover:bg-i3-bg-alt text-i3-cyan border border-i3-cyan/30 px-2 py-1 rounded transition-colors" title="Send to Intruder">
              🎯 Intruder
            </button>
            <button @click="selectedId = null" class="text-i3-text-muted hover:text-i3-text">✕</button>
          </div>
        </div>
        <div class="flex-1 overflow-auto text-xs font-mono text-i3-text-secondary space-y-4">
          <div>
            <div class="text-i3-text-muted mb-1">HEADERS</div>
            <div class="bg-i3-bg p-2 rounded border border-i3-border">
              Host: api.example.com<br>
              User-Agent: Mozilla/5.0...<br>
              Accept: application/json
            </div>
          </div>
          <div>
            <div class="text-i3-text-muted mb-1">BODY</div>
            <div class="bg-i3-bg p-2 rounded border border-i3-border text-i3-orange">
              { "data": "sample" }
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
