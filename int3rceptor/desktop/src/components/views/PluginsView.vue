<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { apiClient } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'
import FeatureCard from '../shared/FeatureCard.vue'

interface Plugin {
  id: string
  name: string
  description: string
  version: string
  author: string
  icon: string
  category: 'security' | 'proxy' | 'extender' | 'intruder' | 'scanner' | 'decoder' | 'utility'
  enabled: boolean
  installed: boolean
  rating: number
  downloads: number
  hooks: string[]
  tags: string[]
}

const plugins = ref<Plugin[]>([])
const searchQuery = ref('')
const selectedCategory = ref<string>('all')
const showInstalledOnly = ref(false)
const selectedPlugin = ref<Plugin | null>(null)
const showDetailsModal = ref(false)

// Mock marketplace data (in real app, this would come from API)
const marketplacePlugins: Plugin[] = [
  {
    id: 'security-analyzer',
    name: 'Security Analyzer',
    description: 'Advanced security analysis with OWASP Top 10 detection, SQL injection patterns, XSS vulnerability scanning, and automated risk assessment.',
    version: '2.1.0',
    author: 'INT3RCEPTOR Team',
    icon: 'shield',
    category: 'security',
    enabled: false,
    installed: false,
    rating: 4.8,
    downloads: 12500,
    hooks: ['onRequest', 'onResponse', 'onWebSocket'],
    tags: ['security', 'owasp', 'vulnerability']
  },
  {
    id: 'http-proxy',
    name: 'HTTP Proxy',
    description: 'Full HTTP/HTTPS proxy with SSL/TLS interception, request/response modification, certificate management, and transparent proxy support.',
    version: '1.5.2',
    author: 'INT3RCEPTOR Team',
    icon: 'network',
    category: 'proxy',
    enabled: true,
    installed: true,
    rating: 4.9,
    downloads: 25000,
    hooks: ['onRequest', 'onResponse', 'onConnect'],
    tags: ['proxy', 'http', 'https', 'ssl']
  },
  {
    id: 'extender-api',
    name: 'Extender API',
    description: 'Powerful extension API for custom plugins, WASM runtime support, event hooks, and integration with external security tools.',
    version: '3.0.1',
    author: 'INT3RCEPTOR Team',
    icon: 'code',
    category: 'extender',
    enabled: true,
    installed: true,
    rating: 4.7,
    downloads: 8500,
    hooks: ['onPluginLoad', 'onPluginUnload', 'onCommand'],
    tags: ['api', 'wasm', 'extension']
  },
  {
    id: 'turbo-intruder',
    name: 'Turbo Intruder',
    description: 'High-performance automated attack tool with multi-threading, custom payload generation, timing analysis, and advanced attack vectors.',
    version: '1.8.0',
    author: 'INT3RCEPTOR Team',
    icon: 'zap',
    category: 'intruder',
    enabled: false,
    installed: false,
    rating: 4.6,
    downloads: 10200,
    hooks: ['onAttack', 'onPayload', 'onResult'],
    tags: ['intruder', 'attack', 'automation']
  },
  {
    id: 'sql-injector',
    name: 'SQL Injector',
    description: 'Automated SQL injection testing with blind SQL detection, time-based attacks, error-based injection, and comprehensive payload library.',
    version: '2.3.0',
    author: 'INT3RCEPTOR Team',
    icon: 'database',
    category: 'intruder',
    enabled: false,
    installed: false,
    rating: 4.5,
    downloads: 7800,
    hooks: ['onRequest', 'onResponse'],
    tags: ['sql', 'injection', 'database']
  },
  {
    id: 'xss-scanner',
    name: 'XSS Scanner',
    description: 'Cross-site scripting vulnerability scanner with DOM XSS detection, reflected XSS patterns, stored XSS analysis, and payload fuzzing.',
    version: '1.9.2',
    author: 'INT3RCEPTOR Team',
    icon: 'alert-triangle',
    category: 'scanner',
    enabled: false,
    installed: false,
    rating: 4.4,
    downloads: 6200,
    hooks: ['onRequest', 'onResponse', 'onWebSocket'],
    tags: ['xss', 'dom', 'security']
  },
  {
    id: 'base64-decoder',
    name: 'Base64 Decoder',
    description: 'Base64 encoding/decoding with multiple variants, URL-safe encoding, automatic detection, and batch processing support.',
    version: '1.2.0',
    author: 'INT3RCEPTOR Team',
    icon: 'file-code',
    category: 'decoder',
    enabled: false,
    installed: false,
    rating: 4.3,
    downloads: 4500,
    hooks: ['onDecode', 'onEncode'],
    tags: ['base64', 'encoding', 'decoder']
  },
  {
    id: 'url-decoder',
    name: 'URL Decoder',
    description: 'URL encoding/decoding with Unicode support, plus encoding, query string parsing, and recursive decoding.',
    version: '1.1.5',
    author: 'INT3RCEPTOR Team',
    icon: 'link',
    category: 'decoder',
    enabled: false,
    installed: false,
    rating: 4.2,
    downloads: 3800,
    hooks: ['onDecode', 'onEncode'],
    tags: ['url', 'encoding', 'decoder']
  },
  {
    id: 'jwt-analyzer',
    name: 'JWT Analyzer',
    description: 'JWT token analysis with signature verification, payload decoding, token validation, and security best practices checking.',
    version: '1.4.0',
    author: 'INT3RCEPTOR Team',
    icon: 'key',
    category: 'security',
    enabled: false,
    installed: false,
    rating: 4.6,
    downloads: 5600,
    hooks: ['onRequest', 'onResponse'],
    tags: ['jwt', 'token', 'security']
  },
  {
    id: 'websocket-monitor',
    name: 'WebSocket Monitor',
    description: 'Real-time WebSocket monitoring with frame capture, message inspection, connection tracking, and binary data visualization.',
    version: '1.3.0',
    author: 'INT3RCEPTOR Team',
    icon: 'activity',
    category: 'proxy',
    enabled: false,
    installed: false,
    rating: 4.5,
    downloads: 4200,
    hooks: ['onWebSocket', 'onFrame', 'onMessage'],
    tags: ['websocket', 'monitor', 'real-time']
  }
]

