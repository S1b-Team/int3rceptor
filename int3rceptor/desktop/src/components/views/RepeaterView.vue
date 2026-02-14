<script setup lang="ts">
import { ref, computed } from 'vue'
import { apiClient } from '../../api/client'
import Badge from '../base/Badge.vue'
import Icon from '../shared/Icon.vue'
import SplitView from '../shared/SplitView.vue'

const method = ref('GET')
const url = ref('https://api.example.com/v1/users')
const headersExpanded = ref(true)
const bodyExpanded = ref(true)
const requestHeaders = ref('User-Agent: INT3RCEPTOR/3.0\nAccept: application/json\nContent-Type: application/json')
const requestBody = ref('{\n  "query": "test"\n}')
const response = ref<null | {
  status: number
  statusText: string
  headers: Record<string, string>
  body: string
  time: number
  size: number
  headersCount: number
}>(null)

const responseTab = ref<'raw' | 'formatted'>('formatted')
const isSending = ref(false)

const parsedHeaders = computed(() => {
  const headers: Record<string, string> = {}
  requestHeaders.value.split('\n').forEach(line => {
    const [key, ...values] = line.split(':')
    if (key && values.length) {
      headers[key.trim()] = values.join(':').trim()
    }
  })
  return headers
})

const headersArray = computed(() => {
  return Object.entries(parsedHeaders.value).map(([key, value]) => ({ key, value }))
})

const canSend = computed(() => {
  return url.value.trim().length > 0
})

const getStatusColor = (status: number): string => {
  if (status >= 200 && status < 300) return 'text-i3-green'
  if (status >= 300 && status < 400) return 'text-i3-orange'
  if (status >= 400 && status < 500) return 'text-i3-magenta'
  if (status >= 500) return 'text-i3-red'
  return 'text-i3-text-muted'
}

const getStatusBadgeVariant = (status: number): 'cyan' | 'orange' | 'magenta' | 'red' => {
  if (status >= 200 && status < 300) return 'cyan'
  if (status >= 300 && status < 400) return 'orange'
  if (status >= 400 && status < 500) return 'magenta'
  return 'red'
}

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const formatTime = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

const formatJson = (jsonString: string): string => {
  try {
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return jsonString
  }
}

async function sendRequest() {
  if (!canSend.value) return

  isSending.value = true
  response.value = null

  try {
    const headers = parsedHeaders.value

    const res = await apiClient.sendRepeaterRequest({
      method: method.value,
      url: url.value,
      headers: headers,
      body: (method.value !== 'GET' && method.value !== 'HEAD') ? requestBody.value : undefined
    })

    response.value = {
      status: res.status,
      statusText: res.status_text,
      headers: res.headers,
      body: res.body,
      time: res.time_ms,
      size: res.size_bytes,
      headersCount: Object.keys(res.headers || {}).length
    }
  } catch (e: any) {
    response.value = {
      status: 0,
      statusText: 'ERROR',
      headers: {},
      body: `Error: ${e.message || 'Unknown error occurred'}\n\nCheck backend connection.`,
      time: 0,
      size: 0,
      headersCount: 0
    }
  } finally {
    isSending.value = false
  }
}

function addHeader() {
  const key = prompt('Enter header name:')
  if (!key) return
  const value = prompt('Enter header value:')
  if (value === null) return

  const newLine = `${key}: ${value}`
  requestHeaders.value += (requestHeaders.value.trim() ? '\n' : '') + newLine
}

function removeHeader(index: number) {
  const headers = headersArray.value
  headers.splice(index, 1)
  requestHeaders.value = headers.map(h => `${h.key}: ${h.value}`).join('\n')
}

function clearRequest() {
  url.value = ''
  requestHeaders.value = ''
  requestBody.value = ''
  response.value = null
}

