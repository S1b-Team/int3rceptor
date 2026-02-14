<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type Finding, type ScanConfig, type ScanStats, type Severity, type VulnerabilityCategory } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import ProgressRing from '../shared/ProgressRing.vue'

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
const scanProgress = ref(0)

const config = ref<ScanConfig>({
  passive: true,
  active: false,
  categories: ['injection', 'xss', 'path_traversal', 'information_disclosure', 'security_misconfiguration'],
  concurrency: 5,
  delay_ms: 100,
  follow_redirects: true,
  max_depth: 3
})

const severityData = computed(() => {
  return [
    { label: 'Critical', value: stats.value.critical_count, color: 'magenta' as const },
    { label: 'High', value: stats.value.high_count, color: 'orange' as const },
    { label: 'Medium', value: stats.value.medium_count, color: 'yellow' as const },
    { label: 'Low', value: stats.value.low_count, color: 'cyan' as const },
    { label: 'Info', value: stats.value.info_count, color: 'gray' as const },
  ].filter(item => item.value > 0)
})

const totalFindings = computed(() => severityData.value.reduce((sum, item) => sum + item.value, 0))

const allCategories: VulnerabilityCategory[] = [
  'injection', 'xss', 'broken_auth', 'sensitive_data_exposure', 'xxe',
  'broken_access_control', 'security_misconfiguration', 'csrf',
  'vulnerable_components', 'insufficient_logging', 'ssrf',
  'path_traversal', 'information_disclosure', 'open_redirect', 'other'
]

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

const getSeverityColor = (severity: Severity): string => {
  const colors: Record<Severity, string> = {
    critical: 'text-i3-magenta',
    high: 'text-i3-orange',
    medium: 'text-i3-yellow',
    low: 'text-i3-cyan',
    info: 'text-i3-text-muted'
  }
  return colors[severity] || 'text-i3-text-muted'
}

const getSeverityBadgeVariant = (severity: Severity): 'magenta' | 'orange' | 'yellow' | 'cyan' | 'gray' => {
  const variants: Record<Severity, 'magenta' | 'orange' | 'yellow' | 'cyan' | 'gray'> = {
    critical: 'magenta',
    high: 'orange',
    medium: 'yellow',
    low: 'cyan',
    info: 'gray'
  }
  return variants[severity] || 'gray'
}

const getSeverityIcon = (severity: Severity): string => {
  const icons: Record<Severity, string> = {
    critical: 'critical',
    high: 'high',
    medium: 'medium',
    low: 'low',
    info: 'info'
  }
  return icons[severity] || 'info'
}

const getSeverityLevel = (severity: Severity): number => {
  const levels: Record<Severity, number> = {
    critical: 4,
    high: 3,
    medium: 2,
    low: 1,
    info: 0
  }
  return levels[severity] || 0
}

const canStart = computed(() => {
  return targetUrls.value.trim().length > 0
})

let pollInterval: number | null = null

const startPolling = () => {
  if (pollInterval) clearInterval(pollInterval)
  pollInterval = setInterval(async () => {
    await fetchStats()
    if (stats.value.is_running && totalFindings.value > 0) {
      scanProgress.value = Math.min(scanProgress.value + Math.ceil(totalFindings.value / 20), totalFindings.value)
    }
  }, 500)
}

const stopPolling = () => {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
}

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

const startScan = async () => {
  const targets = targetUrls.value.split('\n').filter(t => t.trim())
  if (targets.length === 0) {
    alert('Please enter at least one target URL')
    return
  }

  isLoading.value = true
  scanProgress.value = 0
  findings.value = []

  try {
    await apiClient.scannerStart(targets)
    await fetchStats()
    startPolling()
  } catch (e) {
    console.error('Failed to start scan:', e)
    alert('Failed to start scan. Check console for details.')
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
  } finally {
    stopPolling()
  }
}

const clearFindings = async () => {
  try {
    await apiClient.scannerClearFindings()
    findings.value = []
    selectedFinding.value = null
    scanProgress.value = 0
    await fetchStats()
  } catch (e) {
    console.error('Failed to clear findings:', e)
  }
}