const categories = [
  { id: 'all', name: 'All', icon: 'grid' },
  { id: 'security', name: 'Security', icon: 'shield' },
  { id: 'proxy', name: 'Proxy', icon: 'network' },
  { id: 'extender', name: 'Extender', icon: 'code' },
  { id: 'intruder', name: 'Intruder', icon: 'zap' },
  { id: 'scanner', name: 'Scanner', icon: 'search' },
  { id: 'decoder', name: 'Decoder', icon: 'file-code' },
  { id: 'utility', name: 'Utility', icon: 'tool' }
]

onMounted(async () => {
  await loadPlugins()
})

async function loadPlugins() {
  try {
    const data = await apiClient.getPlugins()
    // Merge with marketplace data
    plugins.value = marketplacePlugins.map(mp => ({
      ...mp,
      ...data.find((d: Plugin) => d.id === mp.id)
    }))
  } catch (e) {
    console.error('Failed to load plugins', e)
    plugins.value = marketplacePlugins
  }
}

const filteredPlugins = computed(() => {
  return plugins.value.filter(plugin => {
    const matchesSearch = plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                         plugin.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                         plugin.tags.some(tag => tag.toLowerCase().includes(searchQuery.value.toLowerCase()))
    const matchesCategory = selectedCategory.value === 'all' || plugin.category === selectedCategory.value
    const matchesInstalled = !showInstalledOnly.value || plugin.installed
    return matchesSearch && matchesCategory && matchesInstalled
  })
})

const installedPlugins = computed(() => {
  return plugins.value.filter(p => p.installed)
})

