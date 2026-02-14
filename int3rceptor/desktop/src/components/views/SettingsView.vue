<script setup lang="ts">
import { ref, computed } from 'vue'
import { apiClient } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'
import TwoColumnLayout from '../shared/TwoColumnLayout.vue'

const activeTab = ref<'proxy' | 'security' | 'ui' | 'advanced'>('proxy')
const showCertGuide = ref(false)

// Proxy Settings
const proxyConfig = ref({
  port: 8080,
  host: '127.0.0.1',
  interceptHttps: true,
  http2: true
  enabled: true
})

const proxyList = ref([
  { id: 1, host: '127.0.0.1', port: 8080, enabled: true, name: 'Local Proxy' },
  { id: 2, host: 'proxy.example.com', port: 3128, enabled: false, name: 'External Proxy' },
])

// Security Settings
const ipFilter = ref({
  enabled: false,
  allowList: ['127.0.0.1'],
  blockList: []
})

// UI Settings
const uiConfig = ref({
  theme: 'cyberpunk',
  animations: true,
  notifications: true
})

// Advanced Settings
const advancedConfig = ref({
  maxConnections: 100,
  timeoutMs: 30000,
  bufferSize: 8192
})

const canSave = computed(() => {
  return true
})

const addProxy = () => {
  const name = prompt('Enter proxy name:')
  const host = prompt('Enter proxy host (e.g., 127.0.0.1):')
  const port = prompt('Enter proxy port:')
  if (!name || !host || !port) return
  proxyList.value.push({
    id: Date.now(),
    name,
    host,
    port: parseInt(port),
    enabled: true
  })
}

const removeProxy = (id: number) => {
  const index = proxyList.value.findIndex(p => p.id === id)
  if (index >= 0) {
    proxyList.value.splice(index, 1)
  }
}

const toggleProxy = (id: number) => {
  const proxy = proxyList.value.find(p => p.id === id)
  if (proxy) {
    proxy.enabled = !proxy.enabled
  }
}

const saveSettings = async () => {
  try {
    await apiClient.updateSettings({
      proxy: proxyConfig.value,
      security: ipFilter.value,
      ui: uiConfig.value,
      advanced: advancedConfig.value
    })
    alert('Settings saved successfully!')
  } catch (e) {
    console.error('Failed to save settings:', e)
    alert('Failed to save settings. Check console for details.')
  }
}

const resetDefaults = () => {
  if (confirm('Are you sure you want to reset all settings to defaults?')) {
    proxyConfig.value = {
      port: 8080,
      host: '127.0.0.1',
      interceptHttps: true,
      http2: true,
      enabled: true
    }
    ipFilter.value = {
      enabled: false,
      allowList: ['127.0.0.1'],
      blockList: []
    }
    uiConfig.value = {
      theme: 'cyberpunk',
      animations: true,
      notifications: true
    }
    advancedConfig.value = {
      maxConnections: 100,
      timeoutMs: 30000,
      bufferSize: 8192
    }
  }
}