const saveConfig = async () => {
  try {
    await apiClient.scannerSetConfig(config.value)
    alert('Configuration saved successfully!')
  } catch (e) {
    console.error('Failed to save config:', e)
    alert('Failed to save configuration. Check console for details.')
  }
}

const toggleCategory = (cat: VulnerabilityCategory) => {
  const idx = config.value.categories.indexOf(cat)
  if (idx >= 0) {
    config.value.categories.splice(idx, 1)
  } else {
    config.value.categories.push(cat)
  }
}

onMounted(async () => {
  await fetchStats()
  await fetchFindings()
  await fetchConfig()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header with Stats -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="scanner" size="lg" color="cyan" />
        <div>
          <h2 class="text-lg font-bold text-i3-text">Scanner</h2>
          <p class="text-xs text-i3-text-muted">Security vulnerability detection</p>
        </div>
      </div>

      <!-- Stats Pills -->
      <div class="flex gap-3">
        <div class="flex items-center gap-2 px-3 py-1.5 bg-i3-bg border border-i3-border rounded">
          <span class="text-i3-text-muted text-xs">Scanned:</span>
          <span class="font-mono text-i3-cyan text-sm">{{ stats.requests_scanned }}</span>
        </div>
        <div class="flex items-center gap-2 px-3 py-1.5 bg-i3-bg border border-i3-border rounded">
          <span class="text-i3-text-muted text-xs">Findings:</span>
          <span class="font-mono text-i3-orange text-sm">{{ stats.vulnerabilities_found }}</span>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex gap-4 overflow-hidden">
      <!-- Left Panel: Findings & Config -->
      <div class="flex-1 flex flex-col gap-4 overflow-hidden">
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
            <Icon name="list" size="xs" />
            <span>Findings ({{ findings.length }})</span>
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
            <Icon name="settings" size="xs" />
            <span>Configuration</span>
          </button>
        </div>

        <!-- Findings Tab Content -->
        <div v-if="activeTab === 'findings'" class="flex-1 flex flex-col overflow-hidden">
          <!-- Findings List -->
          <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Empty State -->
            <div v-if="findings.length === 0 && !stats.is_running" class="h-full flex flex-col items-center justify-center py-12">
              <Icon name="shield" size="xl" color="muted" class="mb-3" />
              <div class="text-center">
                <div class="text-lg font-semibold text-i3-text">No vulnerabilities found</div>
                <div class="text-sm text-i3-text-muted mt-1">Start a scan to detect security issues</div>
              </div>
            </div>

            <!-- Findings Cards -->
            <div v-else class="flex-1 overflow-auto">
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <div
                  v-for="finding in findings"
                  :key="finding.id"
                  @click="selectedFinding = finding"
                  :class="[
                    'panel p-4 cursor-pointer transition-all duration-200',
                    selectedFinding?.id === finding.id
                      ? 'border-i3-cyan shadow-glow-cyan'
                      : 'border-i3-border hover:border-i3-border-active hover:shadow-elevation-sm'
                  ]"
                >
                  <div class="flex items-start gap-3">
                    <div class="flex-shrink-0">
                      <Icon :name="getSeverityIcon(finding.severity)" size="lg" :color="getSeverityColor(finding.severity)" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between mb-2">
                        <h4 class="text-base font-bold text-i3-text truncate">{{ finding.title }}</h4>
                        <Badge :variant="getSeverityBadgeVariant(finding.severity)">
                          {{ finding.severity.toUpperCase() }}
                        </Badge>
                      </div>
                      <div class="text-xs text-i3-text-muted mb-1 truncate">{{ finding.url }}</div>
                      <div class="flex items-center gap-2 mb-2">
                        <span class="text-xs px-1.5 py-0.5 bg-i3-bg rounded text-i3-text-secondary">
                          {{ getCategoryLabel(finding.category) }}
                        </span>
                        <span v-if="finding.confirmed" class="text-xs px-1.5 py-0.5 bg-i3-cyan/20 text-i3-cyan rounded">
                          Confirmed
                        </span>
                      </div>
                      <p class="text-sm text-i3-text-secondary line-clamp-2">{{ finding.description }}</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Scan Progress Overlay -->
            <div v-if="stats.is_running" class="absolute bottom-4 right-4 bg-i3-bg-alt border border-i3-border rounded-lg p-4 shadow-elevation-lg">
              <div class="flex items-center gap-3">
                <Icon v-if="isLoading" name="spinner" size="sm" spin color="cyan" />
                <div class="text-sm text-i3-text">
                  <span v-if="isLoading">Initializing scan...</span>
                  <span v-else>Scanning in progress...</span>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <ProgressRing :progress="scanProgress" :size="24" :stroke-width="2" variant="cyan" />
                <span class="text-xs text-i3-text-muted">{{ scanProgress }}%</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Config Tab Content -->
        <div v-if="activeTab === 'config'" class="flex-1 overflow-auto">
          <div class="space-y-6">
            <!-- Target URLs -->
            <div>
              <label class="block text-sm font-medium text-i3-text mb-2">Target URLs (one per line)</label>
              <textarea
                v-model="targetUrls"
                rows="4"
                placeholder="https://example.com&#10;https://api.example.com/v1&#10;https://example.com/admin"
                class="w-full bg-i3-bg-alt border border-i3-border rounded-lg p-3 text-sm font-mono text-i3-text placeholder:text-i3-text-muted focus:outline-none focus:border-i3-cyan resize-none"
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
                  class="flex items-center gap-2 p-2 bg-i3-bg rounded border border-i3-border cursor-pointer hover:border-i3-cyan/50 transition-all duration-200"
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
                  class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-sm text-i3-text focus:outline-none focus:border-i3-cyan"
                >
              </div>
              <div>
                <label class="block text-sm font-medium text-i3-text mb-2">Delay (ms)</label>
                <input
                  type="number"
                  v-model.number="config.delay_ms"
                  min="0"
                  class="w-full bg-i3-bg-alt border border-i3-border rounded p-2 text-sm text-i3-text focus:outline-none focus:border-i3-cyan"
                >
              </div>
            </div>

            <!-- Additional Options -->
            <div class="grid grid-cols-2 gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="config.follow_redirects" class="accent-i3-cyan" />
                <span class="text-sm text-i3-text-secondary">Follow Redirects</span>
              </label>
              <div>
                <label class="block text-sm font-medium text-i3-text mb-2">Max Depth</label>
                <input
                  type="number"
                  v-model.number="config.max_depth"
                  min="1"
                  max="10"
                  class="w-16 bg-i3-bg-alt border border-i3-border rounded p-2 text-sm text-i3-text text-center focus:outline-none focus:border-i3-cyan"
                >
              </div>
            </div>

            <!-- Save Button -->
            <button
              @click="saveConfig"
              class="w-full px-4 py-2.5 bg-i3-cyan text-i3-bg font-bold rounded-lg hover:bg-i3-cyan/80 hover:shadow-glow-cyan transition-all duration-200"
            >
              <Icon name="save" size="sm" />
              <span>Save Configuration</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Right Panel: Finding Details -->
      <div class="w-1/3 panel flex flex-col overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 bg-i3-bg border-b border-i3-border">
          <div class="flex items-center gap-2">
            <Icon name="details" size="sm" color="cyan" />
            <h3 class="text-sm font-bold text-i3-text">Finding Details</h3>
          </div>
          <button
            @click="selectedFinding = null"
            class="p-1.5 bg-i3-bg-alt border border-i3-border rounded text-i3-text-muted hover:text-i3-text hover:border-i3-cyan transition-all duration-200"
          >
            <Icon name="close" size="xs" />
          </button>
        </div>

        <!-- Content -->
        <div v-if="selectedFinding" class="flex-1 overflow-auto p-4 space-y-4">
          <!-- Severity & Title -->
          <div class="flex items-center gap-3 mb-4">
            <div class="flex items-center gap-3">
              <Icon :name="getSeverityIcon(selectedFinding.severity)" size="xl" :color="getSeverityColor(selectedFinding.severity)" />
            </div>
            <div class="flex-1">
              <h4 class="text-xl font-bold text-i3-text">{{ selectedFinding.title }}</h4>
              <Badge :variant="getSeverityBadgeVariant(selectedFinding.severity)" size="lg">
                {{ selectedFinding.severity.toUpperCase() }}
              </Badge>
            </div>
          </div>

          <!-- URL -->
          <div>
            <label class="block text-xs text-i3-text-muted mb-1">Affected URL</label>
            <div class="bg-i3-bg-alt p-2 rounded border border-i3-border">
              <a :href="selectedFinding.url" target="_blank" class="text-sm font-mono text-i3-cyan break-all">{{ selectedFinding.url }}</a>
            </div>
          </div>

          <!-- Description -->
          <div>
            <label class="block text-xs text-i3-text-muted mb-1">Description</label>
            <p class="text-sm text-i3-text-secondary bg-i3-bg p-3 rounded border border-i3-border">{{ selectedFinding.description }}</p>
          </div>

          <!-- Evidence -->
          <div v-if="selectedFinding.evidence">
            <label class="block text-xs text-i3-text-muted mb-1">Evidence</label>
            <pre class="text-xs font-mono text-i3-text-secondary bg-i3-bg p-3 rounded border border-i3-border overflow-x-auto whitespace-pre-wrap">{{ selectedFinding.evidence }}</pre>
          </div>

          <!-- Remediation -->
          <div v-if="selectedFinding.remediation">
            <label class="block text-xs text-i3-text-muted mb-1">Remediation</label>
            <p class="text-sm text-i3-text-secondary bg-i3-bg p-3 rounded border border-i3-border">{{ selectedFinding.remediation }}</p>
          </div>

          <!-- References -->
          <div v-if="selectedFinding.references && selectedFinding.references.length > 0">
            <label class="block text-xs text-i3-text-muted mb-1">References</label>
            <ul class="text-sm space-y-1">
              <li v-for="ref in selectedFinding.references" :key="ref">
                <a :href="ref" target="_blank" class="text-i3-cyan hover:underline break-all">{{ ref }}</a>
              </li>
            </ul>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="flex-1 flex flex-col items-center justify-center text-i3-text-muted">
          <div class="text-center">
            <Icon name="list" size="xl" color="muted" class="mb-3" />
            <div class="text-lg font-semibold">Select a Finding</div>
            <div class="text-sm mt-1">Click on a vulnerability to view details</div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2">
        <button
          @click="startScan"
          :disabled="!canStart || isLoading || stats.is_running"
          :class="[
            'flex-1 px-4 py-2.5 rounded-lg text-sm font-bold uppercase tracking-wider transition-all duration-200',
            !canStart || isLoading || stats.is_running
              ? 'bg-i3-bg border-i3-border text-i3-text-muted cursor-not-allowed'
              : 'bg-gradient-to-r from-i3-cyan to-i3-cyan/80 text-i3-bg shadow-neon-cyan hover:shadow-glow-cyan-lg hover:-translate-y-0.5'
          ]"
        >
          <Icon v-if="isLoading" name="spinner" size="sm" spin />
          <Icon v-else name="play" size="sm" />
          <span>{{ isLoading ? 'Initializing...' : stats.is_running ? 'Scan Running...' : 'Start Scan' }}</span>
        </button>
        <button
          v-if="stats.is_running"
          @click="stopScan"
          class="px-4 py-2.5 rounded-lg bg-i3-magenta/20 border border-i3-magenta text-i3-magenta text-sm font-bold uppercase tracking-wider hover:bg-i3-magenta/30 hover:shadow-glow-magenta transition-all duration-200"
        >
          <Icon name="stop" size="sm" />
          <span>Stop Scan</span>
        </button>
        <button
          @click="clearFindings"
          :disabled="stats.is_running || findings.length === 0"
          class="p-2.5 rounded-lg bg-i3-bg-alt border border-i3-border text-i3-text-muted hover:border-i3-red hover:text-i3-red transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Icon name="delete" size="sm" />
        </button>
      </div>
    </div>
  </div>
</template>