const availablePlugins = computed(() => {
  return plugins.value.filter(p => !p.installed)
})

async function installPlugin(plugin: Plugin) {
  try {
    await apiClient.installPlugin(plugin.id)
    plugin.installed = true
    plugin.enabled = true
    plugin.downloads++
  } catch (e) {
    console.error('Failed to install plugin', e)
    alert('Failed to install plugin: ' + e)
  }
}

async function uninstallPlugin(plugin: Plugin) {
  if (confirm(`Are you sure you want to uninstall ${plugin.name}?`)) {
    try {
      await apiClient.uninstallPlugin(plugin.id)
      plugin.installed = false
      plugin.enabled = false
    } catch (e) {
      console.error('Failed to uninstall plugin', e)
      alert('Failed to uninstall plugin: ' + e)
    }
  }
}

async function togglePlugin(plugin: Plugin) {
  try {
    await apiClient.togglePlugin(plugin.name, !plugin.enabled)
    plugin.enabled = !plugin.enabled
  } catch (e) {
    console.error('Failed to toggle plugin', e)
  }
}

async function reloadPlugin(plugin: Plugin) {
  try {
    await apiClient.reloadPlugin(plugin.name)
  } catch (e) {
    console.error('Failed to reload plugin', e)
  }
}

function showPluginDetails(plugin: Plugin) {
  selectedPlugin.value = plugin
  showDetailsModal.value = true
}

function getCategoryColor(category: string): string {
  const colors: Record<string, string> = {
    security: 'magenta',
    proxy: 'cyan',
    extender: 'purple',
    intruder: 'orange',
    scanner: 'red',
    decoder: 'green',
    utility: 'gray'
  }
  return colors[category] || 'gray'
}

function formatNumber(num: number): string {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k'
  }
  return num.toString()
}

