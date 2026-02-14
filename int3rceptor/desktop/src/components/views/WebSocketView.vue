<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type WsConnection, type WsFrame } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'
import ProgressRing from '../shared/ProgressRing.vue'

const connections = ref<WsConnection[]>([])
const frames = ref<WsFrame[]>([])
const selectedConnectionId = ref<string | null>(null)
const autoScroll = ref(true)
const loading = ref(false)
const searchQuery = ref('')
const frameFilter = ref<'all' | 'client' | 'server' | 'text' | 'binary'>('all')
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

function getStatusColor(conn: WsConnection): string {
  return conn.closed_at ? 'text-i3-text-muted' : 'text-i3-cyan'
}

function getStatusText(conn: WsConnection): string {
  return conn.closed_at ? 'closed' : 'open'
}

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString()
}

function extractHost(url: string): string {
  try {
    const u = new URL(url)
    return u.host
  } catch {
    return url
  }
}

function extractPath(url: string): string {
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

function isBinary(payload: number[]): boolean {
  try {
    const text = new TextDecoder().decode(new Uint8Array(payload))
    return !/^[\x20-\x7E]*$/.test(text)
  } catch {
    return true
  }
}

const selectedConnection = computed(() => {
  return connections.value.find(c => c.id === selectedConnectionId.value)
})

const activeCount = computed(() => {
  return connections.value.filter(c => !c.closed_at).length
})

const filteredFrames = computed(() => {
  return frames.value.filter(frame => {
    const matchesSearch = searchQuery.value === '' ||
                         decodePayload(frame.payload).toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesFilter = frameFilter.value === 'all' ||
                         (frameFilter.value === 'client' && frame.direction === 'ClientToServer') ||
                         (frameFilter.value === 'server' && frame.direction === 'ServerToClient') ||
                         (frameFilter.value === 'text' && !isBinary(frame.payload)) ||
                         (frameFilter.value === 'binary' && isBinary(frame.payload))
    return matchesSearch && matchesFilter
  })
})

const frameStats = computed(() => {
  const clientFrames = frames.value.filter(f => f.direction === 'ClientToServer')
  const serverFrames = frames.value.filter(f => f.direction === 'ServerToClient')
  const textFrames = frames.value.filter(f => !isBinary(f.payload))
  const binaryFrames = frames.value.filter(f => isBinary(f.payload))
  const pingFrames = frames.value.filter(f => f.frame_type === 'Ping')
  const pongFrames = frames.value.filter(f => f.frame_type === 'Pong')

  return {
    clientSent: clientFrames.length,
    serverReceived: serverFrames.length,
    text: textFrames.length,
    binary: binaryFrames.length,
    ping: pingFrames.length,
    pong: pongFrames.length,
    total: frames.value.length
  }
})

const frameTypeDistribution = computed(() => {
  const stats = frameStats.value
  const total = stats.total || 1
  return {
    text: (stats.text / total) * 100,
    binary: (stats.binary / total) * 100
  }
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="activity" size="lg" color="cyan" />
        <h2 class="text-lg font-bold text-i3-text">WebSocket Interception</h2>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="cyan">{{ activeCount }} Active</Badge>
        <Badge variant="orange">{{ connections.length }} Total</Badge>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- Left Panel: Connections -->
      <div class="w-80 flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
            <Icon name="network" size="xs" />
            <span>Connections</span>
          </label>
        </div>
        <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg overflow-auto">
          <div v-if="connections.length === 0" class="text-center py-8 text-i3-text-muted">
            <Icon name="wifi-off" size="xl" color="muted" class="mb-2" />
            <p class="text-sm">No connections captured yet</p>
          </div>
          <div
            v-for="conn in connections"
            :key="conn.id"
            @click="selectConnection(conn.id)"
            :class="[
              'p-3 border-b border-i3-border cursor-pointer transition-all duration-200 hover:bg-i3-cyan/5',
              selectedConnectionId === conn.id
                ? 'bg-i3-cyan/10 border-l-2 border-l-i3-cyan'
                : 'border-l-2 border-l-transparent'
            ]"
          >
            <div class="flex items-start justify-between mb-2">
              <span class="font-bold text-i3-text truncate text-sm" :title="extractHost(conn.url)">
                {{ extractHost(conn.url) }}
              </span>
              <span :class="['text-[10px] font-bold uppercase', getStatusColor(conn)]">
                ● {{ getStatusText(conn) }}
              </span>
            </div>
            <div class="text-xs text-i3-text-muted truncate font-mono mb-2">
              {{ extractPath(conn.url) }}
            </div>
            <div class="flex items-center justify-between text-xs text-i3-text-muted">
              <div class="flex items-center gap-1">
                <Icon name="clock" size="xs" />
                <span>{{ formatTimestamp(conn.established_at) }}</span>
              </div>
              <div class="flex items-center gap-1">
                <Icon name="message-square" size="xs" />
                <span>{{ conn.frames_count }} msgs</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Center Panel: Frames -->
      <div class="flex-1 flex flex-col gap-2 min-w-0">
        <!-- Toolbar -->
        <div class="flex items-center gap-3 px-2">
          <div class="flex-1 relative">
            <Icon name="search" size="sm" class="absolute left-3 top-1/2 -translate-y-1/2 text-i3-text-muted" />
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search frames..."
              class="w-full bg-i3-bg border border-i3-border rounded-lg pl-10 pr-4 py-2 text-sm text-i3-text placeholder-i3-text-muted focus:border-i3-cyan focus:outline-none focus:ring-2 focus:ring-i3-cyan/20 transition-all duration-200"
            />
          </div>

          <!-- Frame Filter -->
          <div class="flex bg-i3-bg rounded-lg border border-i3-border overflow-hidden">
            <button
              @click="frameFilter = 'all'"
              :class="[
                'px-3 py-2 text-xs font-medium transition-all duration-200',
                frameFilter === 'all'
                  ? 'bg-i3-cyan text-i3-bg'
                  : 'text-i3-text-muted hover:bg-i3-bg-alt'
              ]"
            >
              All
            </button>
            <button
              @click="frameFilter = 'client'"
              :class="[
                'px-3 py-2 text-xs font-medium transition-all duration-200',
                frameFilter === 'client'
                  ? 'bg-i3-orange text-i3-bg'
                  : 'text-i3-text-muted hover:bg-i3-bg-alt'
              ]"
            >
              Client
            </button>
            <button
              @click="frameFilter = 'server'"
              :class="[
                'px-3 py-2 text-xs font-medium transition-all duration-200',
                frameFilter === 'server'
                  ? 'bg-i3-cyan text-i3-bg'
                  : 'text-i3-text-muted hover:bg-i3-bg-alt'
              ]"
            >
              Server
            </button>
            <button
              @click="frameFilter = 'text'"
              :class="[
                'px-3 py-2 text-xs font-medium transition-all duration-200',
                frameFilter === 'text'
                  ? 'bg-i3-green text-i3-bg'
                  : 'text-i3-text-muted hover:bg-i3-bg-alt'
              ]"
            >
              Text
            </button>
            <button
              @click="frameFilter = 'binary'"
              :class="[
                'px-3 py-2 text-xs font-medium transition-all duration-200',
                frameFilter === 'binary'
                  ? 'bg-i3-magenta text-i3-bg'
                  : 'text-i3-text-muted hover:bg-i3-bg-alt'
              ]"
            >
              Binary
            </button>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2">
            <Button
              @click="clearAll"
              variant="secondary"
              size="sm"
            >
              <Icon name="trash" size="xs" />
              <span>Clear</span>
            </Button>
          </div>
        </div>

        <!-- Connection Info -->
        <div v-if="selectedConnection" class="bg-i3-bg-alt border border-i3-border rounded-lg p-3">
          <div class="flex items-center gap-2 mb-2">
            <Icon name="link" size="sm" color="cyan" />
            <span class="text-sm font-bold text-i3-text truncate">{{ selectedConnection.url }}</span>
          </div>
          <div class="grid grid-cols-4 gap-4 text-xs">
            <div>
              <div class="text-i3-text-muted mb-1">Established</div>
              <div class="font-mono text-i3-text">{{ formatTimestamp(selectedConnection.established_at) }}</div>
            </div>
            <div>
              <div class="text-i3-text-muted mb-1">Status</div>
              <Badge :variant="selectedConnection.closed_at ? 'gray' : 'cyan'" outline size="sm">
                {{ selectedConnection.closed_at ? 'Closed' : 'Open' }}
              </Badge>
            </div>
            <div>
              <div class="text-i3-text-muted mb-1">Frames</div>
              <div class="font-mono text-i3-text">{{ selectedConnection.frames_count }}</div>
            </div>
            <div>
              <div class="text-i3-text-muted mb-1">Closed At</div>
              <div class="font-mono text-i3-text">
                {{ selectedConnection.closed_at ? formatTimestamp(selectedConnection.closed_at) : '-' }}
              </div>
            </div>
          </div>
        </div>

        <!-- Frames Table -->
        <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg overflow-auto">
          <div v-if="!selectedConnectionId" class="h-full flex flex-col items-center justify-center text-i3-text-muted">
            <Icon name="mouse-pointer" size="xl" color="muted" class="mb-2" />
            <p class="text-sm">Select a connection to view frames</p>
          </div>
          <div v-else-if="loading" class="h-full flex items-center justify-center">
            <div class="text-i3-cyan animate-pulse">Loading frames...</div>
          </div>
          <div v-else-if="filteredFrames.length === 0" class="h-full flex flex-col items-center justify-center text-i3-text-muted">
            <Icon name="inbox" size="xl" color="muted" class="mb-2" />
            <p class="text-sm">No frames found</p>
          </div>
          <table v-else class="w-full text-sm">
            <thead class="bg-i3-bg-alt sticky top-0">
              <tr>
                <th class="px-3 py-2 text-left text-xs font-medium text-i3-text-muted border-b border-i3-border w-24">
                  Timestamp
                </th>
                <th class="px-3 py-2 text-left text-xs font-medium text-i3-text-muted border-b border-i3-border w-24">
                  Direction
                </th>
                <th class="px-3 py-2 text-left text-xs font-medium text-i3-text-muted border-b border-i3-border w-20">
                  Type
                </th>
                <th class="px-3 py-2 text-left text-xs font-medium text-i3-text-muted border-b border-i3-border">
                  Payload
                </th>
                <th class="px-3 py-2 text-right text-xs font-medium text-i3-text-muted border-b border-i3-border w-20">
                  Size
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="frame in filteredFrames"
                :key="frame.id"
                :class="[
                  'hover:bg-i3-cyan/5 transition-colors border-b border-i3-border/30',
                  frame.direction === 'ClientToServer' ? 'border-l-2 border-l-i3-orange' : 'border-l-2 border-l-i3-cyan'
                ]"
              >
                <td class="px-3 py-2 font-mono text-i3-text-muted text-xs">
                  {{ formatTimestamp(frame.timestamp) }}
                </td>
                <td class="px-3 py-2">
                  <Badge
                    :variant="frame.direction === 'ClientToServer' ? 'orange' : 'cyan'"
                    outline
                    size="sm"
                  >
                    {{ frame.direction === 'ClientToServer' ? '→ Client' : '← Server' }}
                  </Badge>
                </td>
                <td class="px-3 py-2">
                  <Badge
                    :variant="frame.frame_type === 'Ping' ? 'purple' : frame.frame_type === 'Pong' ? 'green' : 'gray'"
                    outline
                    size="sm"
                  >
                    {{ frame.frame_type }}
                  </Badge>
                </td>
                <td class="px-3 py-2 font-mono text-i3-text-secondary break-all max-w-md">
                  {{ isBinary(frame.payload) ? `[Binary: ${frame.payload.length} bytes]` : decodePayload(frame.payload) }}
                </td>
                <td class="px-3 py-2 font-mono text-i3-text-muted text-xs text-right">
                  {{ frame.payload.length }} B
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Right Panel: Stats -->
      <div class="w-72 flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
            <Icon name="bar-chart-2" size="xs" />
            <span>Statistics</span>
          </label>
        </div>
        <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg p-4 overflow-auto space-y-4">
          <!-- Frame Type Distribution -->
          <div v-if="selectedConnectionId && frameStats.total > 0">
            <div class="text-sm font-bold text-i3-text mb-3">Frame Type Distribution</div>
            <div class="flex items-center justify-center mb-4">
              <ProgressRing
                :progress="frameTypeDistribution.text"
                :size="120"
                :stroke-width="12"
                variant="cyan"
              >
                <div class="text-center">
                  <div class="text-2xl font-bold text-i3-text">{{ frameStats.text }}</div>
                  <div class="text-xs text-i3-text-muted">Text</div>
                </div>
              </ProgressRing>
            </div>
            <div class="grid grid-cols-2 gap-2 text-xs">
              <div class="bg-i3-cyan/10 border border-i3-cyan/20 rounded-lg p-2 text-center">
                <div class="font-bold text-i3-cyan">{{ frameStats.text }}</div>
                <div class="text-i3-text-muted">Text Frames</div>
              </div>
              <div class="bg-i3-magenta/10 border border-i3-magenta/20 rounded-lg p-2 text-center">
                <div class="font-bold text-i3-magenta">{{ frameStats.binary }}</div>
                <div class="text-i3-text-muted">Binary Frames</div>
              </div>
            </div>
          </div>

          <!-- Frame Stats -->
          <div v-if="selectedConnectionId && frameStats.total > 0">
            <div class="text-sm font-bold text-i3-text mb-3">Frame Statistics</div>
            <div class="space-y-2">
              <div class="flex items-center justify-between p-2 bg-i3-bg-alt border border-i3-border rounded-lg">
                <div class="flex items-center gap-2">
                  <Icon name="arrow-up" size="xs" color="orange" />
                  <span class="text-xs text-i3-text-muted">Client → Server</span>
                </div>
                <span class="text-sm font-bold text-i3-orange">{{ frameStats.clientSent }}</span>
              </div>
              <div class="flex items-center justify-between p-2 bg-i3-bg-alt border border-i3-border rounded-lg">
                <div class="flex items-center gap-2">
                  <Icon name="arrow-down" size="xs" color="cyan" />
                  <span class="text-xs text-i3-text-muted">Server → Client</span>
                </div>
                <span class="text-sm font-bold text-i3-cyan">{{ frameStats.serverReceived }}</span>
              </div>
              <div class="flex items-center justify-between p-2 bg-i3-bg-alt border border-i3-border rounded-lg">
                <div class="flex items-center gap-2">
                  <Icon name="activity" size="xs" color="purple" />
                  <span class="text-xs text-i3-text-muted">Ping Frames</span>
                </div>
                <span class="text-sm font-bold text-i3-purple">{{ frameStats.ping }}</span>
              </div>
              <div class="flex items-center justify-between p-2 bg-i3-bg-alt border border-i3-border rounded-lg">
                <div class="flex items-center gap-2">
                  <Icon name="check-circle" size="xs" color="green" />
                  <span class="text-xs text-i3-text-muted">Pong Frames</span>
                </div>
                <span class="text-sm font-bold text-i3-green">{{ frameStats.pong }}</span>
              </div>
            </div>
          </div>

          <!-- Empty State -->
          <div v-if="!selectedConnectionId || frameStats.total === 0" class="text-center py-8 text-i3-text-muted">
            <Icon name="bar-chart" size="xl" color="muted" class="mb-2" />
            <p class="text-sm">Select a connection to view statistics</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
