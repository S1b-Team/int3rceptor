<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type Finding, type ScanConfig, type ScanStats, type Severity, type VulnerabilityCategory } from '../../api/client'
import Badge from '../base/Badge.vue'

// State
const stats = ref<ScanStats>({
  is_running: false,
  requests_scanned: 0,
  vulnerabilities_found: 0,
  critical_count: 0,
  high_count: 0,
  medium_count: 0,
  low_count: 0,
  info_count: 0
})

const findings = ref<Finding[]>([])
const selectedFinding = ref<Finding | null>(null)
const activeTab = ref<'findings' | 'config'>('findings')
const isLoading = ref(false)
const targetUrls = ref('')

const config = ref<ScanConfig>({
  passive: true,
  active: false,
  categories: ['injection', 'xss', 'path_traversal', 'information_disclosure', 'security_misconfiguration'],
  concurrency: 5,
  delay_ms: 100,
  follow_redirects: true,
  max_depth: 3
})

// Polling
let pollInterval: number | null = null

const startPolling = () => {
  pollInterval = setInterval(async () => {
    await fetchStats()
    if (stats.value.is_running) {
      await fetchFindings()
    }
  }, 1000) as unknown as number
}

const stopPolling = () => {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
}

// Fetch data
const fetchStats = async () => {
  try {
    stats.value = await apiClient.scannerGetStats()
  } catch (e) {
    console.error('Failed to fetch scanner stats:', e)
  }
}

const fetchFindings = async () => {
  try {
    findings.value = await apiClient.scannerGetFindings()
  } catch (e) {
    console.error('Failed to fetch findings:', e)
  }
}

const fetchConfig = async () => {
  try {
    config.value = await apiClient.scannerGetConfig()
  } catch (e) {
    console.error('Failed to fetch scanner config:', e)
  }
}

// Actions
const startScan = async () => {
  const targets = targetUrls.value.split('\n').filter(t => t.trim())
  if (targets.length === 0) {
    alert('Please enter at least one target URL')
    return
  }

  isLoading.value = true
  try {
    await apiClient.scannerStart(targets)
    await fetchStats()
  } catch (e) {
    console.error('Failed to start scan:', e)
  } finally {
    isLoading.value = false
  }
}

const stopScan = async () => {
  try {
    await apiClient.scannerStop()
    await fetchStats()
  } catch (e) {
    console.error('Failed to stop scan:', e)
  }
}

const clearFindings = async () => {
  try {
    await apiClient.scannerClearFindings()
    findings.value = []
    selectedFinding.value = null
    await fetchStats()
  } catch (e) {
    console.error('Failed to clear findings:', e)
  }
}

const saveConfig = async () => {
  try {
    await apiClient.scannerSetConfig(config.value)
  } catch (e) {
    console.error('Failed to save config:', e)
  }
}

// Helpers
const getSeverityColor = (severity: Severity): string => {
  const colors: Record<Severity, string> = {
    critical: 'magenta',
    high: 'orange',
    medium: 'yellow',
    low: 'cyan',
    info: 'gray'
  }
  return colors[severity] || 'gray'
}

const getSeverityIcon = (severity: Severity): string => {
  const icons: Record<Severity, string> = {
    critical: '🔴',
    high: '🟠',
    medium: '🟡',
    low: '🔵',
    info: 'ℹ️'
  }
  return icons[severity] || '❓'
}

const getCategoryLabel = (category: VulnerabilityCategory): string => {
  const labels: Record<VulnerabilityCategory, string> = {
    injection: 'Injection',
    xss: 'XSS',
    broken_auth: 'Broken Auth',
    sensitive_data_exposure: 'Data Exposure',
    xxe: 'XXE',
    broken_access_control: 'Access Control',
    security_misconfiguration: 'Misconfiguration',
    csrf: 'CSRF',
    vulnerable_components: 'Vuln Components',
    insufficient_logging: 'Logging',
    ssrf: 'SSRF',
    path_traversal: 'Path Traversal',
    information_disclosure: 'Info Disclosure',
    open_redirect: 'Open Redirect',
    other: 'Other'
  }
  return labels[category] || category
}