const fileInput = ref<HTMLInputElement | null>(null)

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  try {
    await apiClient.uploadPlugin(file)
    await loadPlugins()
    target.value = ''
  } catch (e) {
    console.error('Failed to upload plugin', e)
    alert('Failed to upload plugin: ' + e)
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="puzzle" size="lg" color="cyan" />
        <h2 class="text-lg font-bold text-i3-text">Plugins</h2>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="cyan">{{ installedPlugins.length }} Installed</Badge>
        <Badge variant="orange">{{ availablePlugins.length }} Available</Badge>
      </div>
    </div>

    <!-- Search and Filter Bar -->
    <div class="flex items-center gap-4 px-4 py-3 bg-i3-bg-alt border border-i3-border rounded-lg">
      <!-- Search -->
      <div class="flex-1 relative">
        <Icon name="search" size="sm" class="absolute left-3 top-1/2 -translate-y-1/2 text-i3-text-muted" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search plugins by name, description, or tags..."
          class="w-full bg-i3-bg border border-i3-border rounded-lg pl-10 pr-4 py-2 text-i3-text placeholder-i3-text-muted focus:border-i3-cyan focus:outline-none focus:ring-2 focus:ring-i3-cyan/20 transition-all duration-200"
        />
      </div>

      <!-- Category Filter -->
      <div class="flex items-center gap-2">
        <Icon name="filter" size="sm" class="text-i3-text-muted" />
        <div class="flex gap-1">
          <button
            v-for="cat in categories"
            :key="cat.id"
            @click="selectedCategory = cat.id"
            :class="[
              'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200',
              selectedCategory === cat.id
                ? 'bg-i3-cyan/20 border border-i3-cyan text-i3-cyan shadow-glow-cyan'
                : 'bg-i3-bg border border-i3-border text-i3-text-muted hover:border-i3-cyan/50 hover:text-i3-text'
            ]"
          >
            <Icon :name="cat.icon" size="xs" />
            <span>{{ cat.name }}</span>
          </button>
        </div>
      </div>

      <!-- Toggle Installed Only -->
      <label class="flex items-center gap-2 cursor-pointer group">
        <input
          v-model="showInstalledOnly"
          type="checkbox"
          class="accent-i3-cyan peer"
        />
        <div class="w-5 h-5 bg-i3-bg border border-i3-border rounded peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
        <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Installed Only</span>
      </label>

      <!-- Load Plugin Button -->
      <input
        type="file"
        ref="fileInput"
        accept=".wasm"
        class="hidden"
        @change="handleFileUpload"
      />
      <Button
        @click="fileInput?.click()"
        variant="primary"
        size="sm"
        class="shadow-neon-cyan"
      >
        <Icon name="plus" size="xs" />
        <span>Load Plugin</span>
      </Button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto">
      <!-- Available Plugins Section -->
      <div v-if="!showInstalledOnly && availablePlugins.length > 0" class="mb-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-bold text-i3-text flex items-center gap-2">
            <Icon name="download-cloud" size="md" color="orange" />
            Available Plugins
          </h3>
          <span class="text-sm text-i3-text-muted">{{ availablePlugins.length }} plugins</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <div
            v-for="plugin in availablePlugins"
            :key="plugin.id"
            class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 hover:border-i3-cyan/50 transition-all duration-200 group hover:shadow-glow-cyan/20"
          >
            <!-- Plugin Header -->
            <div class="flex items-start gap-3 mb-3">
              <div class="w-12 h-12 rounded-lg bg-i3-bg border border-i3-border flex items-center justify-center group-hover:border-i3-cyan transition-colors">
                <Icon :name="plugin.icon" size="lg" :color="getCategoryColor(plugin.category)" />
              </div>
              <div class="flex-1 min-w-0">
                <h4 class="text-base font-bold text-i3-text truncate">{{ plugin.name }}</h4>
                <div class="flex items-center gap-2 mt-1">
                  <Badge :variant="getCategoryColor(plugin.category)" outline size="sm">
                    {{ plugin.category }}
                  </Badge>
                  <span class="text-xs text-i3-text-muted">v{{ plugin.version }}</span>
                </div>
              </div>
            </div>

            <!-- Description -->
            <p class="text-sm text-i3-text-secondary mb-3 line-clamp-2">{{ plugin.description }}</p>

            <!-- Tags -->
            <div class="flex flex-wrap gap-1 mb-3">
              <span
                v-for="tag in plugin.tags.slice(0, 3)"
                :key="tag"
                class="text-[10px] uppercase font-bold text-i3-text-muted bg-i3-bg border border-i3-border px-1.5 py-0.5 rounded"
              >
                {{ tag }}
              </span>
            </div>

            <!-- Stats -->
            <div class="flex items-center gap-4 mb-3 text-xs text-i3-text-muted">
              <div class="flex items-center gap-1">
                <Icon name="star" size="xs" color="orange" />
                <span>{{ plugin.rating.toFixed(1) }}</span>
              </div>
              <div class="flex items-center gap-1">
                <Icon name="download" size="xs" />
                <span>{{ formatNumber(plugin.downloads) }}</span>
              </div>
              <div class="flex items-center gap-1">
                <Icon name="user" size="xs" />
                <span>{{ plugin.author }}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex gap-2">
              <Button
                @click="installPlugin(plugin)"
                variant="primary"
                size="sm"
                class="flex-1 shadow-neon-cyan"
              >
                <Icon name="download" size="xs" />
                <span>Install</span>
              </Button>
              <Button
                @click="showPluginDetails(plugin)"
                variant="secondary"
                size="sm"
              >
                <Icon name="info" size="xs" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- Installed Plugins Section -->
      <div v-if="installedPlugins.length > 0">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-bold text-i3-text flex items-center gap-2">
            <Icon name="check-circle" size="md" color="cyan" />
            Installed Plugins
          </h3>
          <span class="text-sm text-i3-text-muted">{{ installedPlugins.length }} plugins</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <div
            v-for="plugin in installedPlugins"
            :key="plugin.id"
            class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 hover:border-i3-cyan/50 transition-all duration-200 group hover:shadow-glow-cyan/20"
          >
            <!-- Plugin Header -->
            <div class="flex items-start gap-3 mb-3">
              <div class="w-12 h-12 rounded-lg bg-i3-bg border border-i3-border flex items-center justify-center group-hover:border-i3-cyan transition-colors">
                <Icon :name="plugin.icon" size="lg" :color="getCategoryColor(plugin.category)" />
              </div>
              <div class="flex-1 min-w-0">
                <h4 class="text-base font-bold text-i3-text truncate">{{ plugin.name }}</h4>
                <div class="flex items-center gap-2 mt-1">
                  <Badge :variant="plugin.enabled ? 'cyan' : 'gray'}" outline size="sm">
                    {{ plugin.enabled ? 'Active' : 'Disabled' }}
                  </Badge>
                  <Badge :variant="getCategoryColor(plugin.category)" outline size="sm">
                    {{ plugin.category }}
                  </Badge>
                </div>
              </div>
            </div>

            <!-- Description -->
            <p class="text-sm text-i3-text-secondary mb-3 line-clamp-2">{{ plugin.description }}</p>

            <!-- Hooks -->
            <div class="flex flex-wrap gap-1 mb-3">
              <span
                v-for="hook in plugin.hooks.slice(0, 3)"
                :key="hook"
                class="text-[10px] uppercase font-bold text-i3-cyan/70 bg-i3-cyan/10 px-1.5 py-0.5 rounded border border-i3-cyan/20"
              >
                {{ hook }}
              </span>
            </div>

            <!-- Stats -->
            <div class="flex items-center gap-4 mb-3 text-xs text-i3-text-muted">
              <div class="flex items-center gap-1">
                <Icon name="star" size="xs" color="orange" />
                <span>{{ plugin.rating.toFixed(1) }}</span>
              </div>
              <div class="flex items-center gap-1">
                <Icon name="download" size="xs" />
                <span>{{ formatNumber(plugin.downloads) }}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex gap-2">
              <Button
                @click="togglePlugin(plugin)"
                :variant="plugin.enabled ? 'secondary' : 'primary'"
                size="sm"
                class="flex-1"
              >
                <Icon :name="plugin.enabled ? 'pause' : 'play'" size="xs" />
                <span>{{ plugin.enabled ? 'Disable' : 'Enable' }}</span>
              </Button>
              <Button
                @click="reloadPlugin(plugin)"
                variant="secondary"
                size="sm"
              >
                <Icon name="refresh" size="xs" />
              </Button>
              <Button
                @click="uninstallPlugin(plugin)"
                variant="secondary"
                size="sm"
              >
                <Icon name="delete" size="xs" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredPlugins.length === 0" class="text-center py-12 border-2 border-dashed border-i3-border/30 rounded-lg">
        <Icon name="puzzle" size="xl" color="muted" class="mb-4" />
        <h3 class="text-xl font-bold text-i3-text-secondary mb-2">No Plugins Found</h3>
        <p class="text-i3-text-muted max-w-md mx-auto mb-4">
          {{ searchQuery ? 'Try adjusting your search or filter criteria.' : 'Browse the marketplace to discover and install plugins.' }}
        </p>
        <Button
          v-if="searchQuery || selectedCategory !== 'all' || showInstalledOnly"
          @click="searchQuery = ''; selectedCategory = 'all'; showInstalledOnly = false"
          variant="secondary"
          size="sm"
        >
          <Icon name="x" size="xs" />
          <span>Clear Filters</span>
        </Button>
      </div>
    </div>

    <!-- Plugin Details Modal -->
    <div
      v-if="showDetailsModal && selectedPlugin"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4"
      @click.self="showDetailsModal = false"
    >
      <div class="bg-i3-bg border border-i3-border rounded-xl max-w-2xl w-full max-h-[90vh] overflow-auto shadow-glow-cyan">
        <!-- Modal Header -->
        <div class="flex items-center justify-between p-6 border-b border-i3-border">
          <div class="flex items-center gap-4">
            <div class="w-16 h-16 rounded-lg bg-i3-bg-alt border border-i3-border flex items-center justify-center">
              <Icon :name="selectedPlugin.icon" size="xl" :color="getCategoryColor(selectedPlugin.category)" />
            </div>
            <div>
              <h3 class="text-xl font-bold text-i3-text">{{ selectedPlugin.name }}</h3>
              <div class="flex items-center gap-2 mt-1">
                <Badge :variant="getCategoryColor(selectedPlugin.category)" outline>
                  {{ selectedPlugin.category }}
                </Badge>
                <span class="text-sm text-i3-text-muted">v{{ selectedPlugin.version }}</span>
              </div>
            </div>
          </div>
          <button
            @click="showDetailsModal = false"
            class="p-2 rounded hover:bg-i3-bg-alt transition-colors"
          >
            <Icon name="x" size="md" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-6">
          <!-- Description -->
          <div>
            <h4 class="text-sm font-bold text-i3-text mb-2">Description</h4>
            <p class="text-sm text-i3-text-secondary">{{ selectedPlugin.description }}</p>
          </div>

          <!-- Author -->
          <div>
            <h4 class="text-sm font-bold text-i3-text mb-2">Author</h4>
            <div class="flex items-center gap-2">
              <Icon name="user" size="sm" />
              <span class="text-sm text-i3-text-secondary">{{ selectedPlugin.author }}</span>
            </div>
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-3 gap-4">
            <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-3 text-center">
              <div class="flex items-center justify-center gap-1 mb-1">
                <Icon name="star" size="sm" color="orange" />
                <span class="text-lg font-bold text-i3-text">{{ selectedPlugin.rating.toFixed(1) }}</span>
              </div>
              <span class="text-xs text-i3-text-muted">Rating</span>
            </div>
            <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-3 text-center">
              <div class="flex items-center justify-center gap-1 mb-1">
                <Icon name="download" size="sm" />
                <span class="text-lg font-bold text-i3-text">{{ formatNumber(selectedPlugin.downloads) }}</span>
              </div>
              <span class="text-xs text-i3-text-muted">Downloads</span>
            </div>
            <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-3 text-center">
              <div class="flex items-center justify-center gap-1 mb-1">
                <Icon name="code" size="sm" />
                <span class="text-lg font-bold text-i3-text">{{ selectedPlugin.hooks.length }}</span>
              </div>
              <span class="text-xs text-i3-text-muted">Hooks</span>
            </div>
          </div>

          <!-- Tags -->
          <div>
            <h4 class="text-sm font-bold text-i3-text mb-2">Tags</h4>
            <div class="flex flex-wrap gap-2">
              <Badge
                v-for="tag in selectedPlugin.tags"
                :key="tag"
                variant="gray"
                outline
                size="sm"
              >
                {{ tag }}
              </Badge>
            </div>
          </div>

          <!-- Hooks -->
          <div>
            <h4 class="text-sm font-bold text-i3-text mb-2">Event Hooks</h4>
            <div class="flex flex-wrap gap-2">
              <Badge
                v-for="hook in selectedPlugin.hooks"
                :key="hook"
                variant="cyan"
                outline
                size="sm"
              >
                {{ hook }}
              </Badge>
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="flex items-center justify-end gap-3 p-6 border-t border-i3-border">
          <Button
            @click="showDetailsModal = false"
            variant="secondary"
            size="sm"
          >
            <span>Close</span>
          </Button>
          <Button
            v-if="!selectedPlugin.installed"
            @click="installPlugin(selectedPlugin); showDetailsModal = false"
            variant="primary"
            size="sm"
            class="shadow-neon-cyan"
          >
            <Icon name="download" size="xs" />
            <span>Install Plugin</span>
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
