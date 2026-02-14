<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { apiClient } from '../../api/client'
import { useIntruderStore } from '../../stores/intruder'
import { useUiStore } from '../../stores/ui'
import Badge from '../base/Badge.vue'
import Icon from '../shared/Icon.vue'
import TwoColumnLayout from '../shared/TwoColumnLayout.vue'

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

interface RequestDetails {
  id: string
  method: string
  url: string
  path: string
  http_version: string
  headers: Record<string, string>
  body: any
  status: number
  response_headers: Record<string, string>
  response_body: any
  timestamp: string
}

const items = ref<TrafficItem[]>([])
const selectedId = ref<string | null>(null)
const selectedRequest = ref<RequestDetails | null>(null)
const searchQuery = ref('')
const activeFilters = ref<string[]>([])
const detailsTab = ref<'headers' | 'body' | 'raw'>('headers')
let intervalId: number | undefined

const availableMethods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']

// Filter items based on search and method filters
const filteredItems = computed(() => {
  let result = items.value

  // Apply method filters
  if (activeFilters.value.length > 0) {
    result = result.filter(item => activeFilters.value.includes(item.method))
  }

  // Apply search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(item =>
      item.method.toLowerCase().includes(query) ||
      item.url.toLowerCase().includes(query) ||
      item.status.toString().includes(query)
    )
  }

  return result
})

// Toggle method filter
function toggleFilter(method: string) {
  const index = activeFilters.value.indexOf(method)
  if (index === -1) {
    activeFilters.value.push(method)
  } else {
    activeFilters.value.splice(index, 1)
  }
}

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
  detailsTab.value = 'headers' // Reset to first tab
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

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

function getStatusColor(status: number): string {
  if (status >= 200 && status < 300) return 'text-i3-green'
  if (status >= 300 && status < 400) return 'text-i3-orange'
  if (status >= 400 && status < 500) return 'text-i3-magenta'
  if (status >= 500) return 'text-i3-red'
  return 'text-i3-text-muted'
}

function getStatusBadgeVariant(status: number): 'cyan' | 'orange' | 'magenta' | 'red' {
  if (status >= 200 && status < 300) return 'cyan'
  if (status >= 300 && status < 400) return 'orange'
  if (status >= 400 && status < 500) return 'magenta'
  return 'red'
}

function getMethodBadgeVariant(method: string): 'cyan' | 'orange' | 'magenta' | 'purple' | 'gray' {
  const variants: Record<string, 'cyan' | 'orange' | 'magenta' | 'purple' | 'gray'> = {
    'GET': 'cyan',
    'POST': 'orange',
    'PUT': 'purple',
    'DELETE': 'magenta',
    'PATCH': 'purple',
    'HEAD': 'gray',
    'OPTIONS': 'gray'
  }
  return variants[method] || 'gray'
}

function clearTraffic() {
  items.value = []
  selectedId.value = null
  selectedRequest.value = null
}