function loadFromTraffic() {
  // TODO: Integrate with selected request from traffic tab
  url.value = 'https://api.example.com/v1/users'
  requestHeaders.value = 'User-Agent: INT3RCEPTOR/3.0\nAccept: application/json\nContent-Type: application/json'
  requestBody.value = '{\n  "query": "test"\n}'
}
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header Bar -->
    <div class="flex items-center gap-4">
      <div class="relative flex-1">
        <Icon name="search" size="sm" color="muted" class="absolute left-3 top-1/2 -translate-y-1/2" />
        <select
          v-model="method"
          class="w-full bg-i3-bg-alt border border-i3-border rounded-lg py-2.5 pl-10 pr-4 text-sm text-i3-text focus:border-i3-cyan focus:ring-1 focus:ring-i3-cyan/20 focus:outline-none transition-all duration-200"
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
          <option value="HEAD">HEAD</option>
          <option value="OPTIONS">OPTIONS</option>
        </select>
      </div>

      <div class="relative flex-1">
        <Icon name="link" size="sm" color="muted" class="absolute left-3 top-1/2 -translate-y-1/2" />
        <input
          v-model="url"
          type="text"
          placeholder="Enter URL..."
          class="w-full bg-i3-bg-alt border border-i3-border rounded-lg py-2.5 pl-10 pr-4 text-sm text-i3-text font-mono placeholder:text-i3-text-muted focus:border-i3-cyan focus:ring-1 focus:ring-i3-cyan/20 focus:outline-none transition-all duration-200"
        >
      </div>

      <button
        @click="loadFromTraffic"
        class="flex items-center gap-1.5 px-3 py-2.5 bg-i3-bg-alt border border-i3-border rounded-lg text-xs text-i3-text-secondary hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
      >
        <Icon name="upload" size="xs" />
        <span>Load from Traffic</span>
      </button>

      <button
        @click="clearRequest"
        class="p-2.5 rounded-lg bg-i3-bg-alt border border-i3-border text-i3-text-muted hover:border-i3-red hover:text-i3-red transition-all duration-200"
      >
        <Icon name="delete" size="sm" />
      </button>
    </div>

    <!-- Split View -->
    <SplitView
      orientation="horizontal"
      :min-size="20"
      :default-split="50"
      class="flex-1 overflow-hidden"
    >
      <!-- Request Panel (Left) -->
      <template #left>
        <div class="h-full flex flex-col bg-i3-bg-alt rounded-lg border border-i3-border overflow-hidden">
          <!-- Request Header -->
          <div class="flex items-center justify-between px-4 py-3 bg-i3-bg border-b border-i3-border">
            <div class="flex items-center gap-2">
              <Icon name="upload" size="sm" color="magenta" />
              <h3 class="text-sm font-bold text-i3-text">REQUEST</h3>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="sendRequest"
                :disabled="!canSend || isSending"
                :class="[
                  'flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm font-bold uppercase tracking-wider transition-all duration-200',
                  !canSend || isSending
                    ? 'bg-i3-bg border-i3-border text-i3-text-muted cursor-not-allowed'
                    : 'bg-gradient-to-r from-i3-cyan to-i3-cyan/80 text-i3-bg shadow-neon-cyan hover:shadow-glow-cyan-lg hover:-translate-y-0.5'
                ]"
              >
                <Icon v-if="isSending" name="spinner" size="sm" spin />
                <Icon v-else name="play" size="sm" />
                <span>{{ isSending ? 'Sending...' : 'Send Request' }}</span>
              </button>
            </div>
          </div>

          <!-- Request Content -->
          <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Headers Section -->
            <div class="flex flex-col border-b border-i3-border">
              <button
                @click="headersExpanded = !headersExpanded"
                class="flex items-center justify-between px-4 py-2 bg-i3-bg-panel/50 hover:bg-i3-bg-panel/80 transition-all duration-200"
              >
                <div class="flex items-center gap-2">
                  <Icon name="header" size="xs" color="magenta" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Headers</span>
                  <span class="text-xs text-i3-text-secondary">({{ headersArray.length }})</span>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    @click="addHeader"
                    class="p-1.5 rounded bg-i3-bg border border-i3-border text-i3-text-muted hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
                    title="Add header"
                  >
                    <Icon name="plus" size="xs" />
                  </button>
                  <Icon
                    :name="headersExpanded ? 'chevron-up' : 'chevron-down'"
                    size="xs"
                    color="muted"
                    class="transition-transform duration-200"
                  />
                </div>
              </button>

              <div v-if="headersExpanded" class="flex-1 overflow-auto">
                <div class="space-y-1 p-3">
                  <div
                    v-for="(header, idx) in headersArray"
                    :key="idx"
                    class="flex items-center gap-2 group"
                  >
                    <span class="flex-1 text-xs font-mono text-i3-cyan min-w-[120px]">{{ header.key }}:</span>
                    <span class="flex-1 text-xs font-mono text-i3-text break-all">{{ header.value }}</span>
                    <button
                      @click="removeHeader(idx)"
                      class="opacity-0 group-hover:opacity-100 p-1 rounded bg-i3-bg border border-i3-border text-i3-text-muted hover:border-i3-red hover:text-i3-red transition-all duration-200"
                      title="Remove header"
                    >
                      <Icon name="close" size="xs" />
                    </button>
                  </div>
                  <div v-if="headersArray.length === 0" class="text-center text-xs text-i3-text-muted py-4">
                    No headers. Click + to add one.
                  </div>
                </div>
              </div>
            </div>

            <!-- Body Section -->
            <div class="flex-1 flex flex-col">
              <button
                @click="bodyExpanded = !bodyExpanded"
                class="flex items-center justify-between px-4 py-2 bg-i3-bg-panel/50 hover:bg-i3-bg-panel/80 transition-all duration-200"
              >
                <div class="flex items-center gap-2">
                  <Icon name="code" size="xs" color="orange" />
                  <span class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Body</span>
                  <span class="text-xs text-i3-text-secondary">JSON</span>
                </div>
                <Icon
                  :name="bodyExpanded ? 'chevron-up' : 'chevron-down'"
                  size="xs"
                  color="muted"
                  class="transition-transform duration-200"
                />
              </button>

              <div v-if="bodyExpanded" class="flex-1 overflow-auto">
                <textarea
                  v-model="requestBody"
                  placeholder='{}'
                  class="w-full h-full bg-i3-bg p-3 text-sm font-mono text-i3-text placeholder:text-i3-text-muted focus:outline-none resize-none transition-colors"
                  spellcheck="false"
                ></textarea>
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- Response Panel (Right) -->
      <template #right>
        <div class="h-full flex flex-col bg-i3-bg-alt rounded-lg border border-i3-border overflow-hidden">
          <!-- Response Header -->
          <div v-if="response" class="flex items-center justify-between px-4 py-3 bg-i3-bg border-b border-i3-border">
            <div class="flex items-center gap-3">
              <Icon name="download" size="sm" color="green" />
              <h3 class="text-sm font-bold text-i3-text">RESPONSE</h3>
            </div>
            <Badge :variant="getStatusBadgeVariant(response.status)" size="lg">
              {{ response.status }} {{ response.statusText }}
            </Badge>
          </div>

          <!-- Empty State -->
          <div v-else class="h-full flex items-center justify-center text-i3-text-muted">
            <div class="text-center">
              <Icon name="empty" size="xl" color="muted" class="mb-3" />
              <div class="text-lg font-semibold">Ready to Send</div>
              <div class="text-sm text-i3-text-muted mt-1">Enter a URL and click Send Request</div>
            </div>
          </div>

          <!-- Response Content -->
          <div v-if="response" class="flex-1 flex flex-col overflow-hidden">
            <!-- Tabs -->
            <div class="flex items-center gap-1 px-4 py-2 bg-i3-bg-panel/50 border-b border-i3-border">
              <button
                @click="responseTab = 'raw'"
                :class="[
                  'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition-all duration-200',
                  responseTab === 'raw'
                    ? 'bg-i3-cyan/20 text-i3-cyan'
                    : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-panel/80'
                ]"
              >
                <Icon name="code" size="xs" />
                <span>Raw</span>
              </button>
              <button
                @click="responseTab = 'formatted'"
                :class="[
                  'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition-all duration-200',
                  responseTab === 'formatted'
                    ? 'bg-i3-cyan/20 text-i3-cyan'
                    : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-panel/80'
                ]"
              >
                <Icon name="format" size="xs" />
                <span>Formatted</span>
              </button>
            </div>

            <!-- Response Body -->
            <div class="flex-1 overflow-auto">
              <!-- Raw Tab -->
              <div v-if="responseTab === 'raw'" class="h-full">
                <pre class="w-full h-full p-4 text-sm font-mono text-i3-text whitespace-pre-wrap break-all">{{ response.body }}</pre>
              </div>

              <!-- Formatted Tab -->
              <div v-else class="h-full">
                <pre class="w-full h-full p-4 text-sm font-mono text-i3-text whitespace-pre-wrap break-all">{{ formatJson(response.body) }}</pre>
              </div>
            </div>
          </div>

          <!-- Response Footer -->
          <div v-if="response" class="flex items-center justify-between px-4 py-2 bg-i3-bg border-t border-i3-border text-xs">
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-2">
                <span class="text-i3-text-muted">Time:</span>
                <span class="font-mono text-i3-text">{{ formatTime(response.time) }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-i3-text-muted">Size:</span>
                <span class="font-mono text-i3-text">{{ formatSize(response.size) }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-i3-text-muted">Headers:</span>
                <span class="font-mono text-i3-text">{{ response.headersCount }}</span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </SplitView>
  </div>
</template>
