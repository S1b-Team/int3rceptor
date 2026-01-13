<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { apiClient } from './api/client'
import Badge from './components/base/Badge.vue'
import Button from './components/base/Button.vue'
import ComparerView from './components/views/ComparerView.vue'
import DecoderView from './components/views/DecoderView.vue'
import ExportView from './components/views/ExportView.vue'
import IntruderView from './components/views/IntruderView.vue'
import PluginsView from './components/views/PluginsView.vue'
import RepeaterView from './components/views/RepeaterView.vue'
import ScannerView from './components/views/ScannerView.vue'
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

const recentRequests = ref<TrafficItem[]>([])

const tabs = [
  { id: 'dashboard', name: 'Dashboard', icon: '📊' },
  { id: 'traffic', name: 'Traffic', icon: '📡', module: 'NOWARU' },
  { id: 'websocket', name: 'WebSocket', icon: '🔌', module: 'VOIDWALKER' },
  { id: 'repeater', name: 'Repeater', icon: '🔁' },
  { id: 'intruder', name: 'Intruder', icon: '⚔️' },
  { id: 'scanner', name: 'Scanner', icon: '🔍' },
  { id: 'decoder', name: 'Decoder', icon: '🧩' },
  { id: 'comparer', name: 'Comparer', icon: '⚖️' },
  { id: 'plugins', name: 'Plugins', icon: '🔌' },
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

    // Fetch recent traffic
    const traffic = await apiClient.getTraffic(5)
    recentRequests.value = traffic
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

// Project Management
const showProjectModal = ref(false)
const projectInfo = ref({
  name: 'Untitled Project',
  description: '',
  path: ''
})

async function openProjectModal() {
  try {
    const info = await apiClient.projectInfo()
    projectInfo.value = { ...projectInfo.value, ...info }
    showProjectModal.value = true
  } catch (e) {
    console.error('Failed to load project info', e)
  }
}

async function handleNewProject() {
  const name = prompt('Enter new project name:')
  if (name) {
    try {
      await apiClient.projectNew(name)
      await openProjectModal()
      alert('New project created!')
    } catch (e) {
      alert('Failed to create project')
    }
  }
}

async function handleSaveProject() {
  if (!projectInfo.value.path) {
    const path = prompt('Enter file path to save (e.g., /tmp/audit.i3p):')
    if (!path) return
    projectInfo.value.path = path
  }

  try {
    // TODO: Get real scope from store
    await apiClient.projectSave(projectInfo.value.path, [])
    await apiClient.projectUpdate(projectInfo.value.name, projectInfo.value.description)
    alert('Project saved successfully!')
  } catch (e) {
    alert('Failed to save project: ' + e)
  }
}

async function handleLoadProject() {
  const path = prompt('Enter file path to load:')
  if (path) {
    try {
      await apiClient.projectLoad(path)
      projectInfo.value.path = path
      await openProjectModal()
      alert('Project loaded successfully!')
    } catch (e) {
      alert('Failed to load project: ' + e)
    }
  }
}
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
        <button @click="openProjectModal" class="ml-4 px-3 py-1 bg-i3-bg border border-i3-border rounded text-xs hover:border-i3-cyan hover:text-i3-cyan transition-colors flex items-center gap-2">
          <span>📁</span>
          <span>{{ projectInfo.name || 'Project' }}</span>
        </button>
      </div>

      <!-- Navigation Tabs -->
      <nav class="flex gap-1 overflow-x-auto no-scrollbar mask-linear-fade">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="currentTab = tab.id"
          :class="[
            'px-3 py-1 flex flex-col items-center justify-center transition-all duration-200 relative group min-w-fit',
            currentTab === tab.id
              ? 'text-i3-cyan'
              : 'text-i3-text-secondary hover:text-i3-text'
          ]"
        >
          <!-- Main Label (Module Name or Standard Name) -->
          <div class="flex items-center gap-2">
            <span class="text-lg opacity-80 group-hover:opacity-100 transition-opacity">{{ tab.icon }}</span>
            <span :class="[
              'font-bold tracking-wider whitespace-nowrap',
              tab.module ? 'font-heading text-base' : 'font-sans text-sm'
            ]">
              {{ tab.module || tab.name }}
            </span>
          </div>

          <!-- Subtitle (Functional Name for Modules) -->
          <span v-if="tab.module" class="text-[9px] uppercase tracking-[0.15em] text-i3-text-muted font-mono -mt-0.5">
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
                  <tr v-for="req in recentRequests" :key="req.id" class="hover:bg-i3-bg-alt/50 transition-colors">
                    <td class="py-3"><Badge :variant="req.method === 'GET' ? 'cyan' : req.method === 'POST' ? 'orange' : 'magenta'">{{ req.method }}</Badge></td>
                    <td class="py-3 text-i3-text truncate max-w-[200px]" :title="req.url">{{ req.url }}</td>
                    <td class="py-3"><Badge variant="cyan" outline>{{ req.status }}</Badge></td>
                    <td class="py-3 text-i3-text-secondary">{{ req.size }} B</td>
                  </tr>
                  <tr v-if="recentRequests.length === 0">
                    <td colspan="4" class="py-4 text-center text-i3-text-muted">No requests captured yet</td>
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

        <!-- Decoder View -->
        <DecoderView v-else-if="currentTab === 'decoder'" />

        <!-- Comparer View -->
        <ComparerView v-else-if="currentTab === 'comparer'" />

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

    <!-- Project Modal -->
    <div v-if="showProjectModal" class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div class="bg-i3-bg-alt border border-i3-border rounded-lg shadow-2xl w-[500px] max-w-full flex flex-col max-h-[90vh]">
        <div class="p-4 border-b border-i3-border flex justify-between items-center">
          <h2 class="text-lg font-bold text-i3-text">Project Management</h2>
          <button @click="showProjectModal = false" class="text-i3-text-muted hover:text-i3-text">✕</button>
        </div>
        <div class="p-6 space-y-4">
          <div>
            <label class="block text-xs font-bold text-i3-text-muted uppercase mb-1">Project Name</label>
            <input v-model="projectInfo.name" type="text" class="w-full bg-i3-bg border border-i3-border rounded px-3 py-2 text-i3-text focus:border-i3-cyan outline-none">
          </div>
          <div>
            <label class="block text-xs font-bold text-i3-text-muted uppercase mb-1">Description</label>
            <textarea v-model="projectInfo.description" class="w-full bg-i3-bg border border-i3-border rounded px-3 py-2 text-i3-text focus:border-i3-cyan outline-none h-24"></textarea>
          </div>
          <div>
            <label class="block text-xs font-bold text-i3-text-muted uppercase mb-1">File Path</label>
            <div class="flex gap-2">
              <input v-model="projectInfo.path" type="text" readonly class="flex-1 bg-i3-bg border border-i3-border rounded px-3 py-2 text-i3-text-muted outline-none cursor-not-allowed">
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4 pt-4 border-t border-i3-border">
            <Button variant="secondary" @click="handleNewProject">✨ New Project</Button>
            <Button variant="secondary" @click="handleLoadProject">📂 Load Project</Button>
            <Button variant="primary" @click="handleSaveProject" class="col-span-2">💾 Save Project</Button>
          </div>
        </div>
      </div>
    </div>
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
