<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type WsConnection, type WsFrame } from '../../api/client'
import Badge from '../base/Badge.vue'

const connections = ref<WsConnection[]>([])
const frames = ref<WsFrame[]>([])
const selectedConnectionId = ref<string | null>(null)
const autoScroll = ref(true)
const loading = ref(false)
let pollingInterval: number | null = null

onMounted(async () => {
  await loadConnections()
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})

async function loadConnections() {
  try {
    connections.value = await apiClient.wsGetConnections()
  } catch (e) {
    console.error('Failed to load WebSocket connections', e)
  }
}

async function loadFrames(connectionId: string) {
  try {
    frames.value = await apiClient.wsGetFrames(connectionId)
  } catch (e) {
    console.error('Failed to load frames', e)
  }
}

async function selectConnection(id: string) {
  selectedConnectionId.value = id
  loading.value = true
  await loadFrames(id)
  loading.value = false
}

async function clearAll() {
  try {
    await apiClient.wsClear()
    connections.value = []
    frames.value = []
    selectedConnectionId.value = null
  } catch (e) {
    console.error('Failed to clear WebSocket data', e)
  }
}

function startPolling() {
  pollingInterval = window.setInterval(async () => {
    await loadConnections()
    if (selectedConnectionId.value) {
      await loadFrames(selectedConnectionId.value)
    }
  }, 2000)
}

function stopPolling() {
  if (pollingInterval) {
    clearInterval(pollingInterval)
    pollingInterval = null
  }
}

function getStatusColor(conn: WsConnection) {
  return conn.closed_at ? 'text-i3-text-muted' : 'text-i3-cyan'
}

function getStatusText(conn: WsConnection) {
  return conn.closed_at ? 'closed' : 'open'
}

function formatTimestamp(ts: number) {
  return new Date(ts * 1000).toLocaleTimeString()
}

function extractHost(url: string) {
  try {
    const u = new URL(url)
    return u.host
  } catch {
    return url
  }
}

function extractPath(url: string) {
  try {
    const u = new URL(url)
    return u.pathname + u.search
  } catch {
    return '/'
  }
}

function decodePayload(payload: number[]): string {
  try {
    return new TextDecoder().decode(new Uint8Array(payload))
  } catch {
    return `[Binary: ${payload.length} bytes]`
  }
}

const selectedConnection = computed(() => {
  return connections.value.find(c => c.id === selectedConnectionId.value)
})

const activeCount = computed(() => {
  return connections.value.filter(c => !c.closed_at).length
})
</script>

<template>
  <div class="h-full flex gap-4">

    <!-- Connections List (Left Panel) -->
    <div class="w-1/4 panel p-0 flex flex-col">
      <div class="p-3 border-b border-i3-border bg-i3-bg-alt font-bold text-i3-text flex justify-between items-center">
        <span>Active Connections</span>
        <Badge variant="cyan">{{ activeCount }}</Badge>
      </div>
      <div class="flex-1 overflow-auto">
        <div v-if="connections.length === 0" class="p-4 text-center text-i3-text-muted">
          No WebSocket connections captured yet
        </div>
        <div
          v-for="conn in connections"
          :key="conn.id"
          @click="selectConnection(conn.id)"
          :class="[
            'p-3 border-b border-i3-border cursor-pointer transition-colors hover:bg-i3-bg-panel',
            selectedConnectionId === conn.id ? 'bg-i3-cyan/10 border-l-2 border-l-i3-cyan' : 'border-l-2 border-l-transparent'
          ]"
        >
          <div class="flex justify-between items-start mb-1">
            <span class="font-bold text-i3-text truncate" :title="extractHost(conn.url)">{{ extractHost(conn.url) }}</span>
            <span :class="['text-xs font-mono', getStatusColor(conn)]">● {{ getStatusText(conn) }}</span>
          </div>
          <div class="text-xs text-i3-text-secondary truncate font-mono mb-2">{{ extractPath(conn.url) }}</div>
          <div class="flex justify-between text-xs text-i3-text-muted">
            <span>{{ formatTimestamp(conn.established_at) }}</span>
            <span>{{ conn.frames_count }} msgs</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Messages Stream (Right Panel) -->
    <div class="flex-1 panel p-0 flex flex-col relative overflow-hidden">

      <!-- Toolbar -->
      <div class="p-2 border-b border-i3-border bg-i3-bg-alt flex items-center gap-4">
        <div v-if="selectedConnection" class="flex-1 font-mono text-sm text-i3-cyan truncate">
          {{ selectedConnection.url }}
        </div>
        <div v-else class="flex-1 text-i3-text-muted text-sm italic">Select a connection to view stream</div>

        <div class="flex gap-2">
          <button @click="clearAll" class="p-1 hover:text-i3-cyan transition-colors" title="Clear All">🗑️</button>
          <button
            class="p-1 transition-colors"
            :class="autoScroll ? 'text-i3-cyan' : 'text-i3-text-muted'"
            @click="autoScroll = !autoScroll"
            title="Auto-scroll"
          >
            ⬇️
          </button>
        </div>
      </div>

      <!-- Stream View -->
      <div v-if="selectedConnectionId && frames.length > 0" class="flex-1 overflow-auto p-4 space-y-2 font-mono text-xs bg-black/20">
        <div
          v-for="frame in frames"
          :key="frame.id"
          class="flex gap-4 group hover:bg-white/5 p-1 rounded"
        >
          <div class="w-20 text-i3-text-muted">{{ formatTimestamp(frame.timestamp) }}</div>
          <div class="w-24 font-bold" :class="frame.direction === 'ClientToServer' ? 'text-i3-orange' : 'text-i3-cyan'">
            {{ frame.direction === 'ClientToServer' ? '→ CLIENT' : '← SERVER' }}
          </div>
          <div class="w-16 text-i3-text-muted">{{ frame.frame_type }}</div>
          <div class="flex-1 break-all text-i3-text-secondary group-hover:text-i3-text transition-colors">
            {{ decodePayload(frame.payload) }}
          </div>
          <div class="w-16 text-right text-i3-text-muted">{{ frame.payload.length }} B</div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-else-if="loading" class="flex-1 flex items-center justify-center">
        <div class="text-i3-cyan animate-pulse">Loading frames...</div>
      </div>

      <!-- Empty State -->
      <div v-else class="flex-1 flex items-center justify-center flex-col text-i3-text-muted">
        <div class="text-4xl mb-4 opacity-30">🔌</div>
        <p v-if="!selectedConnectionId">No connection selected</p>
        <p v-else>No messages in this connection</p>
      </div>

      <!-- Input Area (for sending messages) -->
      <div class="p-3 border-t border-i3-border bg-i3-bg-alt flex gap-2">
        <input
          type="text"
          placeholder="Send message to server... (not implemented yet)"
          class="flex-1 bg-i3-bg border border-i3-border rounded px-3 py-2 text-sm text-i3-text focus:border-i3-cyan outline-none font-mono"
          :disabled="!selectedConnectionId"
        >
        <button
          class="px-4 py-2 bg-i3-cyan/10 text-i3-cyan border border-i3-cyan/50 rounded hover:bg-i3-cyan/20 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
          :disabled="!selectedConnectionId"
        >
          Send
        </button>
      </div>

    </div>
  </div>
</template>
