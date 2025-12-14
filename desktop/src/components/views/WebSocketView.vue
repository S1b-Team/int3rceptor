<script setup lang="ts">
import { ref } from 'vue'
import Badge from '../base/Badge.vue'

interface WebSocketConnection {
  id: string
  host: string
  path: string
  status: 'open' | 'closed'
  messagesCount: number
  timestamp: string
}

interface WebSocketMessage {
  id: string
  direction: 'outgoing' | 'incoming'
  type: 'text' | 'binary'
  payload: string
  length: number
  timestamp: string
}

// Mock Data for UI Dev
const connections = ref<WebSocketConnection[]>([
  { id: 'ws-1', host: 'ws.kraken.com', path: '/v2/ticker', status: 'open', messagesCount: 1450, timestamp: '10:42:15' },
  { id: 'ws-2', host: 'socket.chat.io', path: '/room/general', status: 'open', messagesCount: 89, timestamp: '10:45:30' },
  { id: 'ws-3', host: 'api.gaming.net', path: '/stream', status: 'closed', messagesCount: 12, timestamp: '10:12:00' },
])

const messages = ref<WebSocketMessage[]>([])
const selectedConnectionId = ref<string | null>(null)
const autoScroll = ref(true)

function selectConnection(id: string) {
  selectedConnectionId.value = id
  // Generate fake messages for demo
  messages.value = Array.from({ length: 20 }, (_, i) => ({
    id: `msg-${i}`,
    direction: Math.random() > 0.5 ? 'outgoing' : 'incoming',
    type: 'text',
    payload: Math.random() > 0.5 ? '{"event":"heartbeat"}' : '{"data":{"price": 45000.00}}',
    length: 45,
    timestamp: new Date().toLocaleTimeString()
  }))
}

function getStatusColor(status: string) {
  return status === 'open' ? 'text-i3-cyan' : 'text-i3-text-muted'
}
</script>

<template>
  <div class="h-full flex gap-4">

    <!-- Connections List (Left Panel) -->
    <div class="w-1/4 panel p-0 flex flex-col">
      <div class="p-3 border-b border-i3-border bg-i3-bg-alt font-bold text-i3-text flex justify-between items-center">
        <span>Active Connections</span>
        <Badge variant="cyan">{{ connections.filter(c => c.status === 'open').length }}</Badge>
      </div>
      <div class="flex-1 overflow-auto">
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
            <span class="font-bold text-i3-text truncate" :title="conn.host">{{ conn.host }}</span>
            <span :class="['text-xs font-mono', getStatusColor(conn.status)]">● {{ conn.status }}</span>
          </div>
          <div class="text-xs text-i3-text-secondary truncate font-mono mb-2">{{ conn.path }}</div>
          <div class="flex justify-between text-xs text-i3-text-muted">
            <span>{{ conn.timestamp }}</span>
            <span>{{ conn.messagesCount }} msgs</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Messages Stream (Right Panel) -->
    <div class="flex-1 panel p-0 flex flex-col relative overflow-hidden">

      <!-- Toolbar -->
      <div class="p-2 border-b border-i3-border bg-i3-bg-alt flex items-center gap-4">
        <div v-if="selectedConnectionId" class="flex-1 font-mono text-sm text-i3-cyan truncate">
          ws://{{ connections.find(c => c.id === selectedConnectionId)?.host }}{{ connections.find(c => c.id === selectedConnectionId)?.path }}
        </div>
        <div v-else class="flex-1 text-i3-text-muted text-sm italic">Select a connection to view stream</div>

        <div class="flex gap-2">
          <button class="p-1 hover:text-i3-cyan transition-colors" title="Clear">🗑️</button>
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
      <div v-if="selectedConnectionId" class="flex-1 overflow-auto p-4 space-y-2 font-mono text-xs bg-black/20">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex gap-4 group hover:bg-white/5 p-1 rounded"
        >
          <div class="w-20 text-i3-text-muted">{{ msg.timestamp }}</div>
          <div class="w-24 font-bold" :class="msg.direction === 'outgoing' ? 'text-i3-orange' : 'text-i3-cyan'">
            {{ msg.direction === 'outgoing' ? '→ CLIENT' : '← SERVER' }}
          </div>
          <div class="flex-1 break-all text-i3-text-secondary group-hover:text-i3-text transition-colors">
            {{ msg.payload }}
          </div>
          <div class="w-16 text-right text-i3-text-muted">{{ msg.length }} B</div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="flex-1 flex items-center justify-center flex-col text-i3-text-muted">
        <div class="text-4xl mb-4 opacity-30">🔌</div>
        <p>No connection selected</p>
      </div>

      <!-- Input Area (for sending messages) -->
      <div class="p-3 border-t border-i3-border bg-i3-bg-alt flex gap-2">
        <input
          type="text"
          placeholder="Send message to server..."
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