let isPaused = false
function togglePause() {
  isPaused = !isPaused
  if (isPaused) {
    if (intervalId) clearInterval(intervalId)
  } else {
    intervalId = setInterval(fetchTraffic, 1000) as unknown as number
  }
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
  <div class="h-full flex flex-col gap-4">
    <!-- Toolbar with Search and Filters -->
    <div class="flex items-center gap-4">
      <!-- Search Bar -->
      <div class="relative flex-1">
        <Icon name="search" size="sm" color="muted" class="absolute left-3 top-1/2 -translate-y-1/2" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search requests (method, URL, status)..."
          class="w-full bg-i3-bg-alt border border-i3-border rounded-lg py-2.5 pl-10 pr-4 text-sm text-i3-text placeholder:text-i3-text-muted focus:border-i3-cyan focus:ring-1 focus:ring-i3-cyan/20 focus:outline-none transition-all duration-200"
        >
      </div>

      <!-- Filter Chips -->
      <div class="flex items-center gap-2">
        <button
          v-for="method in availableMethods"
          :key="method"
          @click="toggleFilter(method)"
          :class="[
            'px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition-all duration-200',
            activeFilters.includes(method)
              ? 'bg-i3-cyan/20 border border-i3-cyan text-i3-cyan shadow-glow-cyan'
              : 'bg-i3-bg border border-i3-border text-i3-text-secondary hover:border-i3-cyan/50 hover:text-i3-text'
          ]"
        >
          {{ method }}
        </button>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2">
        <button
          @click="togglePause"
          :title="isPaused ? 'Resume' : 'Pause'"
          class="p-2.5 rounded-lg bg-i3-bg-alt border border-i3-border text-i3-text-secondary hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
        >
          <Icon :name="isPaused ? 'play' : 'pause'" size="sm" />
        </button>
        <button
          @click="clearTraffic"
          title="Clear all traffic"
          class="p-2.5 rounded-lg bg-i3-bg-alt border border-i3-border text-i3-text-secondary hover:border-i3-red hover:text-i3-red transition-all duration-200"
        >
          <Icon name="delete" size="sm" />
        </button>
      </div>
    </div>

    <!-- Two Column Layout -->
    <TwoColumnLayout
      :left-panel-class="'flex flex-col overflow-hidden'"
      :right-panel-class="'flex flex-col overflow-hidden'"
      :show-right-panel="!!selectedId"
    >
      <!-- Left Panel: Traffic List -->
      <template #left>
        <div class="flex-1 flex flex-col overflow-hidden">
          <!-- Table Header -->
          <div class="flex items-center gap-2 px-4 py-3 bg-i3-bg-alt border-b border-i3-border">
            <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Traffic List</span>
            <span class="ml-auto text-xs text-i3-text-muted">{{ filteredItems.length }} requests</span>
          </div>

          <!-- Traffic Table -->
          <div class="flex-1 overflow-auto">
            <table class="w-full text-sm text-left border-collapse">
              <thead class="sticky top-0 bg-i3-bg-alt z-10 text-xs uppercase text-i3-text-muted font-semibold shadow-elevation-sm">
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
                  v-for="(req, index) in filteredItems"
                  :key="req.id"
                  @click="selectRequest(req.id)"
                  :class="[
                    'cursor-pointer transition-all duration-200',
                    selectedId === req.id
                      ? 'bg-i3-cyan/10 border-l-4 border-l-i3-cyan'
                      : 'hover:bg-i3-bg-panel/50 border-l-4 border-l-transparent'
                  ]"
                >
                  <td class="p-3 text-i3-text-muted font-mono text-xs">{{ index + 1 }}</td>
                  <td class="p-3">
                    <Badge :variant="getMethodBadgeVariant(req.method)">
                      {{ req.method }}
                    </Badge>
                  </td>
                  <td class="p-3 text-i3-text truncate max-w-[300px]" :title="req.url">{{ req.url }}</td>
                  <td class="p-3">
                    <Badge :variant="getStatusBadgeVariant(req.status)" outline>
                      {{ req.status }}
                    </Badge>
                  </td>
                  <td class="p-3 text-right text-i3-text-secondary font-mono text-xs">{{ formatSize(req.size) }}</td>
                  <td class="p-3 text-right text-i3-text-secondary font-mono text-xs">{{ formatDuration(req.duration) }}</td>
                </tr>

                <!-- Empty State -->
                <tr v-if="filteredItems.length === 0">
                  <td colspan="6" class="p-12 text-center">
                    <div class="flex flex-col items-center gap-4">
                      <Icon name="traffic" size="xl" color="muted" />
                      <div class="text-center">
                        <div class="text-lg font-semibold text-i3-text">No traffic captured yet</div>
                        <div class="text-sm text-i3-text-muted mt-1">
                          {{ items.length === 0
                            ? 'Configure your browser to use the proxy to start capturing traffic.'
                            : 'Adjust your filters or search query to see more results.'
                          }}
                        </div>
                      </div>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Footer -->
          <div class="px-4 py-2 border-t border-i3-border bg-i3-bg-alt text-xs text-i3-text-muted flex justify-between items-center">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 bg-i3-cyan rounded-full animate-pulse"></span>
              <span>Live: {{ isPaused ? 'Paused' : 'Active' }}</span>
            </div>
            <span>Auto-scroll: ON</span>
          </div>
        </div>
      </template>

      <!-- Right Panel: Request Details -->
      <template #right>
        <div v-if="selectedRequest" class="flex-1 flex flex-col overflow-hidden">
          <!-- Header -->
          <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border">
            <div class="flex items-center gap-2">
              <Icon name="request" size="sm" color="cyan" />
              <h3 class="font-bold text-i3-text">Request Details</h3>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="sendToIntruder"
                class="flex items-center gap-1.5 px-3 py-1.5 bg-i3-bg border border-i3-cyan/30 text-i3-cyan rounded-lg text-xs font-bold uppercase tracking-wider hover:bg-i3-cyan/10 hover:border-i3-cyan transition-all duration-200"
                title="Send to Intruder"
              >
                <Icon name="intruder" size="xs" />
                <span>Intruder</span>
              </button>
              <button
                @click="selectedId = null"
                class="p-2 rounded-lg text-i3-text-muted hover:text-i3-text hover:bg-i3-bg transition-all duration-200"
              >
                <Icon name="close" size="sm" />
              </button>
            </div>
          </div>

          <!-- Tabs -->
          <div class="flex items-center gap-1 px-4 py-2 bg-i3-bg border-b border-i3-border">
            <button
              v-for="tab in ['headers', 'body', 'raw'] as const"
              :key="tab"
              @click="detailsTab = tab"
              :class="[
                'px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all duration-200',
                detailsTab === tab
                  ? 'bg-i3-cyan/20 text-i3-cyan'
                  : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-alt'
              ]"
            >
              {{ tab }}
            </button>
          </div>

          <!-- Content -->
          <div class="flex-1 overflow-auto p-4">
            <!-- Headers Tab -->
            <div v-if="detailsTab === 'headers'" class="space-y-4">
              <!-- Request Headers -->
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="header" size="xs" color="magenta" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Request Headers</span>
                </div>
                <div class="bg-i3-bg-alt rounded-lg border border-i3-border overflow-hidden">
                  <table class="w-full text-xs">
                    <tbody class="divide-y divide-i3-border">
                      <tr v-for="(value, key) in selectedRequest.headers" :key="key">
                        <td class="px-3 py-2 font-mono text-i3-cyan w-1/3">{{ key }}</td>
                        <td class="px-3 py-2 font-mono text-i3-text">{{ value }}</td>
                      </tr>
                      <tr v-if="!selectedRequest.headers || Object.keys(selectedRequest.headers).length === 0">
                        <td colspan="2" class="px-3 py-4 text-center text-i3-text-muted">
                          No headers available
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>

              <!-- Response Headers -->
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="header" size="xs" color="green" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Response Headers</span>
                </div>
                <div class="bg-i3-bg-alt rounded-lg border border-i3-border overflow-hidden">
                  <table class="w-full text-xs">
                    <tbody class="divide-y divide-i3-border">
                      <tr v-for="(value, key) in selectedRequest.response_headers" :key="key">
                        <td class="px-3 py-2 font-mono text-i3-green w-1/3">{{ key }}</td>
                        <td class="px-3 py-2 font-mono text-i3-text">{{ value }}</td>
                      </tr>
                      <tr v-if="!selectedRequest.response_headers || Object.keys(selectedRequest.response_headers).length === 0">
                        <td colspan="2" class="px-3 py-4 text-center text-i3-text-muted">
                          No headers available
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>

            <!-- Body Tab -->
            <div v-else-if="detailsTab === 'body'" class="space-y-4">
              <!-- Request Body -->
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="upload" size="xs" color="magenta" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Request Body</span>
                </div>
                <div class="bg-i3-bg-alt rounded-lg border border-i3-border p-4">
                  <pre v-if="selectedRequest.body" class="text-xs font-mono text-i3-text whitespace-pre-wrap break-all">{{ typeof selectedRequest.body === 'string' ? selectedRequest.body : JSON.stringify(selectedRequest.body, null, 2) }}</pre>
                  <div v-else class="text-xs text-i3-text-muted italic">No request body</div>
                </div>
              </div>

              <!-- Response Body -->
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="download" size="xs" color="green" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Response Body</span>
                </div>
                <div class="bg-i3-bg-alt rounded-lg border border-i3-border p-4">
                  <pre v-if="selectedRequest.response_body" class="text-xs font-mono text-i3-text whitespace-pre-wrap break-all">{{ typeof selectedRequest.response_body === 'string' ? selectedRequest.response_body : JSON.stringify(selectedRequest.response_body, null, 2) }}</pre>
                  <div v-else class="text-xs text-i3-text-muted italic">No response body</div>
                </div>
              </div>
            </div>

            <!-- Raw Tab -->
            <div v-else-if="detailsTab === 'raw'" class="space-y-4">
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="code" size="xs" color="cyan" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Raw Request</span>
                </div>
                <div class="bg-i3-bg-alt rounded-lg border border-i3-border p-4">
                  <pre class="text-xs font-mono text-i3-text whitespace-pre-wrap break-all">{{ selectedRequest.method }} {{ selectedRequest.path || selectedRequest.url }} {{ selectedRequest.http_version || 'HTTP/1.1' }}
{{ Object.entries(selectedRequest.headers || {}).map(([k, v]) => `${k}: ${v}`).join('\n') }}

{{ typeof selectedRequest.body === 'string' ? selectedRequest.body : JSON.stringify(selectedRequest.body, null, 2) }}</pre>
                </div>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="px-4 py-3 border-t border-i3-border bg-i3-bg-alt text-xs text-i3-text-muted flex justify-between items-center">
            <div class="flex items-center gap-4">
              <span>Status: <span :class="getStatusColor(selectedRequest.status)">{{ selectedRequest.status }}</span></span>
              <span>Time: {{ formatDuration(selectedRequest.duration) }}</span>
              <span>Size: {{ formatSize(selectedRequest.size) }}</span>
            </div>
            <span>{{ formatTimestamp(selectedRequest.timestamp) }}</span>
          </div>
        </div>
      </template>
    </TwoColumnLayout>
  </div>
</template>
