<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import Badge from './components/base/Badge.vue'
import Button from './components/base/Button.vue'
import ExportView from './components/views/ExportView.vue'
import IntruderView from './components/views/IntruderView.vue'
import PluginsView from './components/views/PluginsView.vue'
import RepeaterView from './components/views/RepeaterView.vue'
import SettingsView from './components/views/SettingsView.vue'
import TrafficView from './components/views/TrafficView.vue'
import WebSocketView from './components/views/WebSocketView.vue'

import { storeToRefs } from 'pinia'
import { useUiStore } from './stores/ui'

const uiStore = useUiStore()
const { currentTab } = storeToRefs(uiStore)

const stats = ref({
  requests: 0,
  memory: 0,
  connections: 0,
})

const tabs = [
  { id: 'dashboard', name: 'Dashboard', icon: '📊' },
  { id: 'traffic', name: 'Traffic', icon: '📡', module: 'NOWARU' },
  { id: 'websocket', name: 'WebSocket', icon: '🔌', module: 'VOIDWALKER' },
  { id: 'repeater', name: 'Repeater', icon: '🔁' },
  { id: 'intruder', name: 'Intruder', icon: '⚔️' },
  { id: 'scanner', name: 'Scanner', icon: '🔍' },
  { id: 'plugins', name: 'Plugins', icon: '🧩' },
]

// Simple backend connection
let intervalId: number | undefined

async function fetchStats() {
  try {
    const response = await fetch('http://localhost:3000/api/stats')
    if (response.ok) {
      const data = await response.json()
      stats.value.requests = data.requests || 0
      stats.value.memory = data.memory || 0
      stats.value.connections = data.connections || 0
    }
  } catch (error) {
    // Backend not ready yet
  }
}

function openSettings() {
  uiStore.setTab('settings')
}

function openExport() {
  uiStore.setTab('export')
}

function startProxy() {
  window.alert('Starting Proxy...')
}

onMounted(() => {
  fetchStats()
  intervalId = setInterval(fetchStats, 2000) as unknown as number
})

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId)
  }
})
</script>