const getStatusColor = (status: string): string => {
  if (status === 'enabled') return 'text-i3-green'
  if (status === 'disabled') return 'text-i3-red'
  return 'text-i3-text-muted'
}
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="settings" size="lg" color="cyan" />
        <h2 class="text-lg font-bold text-i3-text">Settings</h2>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="cyan">v3.0 Beta</Badge>
      </div>
    </div>

    <!-- Tab Navigation -->
    <div class="flex gap-2 border-b border-i3-border">
      <button
        @click="activeTab = 'proxy'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'proxy'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-alt/50'
        ]"
      >
        <Icon name="shield" size="xs" />
        <span>Proxy</span>
      </button>
      <button
        @click="activeTab = 'security'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'security'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-alt/50'
        ]"
      >
        <Icon name="lock" size="xs" />
        <span>Security</span>
      </button>
      <button
        @click="activeTab = 'ui'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'ui'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-alt/50'
        ]"
      >
        <Icon name="palette" size="xs" />
        <span>UI</span>
      </button>
      <button
        @click="activeTab = 'advanced'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'advanced'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text hover:bg-i3-bg-alt/50'
        ]"
      >
        <Icon name="settings" size="xs" />
        <span>Advanced</span>
      </button>
    </div>

    <!-- Main Content -->
    <TwoColumnLayout
      :left-panel-class="'flex flex-col overflow-hidden'"
      :right-panel-class="'flex flex-col overflow-hidden'"
      :show-right-panel="true"
    >
      <!-- Left Panel: Settings Sections -->
      <template #left>
        <div class="flex-1 flex flex-col gap-4 overflow-auto">
          <!-- Proxy Settings -->
          <div v-if="activeTab === 'proxy'" class="panel p-4">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <Icon name="shield" size="md" color="cyan" />
                <h3 class="text-lg font-bold text-i3-text">Proxy Settings</h3>
              </div>
              <Button
                @click="addProxy"
                variant="secondary"
                size="sm"
                class="shadow-neon-cyan"
              >
                <Icon name="plus" size="xs" />
                <span>Add Proxy</span>
              </Button>
            </div>

            <!-- Active Proxy List -->
            <div class="space-y-3">
              <div
                v-for="proxy in proxyList"
                :key="proxy.id"
                class="flex items-center justify-between p-3 bg-i3-bg-alt border border-i3-border rounded-lg group hover:border-i3-cyan/50 transition-all duration-200"
              >
                <div class="flex items-center gap-3">
                  <Icon name="network" size="sm" :color="proxy.enabled ? 'cyan' : 'muted'" />
                  <div class="flex-1">
                    <div class="font-medium text-i3-text">{{ proxy.name }}</div>
                    <div class="text-xs text-i3-text-muted">{{ proxy.host }}:{{ proxy.port }}</div>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    @click="toggleProxy(proxy.id)"
                    class="p-1.5 rounded bg-i3-bg border border-i3-border hover:bg-i3-cyan/10 transition-all duration-200"
                  >
                    <Icon :name="proxy.enabled ? 'check-circle' : 'x-circle'" size="sm" />
                  </button>
                  <button
                    @click="removeProxy(proxy.id)"
                    class="p-1.5 rounded bg-i3-bg border border-i3-border hover:bg-i3-red/10 transition-all duration-200"
                  >
                    <Icon name="delete" size="sm" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Proxy Configuration -->
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-i3-text mb-2">Proxy Configuration</label>
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-xs text-i3-text-muted">Bind Address</label>
                    <input
                      v-model="proxyConfig.host"
                      type="text"
                      placeholder="127.0.0.1"
                      class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-i3-text focus:border-i3-cyan focus:outline-none transition-all duration-200"
                    >
                  </div>
                  <div>
                    <label class="block text-xs text-i3-text-muted">Port</label>
                    <input
                      v-model.number="proxyConfig.port"
                      type="number"
                      min="1"
                      max="65535"
                      class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-i3-text focus:border-i3-cyan focus:outline-none transition-all duration-200"
                    >
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2 mt-4">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="proxyConfig.interceptHttps"
                    type="checkbox"
                    class="accent-i3-cyan peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Intercept HTTPS</span>
                </label>
              </div>
              <div class="flex items-center gap-2">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="proxyConfig.http2"
                    type="checkbox"
                    class="accent-i3-cyan peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Enable HTTP/2</span>
                </label>
              </div>
              <div class="flex items-center gap-2">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="proxyConfig.enabled"
                    type="checkbox"
                    class="accent-i3-cyan peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Proxy Enabled</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Security Settings -->
          <div v-if="activeTab === 'security'" class="panel p-4">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <Icon name="lock" size="md" color="magenta" />
                <h3 class="text-lg font-bold text-i3-text">Security & Access Control</h3>
              </div>
            </div>

            <!-- IP Filtering -->
            <div class="space-y-4">
              <div class="flex items-center gap-2 mb-4">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="ipFilter.enabled"
                    type="checkbox"
                    class="accent-i3-magenta peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-magenta/20 peer-checked:border-i3-magenta transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-magenta transition-colors">Enable IP Filtering</span>
                </label>
              </div>

              <div v-if="ipFilter.enabled" class="bg-i3-bg-alt border border-i3-border rounded-lg p-4">
                <div>
                  <label class="block text-sm text-i3-text-muted mb-2">Allowed IPs (comma separated)</label>
                  <textarea
                    v-model="ipFilter.allowList"
                    rows="3"
                    placeholder="127.0.0.1, 192.168.1.1"
                    class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:border-i3-magenta focus:outline-none resize-none transition-all duration-200"
                  ></textarea>
                </div>

                <div>
                  <label class="block text-sm text-i3-text-muted mb-2">Blocked IPs (comma separated)</label>
                  <textarea
                    v-model="ipFilter.blockList"
                    rows="3"
                    placeholder="192.168.1.1, 10.0.0.1"
                    class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:border-i3-magenta focus:outline-none resize-none transition-all duration-200"
                  ></textarea>
                </div>
              </div>
            </div>
          </div>

          <!-- UI Settings -->
          <div v-if="activeTab === 'ui'" class="panel p-4">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <Icon name="palette" size="md" color="orange" />
                <h3 class="text-lg font-bold text-i3-text">Interface</h3>
              </div>
            </div>

            <div class="space-y-4">
              <!-- Theme -->
              <div>
                <label class="block text-sm font-medium text-i3-text-muted mb-2">Theme</label>
                <div class="grid grid-cols-3 gap-3">
                  <button
                    v-for="theme in ['cyberpunk', 'minimal', 'matrix']"
                    :key="theme"
                    @click="uiConfig.theme = theme"
                    :class="[
                      'p-3 rounded-lg border border transition-all duration-200',
                      uiConfig.theme === theme
                        ? 'bg-i3-cyan/20 border-i3-cyan shadow-glow-cyan'
                        : 'bg-i3-bg-alt border-i3-border hover:border-i3-cyan/50'
                    ]"
                  >
                    <span class="text-sm font-medium capitalize">{{ theme }}</span>
                  </button>
                </div>
              </div>

              <!-- Animations -->
              <div class="flex items-center gap-2 mt-4">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="uiConfig.animations"
                    type="checkbox"
                    class="accent-i3-orange peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-orange/20 peer-checked:border-i3-orange transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-orange transition-colors">UI Animations</span>
                </label>
              </div>

              <!-- Notifications -->
              <div class="flex items-center gap-2 mt-4">
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    v-model="uiConfig.notifications"
                    type="checkbox"
                    class="accent-i3-orange peer"
                  />
                  <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:bg-i3-orange/20 peer-checked:border-i3-orange transition-all"></div>
                  <span class="text-sm text-i3-text-secondary group-hover:text-i3-orange transition-colors">Notifications</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Advanced Settings -->
          <div v-if="activeTab === 'advanced'" class="panel p-4">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <Icon name="settings" size="md" color="purple" />
                <h3 class="text-lg font-bold text-i3-text">Advanced</h3>
              </div>
            </div>

            <div class="space-y-4">
              <!-- Max Connections -->
              <div>
                <label class="block text-sm font-medium text-i3-text-muted mb-2">Max Connections</label>
                <input
                  v-model.number="advancedConfig.maxConnections"
                  type="number"
                  min="1"
                  max="1000"
                  class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-i3-text focus:border-i3-purple focus:outline-none transition-all duration-200"
                >
              </div>

              <!-- Timeout -->
              <div>
                <label class="block text-sm font-medium text-i3-text-muted mb-2">Request Timeout (ms)</label>
                <input
                  v-model.number="advancedConfig.timeoutMs"
                  type="number"
                  min="1000"
                  max="60000"
                  step="100"
                  class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-i3-text focus:border-i3-purple focus:outline-none transition-all duration-200"
                >
              </div>

              <!-- Buffer Size -->
              <div>
                <label class="block text-sm font-medium text-i3-text-muted mb-2">Buffer Size (KB)</label>
                <input
                  v-model.number="advancedConfig.bufferSize"
                  type="number"
                  min="1024"
                  max="16384"
                  step="1024"
                  class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-i3-text focus:border-i3-purple focus:outline-none transition-all duration-200"
                >
              </div>
            </div>

            <!-- Reset -->
            <div class="mt-6 pt-4 border-t border-i3-border">
              <button
                @click="resetDefaults"
                class="w-full px-4 py-2.5 bg-i3-red/20 border border-i3-red text-i3-bg font-bold rounded-lg hover:bg-i3-red/30 hover:shadow-glow-red transition-all duration-200"
              >
                <Icon name="refresh" size="sm" />
                <span>Reset to Defaults</span>
              </button>
            </div>
          </div>
        </div>
      </template>

      <!-- Right Panel: Details -->
      <template #right>
        <div class="h-full flex flex-col overflow-hidden">
          <!-- Header -->
          <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border">
            <div class="flex items-center gap-2">
              <Icon name="info" size="sm" color="cyan" />
              <h3 class="text-sm font-bold text-i3-text">Settings Details</h3>
            </div>
            <div class="text-xs text-i3-text-muted">
              Configure your INT3RCEPTOR proxy and security settings
            </div>
          </div>

          <!-- Content -->
          <div class="flex-1 overflow-auto p-4 space-y-6">
            <!-- Proxy Settings -->
            <div v-if="activeTab === 'proxy'" class="space-y-4">
              <div class="flex items-center gap-2 mb-2">
                <Icon name="shield" size="sm" color="cyan" />
                <h4 class="text-sm font-bold text-i3-text">Proxy Configuration</h4>
              </div>
              <div class="text-xs text-i3-text-muted">
                Configure the proxy listener that intercepts HTTP/HTTPS traffic
              </div>

              <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 space-y-3">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <span class="text-sm font-medium text-i3-text">Bind Address</span>
                    <span :class="['text-i3-cyan', proxyConfig.host ? '' : 'text-i3-text-muted']">{{ proxyConfig.host || '127.0.0.1' }}</span>
                  </div>
                  <div>
                    <span class="text-sm font-medium text-i3-text">Port</span>
                    <span :class="['text-i3-cyan', proxyConfig.port === 8080 ? '' : 'text-i3-text-muted']">{{ proxyConfig.port }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <Badge :variant="proxyConfig.interceptHttps ? 'cyan' : 'gray'" outline>
                      {{ proxyConfig.interceptHttps ? 'HTTPS' : 'HTTP' }}
                    </Badge>
                  </div>
                  <div>
                    <span class="text-sm font-medium text-i3-text">HTTP/2</span>
                    <Badge :variant="proxyConfig.http2 ? 'cyan' : 'gray'" outline>
                      {{ proxyConfig.http2 ? 'Enabled' : 'Disabled' }}
                    </Badge>
                  </div>
                  <div>
                    <span class="text-sm font-medium text-i3-text">Proxy Status</span>
                    <Badge :variant="proxyConfig.enabled ? 'cyan' : 'red'" outline>
                      {{ proxyConfig.enabled ? 'Active' : 'Inactive' }}
                    </Badge>
                  </div>
                </div>
              </div>

              <!-- Security Settings -->
              <div v-if="activeTab === 'security'" class="space-y-4">
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="lock" size="sm" color="magenta" />
                  <h4 class="text-sm font-bold text-i3-text">IP Filtering</h4>
                </div>
                <div class="text-xs text-i3-text-muted">
                  Control which IP addresses can access the proxy
                </div>

              <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 space-y-3">
                  <div class="flex items-center gap-2 mb-2">
                    <span class="text-sm font-medium text-i3-text">IP Filtering</span>
                    <Badge :variant="ipFilter.enabled ? 'magenta' : 'gray'" outline>
                      {{ ipFilter.enabled ? 'Enabled' : 'Disabled' }}
                    </Badge>
                  </div>
                  <div class="flex-1">
                    <span class="text-sm text-i3-text-muted">Status:</span>
                    <span class="text-sm font-medium" :class="ipFilter.enabled ? 'text-i3-magenta' : 'text-i3-text-muted'">
                      {{ ipFilter.enabled ? 'Active' : 'Inactive' }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- UI Settings -->
              <div v-if="activeTab === 'ui'" class="space-y-4">
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="palette" size="sm" color="orange" />
                  <h4 class="text-sm font-bold text-i3-text">Theme</h4>
                </div>
                <div class="text-xs text-i3-text-muted">
                  Customize the visual appearance of INT3RCEPTOR
                </div>

              <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 space-y-3">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium text-i3-text">Current Theme</span>
                    <span class="text-sm font-bold capitalize text-i3-orange">{{ uiConfig.theme }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-i3-text">UI Animations</span>
                    <Badge :variant="uiConfig.animations ? 'orange' : 'gray'" outline>
                      {{ uiConfig.animations ? 'Enabled' : 'Disabled' }}
                    </Badge>
                  </div>
                  <div class="flex-1">
                    <span class="text-sm font-medium text-i3-text">Notifications</span>
                    <Badge :variant="uiConfig.notifications ? 'orange' : 'gray'" outline>
                      {{ uiConfig.notifications ? 'Enabled' : 'Disabled' }}
                    </Badge>
                  </div>
                </div>
              </div>

              <!-- Advanced Settings -->
              <div v-if="activeTab === 'advanced'" class="space-y-4">
                <div class="flex items-center gap-2 mb-2">
                  <Icon name="settings" size="sm" color="purple" />
                  <h4 class="text-sm font-bold text-i3-text">Performance</h4>
                </div>
                <div class="text-xs text-i3-text-muted">
                  Configure advanced proxy settings for optimal performance
                </div>

                <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-4 space-y-3">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium text-i3-text">Max Connections</span>
                    <span class="text-sm font-bold text-i3-purple">{{ advancedConfig.maxConnections }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-i3-text">Request Timeout</span>
                    <span class="text-sm font-mono text-i3-cyan">{{ advancedConfig.timeoutMs }}ms</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-i3-text">Buffer Size</span>
                    <span class="text-sm font-mono text-i3-cyan">{{ advancedConfig.bufferSize }}KB</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Actions -->
            <div class="mt-6 pt-4 border-t border-i3-border">
              <div class="flex gap-3">
                <Button
                  @click="resetDefaults"
                  variant="secondary"
                  size="sm"
                >
                  <Icon name="refresh" size="xs" />
                  <span>Reset to Defaults</span>
                </Button>
                <Button
                  @click="saveSettings"
                  :disabled="!canSave"
                  variant="primary"
                  size="sm"
                  class="shadow-neon-cyan"
                >
                  <Icon name="save" size="xs" />
                  <span>Save Configuration</span>
                </Button>
              </div>
            </div>
          </div>
        </div>
      </template>
    </TwoColumnLayout>
  </div>
</template>