const allCategories: VulnerabilityCategory[] = [
  'injection', 'xss', 'broken_auth', 'sensitive_data_exposure', 'xxe',
  'broken_access_control', 'security_misconfiguration', 'csrf',
  'vulnerable_components', 'insufficient_logging', 'ssrf',
  'path_traversal', 'information_disclosure', 'open_redirect', 'other'
]

const toggleCategory = (cat: VulnerabilityCategory) => {
  const idx = config.value.categories.indexOf(cat)
  if (idx >= 0) {
    config.value.categories.splice(idx, 1)
  } else {
    config.value.categories.push(cat)
  }
}

// Lifecycle
onMounted(async () => {
  await fetchStats()
  await fetchFindings()
  await fetchConfig()
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header with Stats -->
    <div class="flex items-center justify-between p-4 bg-i3-bg-alt border border-i3-border rounded-lg">
      <div class="flex items-center gap-6">
        <h2 class="text-xl font-bold text-i3-cyan">🔍 Scanner</h2>

        <!-- Stats Pills -->
        <div class="flex gap-3">
          <div class="flex items-center gap-2 px-3 py-1 bg-i3-bg rounded border border-i3-border">
            <span class="text-i3-text-muted text-sm">Scanned:</span>
            <span class="font-mono text-i3-cyan">{{ stats.requests_scanned }}</span>
          </div>
          <div class="flex items-center gap-2 px-3 py-1 bg-i3-bg rounded border border-i3-border">
            <span class="text-i3-text-muted text-sm">Findings:</span>
            <span class="font-mono text-i3-orange">{{ stats.vulnerabilities_found }}</span>
          </div>
        </div>

        <!-- Severity Breakdown -->
        <div class="flex gap-2 text-xs">
          <span v-if="stats.critical_count" class="px-2 py-0.5 bg-i3-magenta/20 text-i3-magenta rounded">
            🔴 {{ stats.critical_count }}
          </span>
          <span v-if="stats.high_count" class="px-2 py-0.5 bg-i3-orange/20 text-i3-orange rounded">
            🟠 {{ stats.high_count }}
          </span>
          <span v-if="stats.medium_count" class="px-2 py-0.5 bg-yellow-500/20 text-yellow-500 rounded">
            🟡 {{ stats.medium_count }}
          </span>
          <span v-if="stats.low_count" class="px-2 py-0.5 bg-i3-cyan/20 text-i3-cyan rounded">
            🔵 {{ stats.low_count }}
          </span>
          <span v-if="stats.info_count" class="px-2 py-0.5 bg-gray-500/20 text-gray-400 rounded">
            ℹ️ {{ stats.info_count }}
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2">
        <button
          v-if="!stats.is_running"
          @click="startScan"
          :disabled="isLoading"
          class="px-4 py-2 bg-i3-cyan text-i3-bg font-bold rounded hover:bg-i3-cyan/80 transition-colors disabled:opacity-50"
        >
          🚀 Start Scan
        </button>
        <button
          v-else
          @click="stopScan"
          class="px-4 py-2 bg-i3-magenta text-white font-bold rounded hover:bg-i3-magenta/80 transition-colors"
        >
          🛑 Stop
        </button>
        <button
          @click="clearFindings"
          class="px-4 py-2 bg-i3-bg border border-i3-border text-i3-text rounded hover:bg-i3-bg-alt transition-colors"
        >
          🗑️ Clear
        </button>
      </div>
    </div>

    <!-- Tab Navigation -->
    <div class="flex gap-2 border-b border-i3-border">
      <button
        @click="activeTab = 'findings'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'findings'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text'
        ]"
      >
        📋 Findings ({{ findings.length }})
      </button>
      <button
        @click="activeTab = 'config'"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === 'config'
            ? 'text-i3-cyan border-b-2 border-i3-cyan'
            : 'text-i3-text-muted hover:text-i3-text'
        ]"
      >
        ⚙️ Configuration
      </button>
    </div>

    <!-- Content Area -->
    <div class="flex-1 flex gap-4 overflow-hidden">
      <!-- Findings Tab -->
      <template v-if="activeTab === 'findings'">
        <!-- Findings List -->
        <div class="w-1/2 flex flex-col bg-i3-bg-alt border border-i3-border rounded-lg overflow-hidden">
          <div class="p-3 border-b border-i3-border">
            <h3 class="font-bold text-i3-text">Vulnerabilities Found</h3>
          </div>

          <div class="flex-1 overflow-auto">
            <div
              v-for="finding in findings"
              :key="finding.id"
              @click="selectedFinding = finding"
              :class="[
                'p-3 border-b border-i3-border/50 cursor-pointer transition-colors',
                selectedFinding?.id === finding.id ? 'bg-i3-cyan/10' : 'hover:bg-i3-bg'
              ]"
            >
              <div class="flex items-start gap-3">
                <span class="text-lg">{{ getSeverityIcon(finding.severity) }}</span>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-i3-text truncate">{{ finding.title }}</span>
                    <Badge :variant="getSeverityColor(finding.severity) as any">
                      {{ finding.severity.toUpperCase() }}
                    </Badge>
                  </div>
                  <div class="text-xs text-i3-text-muted truncate mt-1">{{ finding.url }}</div>
                  <div class="flex gap-2 mt-1">
                    <span class="text-xs px-1.5 py-0.5 bg-i3-bg rounded text-i3-text-secondary">
                      {{ getCategoryLabel(finding.category) }}
                    </span>
                    <span v-if="finding.confirmed" class="text-xs px-1.5 py-0.5 bg-i3-cyan/20 text-i3-cyan rounded">
                      Confirmed
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="findings.length === 0" class="p-8 text-center text-i3-text-muted">
              <div class="text-4xl mb-2">🛡️</div>
              <p>No vulnerabilities found yet</p>
              <p class="text-sm mt-1">Start a scan to detect security issues</p>
            </div>
          </div>
        </div>

        <!-- Finding Details -->
        <div class="w-1/2 flex flex-col bg-i3-bg-alt border border-i3-border rounded-lg overflow-hidden">
          <div class="p-3 border-b border-i3-border">
            <h3 class="font-bold text-i3-text">Finding Details</h3>
          </div>

          <div v-if="selectedFinding" class="flex-1 overflow-auto p-4 space-y-4">
            <!-- Header -->
            <div class="flex items-center gap-3">
              <span class="text-2xl">{{ getSeverityIcon(selectedFinding.severity) }}</span>
              <div>
                <h4 class="text-lg font-bold text-i3-text">{{ selectedFinding.title }}</h4>
                <Badge :variant="getSeverityColor(selectedFinding.severity) as any">
                  {{ selectedFinding.severity.toUpperCase() }}
                </Badge>
              </div>
            </div>

            <!-- URL -->
            <div>
              <label class="text-xs text-i3-text-muted">Affected URL</label>
              <div class="font-mono text-sm text-i3-cyan bg-i3-bg p-2 rounded break-all">
                {{ selectedFinding.url }}
              </div>
            </div>

            <!-- Description -->
            <div>
              <label class="text-xs text-i3-text-muted">Description</label>
              <p class="text-sm text-i3-text-secondary">{{ selectedFinding.description }}</p>
            </div>

            <!-- Evidence -->
            <div>
              <label class="text-xs text-i3-text-muted">Evidence</label>
              <pre class="text-xs font-mono bg-i3-bg p-3 rounded overflow-x-auto text-i3-text-secondary">{{ selectedFinding.evidence }}</pre>
            </div>

            <!-- Remediation -->
            <div>
              <label class="text-xs text-i3-text-muted">Remediation</label>
              <p class="text-sm text-i3-text-secondary bg-i3-bg p-3 rounded">{{ selectedFinding.remediation }}</p>
            </div>

            <!-- References -->
            <div v-if="selectedFinding.references.length > 0">
              <label class="text-xs text-i3-text-muted">References</label>
              <ul class="text-sm space-y-1">
                <li v-for="ref in selectedFinding.references" :key="ref">
                  <a :href="ref" target="_blank" class="text-i3-cyan hover:underline">{{ ref }}</a>
                </li>
              </ul>
            </div>
          </div>

          <div v-else class="flex-1 flex items-center justify-center text-i3-text-muted">
            <div class="text-center">
              <div class="text-4xl mb-2">👈</div>
              <p>Select a finding to view details</p>
            </div>
          </div>
        </div>
      </template>

      <!-- Config Tab -->
      <template v-if="activeTab === 'config'">
        <div class="flex-1 p-4 bg-i3-bg-alt border border-i3-border rounded-lg overflow-auto space-y-6">
          <!-- Target URLs -->
          <div>
            <label class="block text-sm font-medium text-i3-text mb-2">Target URLs (one per line)</label>
            <textarea
              v-model="targetUrls"
              rows="4"
              placeholder="https://example.com&#10;https://api.example.com/v1&#10;https://example.com/admin"
              class="w-full bg-i3-bg border border-i3-border rounded p-3 text-i3-text font-mono text-sm focus:outline-none focus:border-i3-cyan"
            ></textarea>
          </div>

          <!-- Scan Mode -->
          <div>
            <label class="block text-sm font-medium text-i3-text mb-2">Scan Mode</label>
            <div class="flex gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="config.passive" class="accent-i3-cyan" />
                <span class="text-sm text-i3-text-secondary">Passive (analyze traffic)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="config.active" class="accent-i3-cyan" />
                <span class="text-sm text-i3-text-secondary">Active (send payloads)</span>
              </label>
            </div>
          </div>

          <!-- Categories -->
          <div>
            <label class="block text-sm font-medium text-i3-text mb-2">Vulnerability Categories</label>
            <div class="grid grid-cols-3 gap-2">
              <label
                v-for="cat in allCategories"
                :key="cat"
                class="flex items-center gap-2 p-2 bg-i3-bg rounded border border-i3-border cursor-pointer hover:border-i3-cyan/50"
              >
                <input
                  type="checkbox"
                  :checked="config.categories.includes(cat)"
                  @change="toggleCategory(cat)"
                  class="accent-i3-cyan"
                />
                <span class="text-xs text-i3-text-secondary">{{ getCategoryLabel(cat) }}</span>
              </label>
            </div>
          </div>

          <!-- Options -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-i3-text mb-2">Concurrency</label>
              <input
                type="number"
                v-model.number="config.concurrency"
                min="1"
                max="50"
                class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:outline-none focus:border-i3-cyan"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-i3-text mb-2">Delay (ms)</label>
              <input
                type="number"
                v-model.number="config.delay_ms"
                min="0"
                class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:outline-none focus:border-i3-cyan"
              />
            </div>
          </div>

          <div class="flex items-center gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" v-model="config.follow_redirects" class="accent-i3-cyan" />
              <span class="text-sm text-i3-text-secondary">Follow Redirects</span>
            </label>
            <div class="flex items-center gap-2">
              <span class="text-sm text-i3-text-secondary">Max Depth:</span>
              <input
                type="number"
                v-model.number="config.max_depth"
                min="1"
                max="10"
                class="w-16 bg-i3-bg border border-i3-border rounded p-1 text-i3-text text-center focus:outline-none focus:border-i3-cyan"
              />
            </div>
          </div>

          <!-- Save Button -->
          <button
            @click="saveConfig"
            class="px-4 py-2 bg-i3-cyan text-i3-bg font-bold rounded hover:bg-i3-cyan/80 transition-colors"
          >
            💾 Save Configuration
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