<template>
  <div class="app-container hexagon-bg cyber-grid min-h-screen flex flex-col">
    <!-- Header -->
    <header class="h-16 bg-i3-bg-alt border-b border-i3-border flex items-center justify-between px-6">
      <!-- Logo & Title -->
      <div class="flex items-center gap-4">
        <div class="text-2xl font-heading font-bold cursor-pointer" @click="currentTab = 'dashboard'">
          <span class="text-i3-cyan">INT</span><span class="text-i3-magenta">3</span><span class="text-i3-cyan">RCEPTOR</span>
        </div>
        <div class="text-i3-text-muted text-sm">v3.0 Beta</div>
      </div>

      <!-- Navigation Tabs -->
      <nav class="flex gap-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="currentTab = tab.id"
          :class="[
            'px-4 py-1 flex flex-col items-center justify-center transition-all duration-200 relative group',
            currentTab === tab.id
              ? 'text-i3-cyan'
              : 'text-i3-text-secondary hover:text-i3-text'
          ]"
        >
          <!-- Main Label (Module Name or Standard Name) -->
          <div class="flex items-center gap-2">
            <span class="text-lg opacity-80 group-hover:opacity-100 transition-opacity">{{ tab.icon }}</span>
            <span :class="[
              'font-bold tracking-wider',
              tab.module ? 'font-heading text-lg' : 'font-sans text-sm'
            ]">
              {{ tab.module || tab.name }}
            </span>
          </div>

          <!-- Subtitle (Functional Name for Modules) -->
          <span v-if="tab.module" class="text-[10px] uppercase tracking-[0.2em] text-i3-text-muted font-mono -mt-1">
            {{ tab.name }}
          </span>

          <!-- Active Indicator -->
          <div
            v-if="currentTab === tab.id"
            class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-transparent via-i3-cyan to-transparent animate-pulse"
          />

          <!-- Hover Glow -->
          <div class="absolute inset-0 bg-i3-cyan/5 opacity-0 group-hover:opacity-100 transition-opacity rounded" />
        </button>
      </nav>

      <!-- Metrics Bar -->
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2">
          <span class="text-i3-text-muted text-sm">Requests:</span>
          <Badge variant="cyan">{{ stats.requests }}</Badge>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-i3-text-muted text-sm">Memory:</span>
          <Badge variant="orange">{{ stats.memory }} MB</Badge>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-i3-text-muted text-sm">Connections:</span>
          <Badge variant="magenta">{{ stats.connections }}</Badge>
        </div>
      </div>
    </header>

    <!-- Main Content Area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <aside class="w-64 bg-i3-bg-alt border-r border-i3-border flex flex-col p-4 gap-4">
        <!-- Primary Actions -->
        <div class="space-y-3">
          <Button variant="primary" class="w-full justify-center shadow-neon-cyan" @click="startProxy">
            <span class="mr-2">▶</span> Start Proxy
          </Button>
          <Button variant="secondary" class="w-full justify-center" @click="openSettings">
            <span class="mr-2">⚙️</span> Settings
          </Button>
          <Button variant="secondary" class="w-full justify-center" @click="openExport">
            <span class="mr-2">📄</span> Export
          </Button>
        </div>

        <!-- Quick Stats Panel -->
        <div class="panel mt-4 space-y-4">
          <h3 class="text-xs uppercase text-i3-text-muted font-bold tracking-wider">Quick Stats</h3>

          <div>
            <div class="flex justify-between text-sm mb-1">
              <span class="text-i3-text-secondary">Success Rate</span>
              <span class="text-i3-cyan">98.5%</span>
            </div>
            <div class="h-1.5 bg-i3-bg rounded-full overflow-hidden">
              <div class="h-full bg-i3-cyan w-[98.5%]"></div>
            </div>
          </div>

          <div>
            <div class="flex justify-between text-sm mb-1">
              <span class="text-i3-text-secondary">Avg Response</span>
              <span class="text-i3-orange">124ms</span>
            </div>
            <div class="h-1.5 bg-i3-bg rounded-full overflow-hidden">
              <div class="h-full bg-i3-orange w-[45%]"></div>
            </div>
          </div>

          <div>
            <div class="flex justify-between text-sm mb-1">
              <span class="text-i3-text-secondary">Active Plugins</span>
              <span class="text-i3-magenta">3</span>
            </div>
          </div>
        </div>
      </aside>

      <!-- View Content -->
      <main class="flex-1 p-6 overflow-hidden relative">
        <!-- Dashboard View -->
        <div v-if="currentTab === 'dashboard'" class="h-full flex flex-col gap-6 animate-fade-in">
          <!-- Welcome Hero -->
          <div class="panel bg-gradient-to-r from-i3-bg-alt to-i3-bg border-l-4 border-l-i3-cyan">
            <h1 class="text-3xl font-heading font-bold text-i3-cyan mb-2">Welcome to INT3RCEPTOR</h1>
            <p class="text-i3-text-secondary">The most advanced HTTP/HTTPS intercepting proxy built with Rust 🦀</p>
          </div>

          <!-- Feature Grid -->
          <div class="grid grid-cols-3 gap-6">
            <div class="panel hover:border-i3-orange transition-colors group">
              <div class="text-i3-orange text-xl mb-2 group-hover:scale-110 transition-transform origin-left">⚡ HTTP/2 Support</div>
              <p class="text-sm text-i3-text-muted">Full multiplexing and server push</p>
            </div>
            <div class="panel hover:border-i3-magenta transition-colors group">
              <div class="text-i3-magenta text-xl mb-2 group-hover:scale-110 transition-transform origin-left">🔌 WASM Plugins</div>
              <p class="text-sm text-i3-text-muted">Extend with any language</p>
            </div>
            <div class="panel hover:border-i3-cyan transition-colors group">
              <div class="text-i3-cyan text-xl mb-2 group-hover:scale-110 transition-transform origin-left">🚀 10x Faster</div>
              <p class="text-sm text-i3-text-muted">Rust-powered performance</p>
            </div>
          </div>

          <!-- Recent Requests Table (Placeholder) -->
          <div class="panel flex-1 flex flex-col">
            <h3 class="text-lg font-bold text-i3-cyan mb-4">Recent Requests</h3>
            <div class="flex-1 overflow-auto">
              <table class="w-full text-sm text-left">
                <thead class="text-xs uppercase text-i3-text-muted border-b border-i3-border">
                  <tr>
                    <th class="py-2">Method</th>
                    <th class="py-2">URL</th>
                    <th class="py-2">Status</th>
                    <th class="py-2">Size</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-i3-border">
                  <tr class="hover:bg-i3-bg-alt/50 transition-colors">
                    <td class="py-3"><Badge variant="cyan">GET</Badge></td>
                    <td class="py-3 text-i3-text">https://api.example.com/users</td>
                    <td class="py-3"><Badge variant="cyan" outline>200</Badge></td>
                    <td class="py-3 text-i3-text-secondary">1.2 KB</td>
                  </tr>
                  <tr class="hover:bg-i3-bg-alt/50 transition-colors">
                    <td class="py-3"><Badge variant="orange">POST</Badge></td>
                    <td class="py-3 text-i3-text">https://api.example.com/login</td>
                    <td class="py-3"><Badge variant="cyan" outline>201</Badge></td>
                    <td class="py-3 text-i3-text-secondary">512 B</td>
                  </tr>
                  <tr class="hover:bg-i3-bg-alt/50 transition-colors">
                    <td class="py-3"><Badge variant="orange">PUT</Badge></td>
                    <td class="py-3 text-i3-text">https://api.example.com/profile</td>
                    <td class="py-3"><Badge variant="magenta" outline>404</Badge></td>
                    <td class="py-3 text-i3-text-secondary">256 B</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Traffic View -->
        <TrafficView v-else-if="currentTab === 'traffic'" />

        <!-- WebSocket View -->
        <WebSocketView v-else-if="currentTab === 'websocket'" />

        <!-- Repeater View -->
        <RepeaterView v-else-if="currentTab === 'repeater'" />

        <!-- Intruder View -->
        <IntruderView v-else-if="currentTab === 'intruder'" />

        <!-- Scanner View -->
        <ScannerView v-else-if="currentTab === 'scanner'" />

        <!-- Settings View -->
        <SettingsView v-else-if="currentTab === 'settings'" />

        <!-- Export View -->
        <ExportView v-else-if="currentTab === 'export'" />

        <!-- Plugins View -->
        <PluginsView v-else-if="currentTab === 'plugins'" />

        <!-- Other Views (Placeholders) -->
        <div v-else class="h-full flex items-center justify-center flex-col text-i3-text-muted animate-fade-in">
          <div class="text-6xl mb-4 opacity-20">🚧</div>
          <h2 class="text-2xl font-bold mb-2">Work in Progress</h2>
          <p>The {{ currentTab }} module is coming soon.</p>
        </div>
      </main>
    </div>

    <!-- Footer -->
    <footer class="h-8 bg-i3-bg-alt border-t border-i3-border flex items-center justify-between px-6 text-xs">
      <div class="flex items-center gap-4">
        <span class="text-i3-text-muted">Status:</span>
        <span class="flex items-center gap-2">
          <span class="w-2 h-2 bg-i3-cyan rounded-full animate-pulse"></span>
          <span class="text-i3-cyan">Proxy Running</span>
        </span>
      </div>
      <div class="text-i3-text-muted">
        Powered by ⚙️ Rust · Vue 3 — crafted with precision by S1BGr0uP © 2025
      </div>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  font-variant-ligatures: common-ligatures;
}

table tbody tr {
  transition: all 0.15s cubic-bezier(0.4, 0.0, 0.2, 1);
}

table tbody tr:hover {
  border-left: 3px solid var(--color-accent-cyan);
}
</style>
