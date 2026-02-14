<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { apiClient, type AttackType, type IntruderConfig, type IntruderResult } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import ProgressRing from '../shared/ProgressRing.vue'

const template = ref('')
const payloadsText = ref('')
const attackType = ref<AttackType>('Sniper')
const running = ref(false)
const results = ref<IntruderResult[]>([])
const generatedRequests = ref<string[]>([])
const concurrency = ref(1)
const delay = ref(0)
const pollingInterval = ref<number | null>(null)
const attackProgress = ref(0)
const totalRequests = ref(0)
const completedRequests = ref(0)

const attackTypes = [
  {
    value: 'Sniper' as AttackType,
    label: 'Sniper',
    icon: 'sniper',
    description: 'Uses a single payload set, iterating through each payload position one at a time. Best for targeted testing.',
    variant: 'cyan' as const
  },
  {
    value: 'Battering' as AttackType,
    label: 'Battering Ram',
    icon: 'battering',
    description: 'Uses a single payload set, inserting the same payload into all defined positions simultaneously.',
    variant: 'magenta' as const
  },
  {
    value: 'Pitchfork' as AttackType,
    label: 'Pitchfork',
    icon: 'pitchfork',
    description: 'Uses multiple payload sets, iterating through them simultaneously at each position.',
    variant: 'orange' as const
  },
  {
    value: 'ClusterBomb' as AttackType,
    label: 'Cluster Bomb',
    icon: 'cluster-bomb',
    description: 'Uses multiple payload sets, generating all possible combinations (Cartesian product).',
    variant: 'purple' as const
  },
]

const detectedPositions = computed(() => {
  const regex = /§(\w+)§/g
  const matches = [...template.value.matchAll(regex)]
  const unique = new Set(matches.map(m => m[1]).filter((n): n is string => !!n))
  return Array.from(unique).map((name) => ({
    name,
    start: 0,
    end: 0,
  }))
})

const payloadsList = computed(() => {
  return payloadsText.value
    .split('\n')
    .map(p => p.trim())
    .filter(p => p.length > 0)
})

const estimatedRequests = computed(() => {
  const positions = detectedPositions.value.length
  const payloads = payloadsList.value.length

  if (positions === 0 || payloads === 0) return 0

  switch (attackType.value) {
    case 'Sniper':
      return positions * payloads
    case 'Battering':
      return payloads
    case 'Pitchfork':
      return payloads
    case 'ClusterBomb':
      return Math.pow(payloads, positions)
    default:
      return 0
  }
})

const canGenerate = computed(() => {
  return (
    template.value.trim().length > 0 &&
    detectedPositions.value.length > 0 &&
    payloadsList.value.length > 0
  )
})

const progressPercentage = computed(() => {
  if (totalRequests.value === 0) return 0
  return Math.round((completedRequests.value / totalRequests.value) * 100)
})

const interestingResults = computed(() => {
  return results.value.filter(r => {
    const status = Math.floor(r.status_code / 100)
    return status === 2 && r.response_length > 0
  })
})

const startAttack = async () => {
  if (!canGenerate.value) return

  running.value = true
  attackProgress.value = 0
  totalRequests.value = estimatedRequests.value
  completedRequests.value = 0
  generatedRequests.value = []
  results.value = []

  try {
    const config: IntruderConfig = {
      positions: detectedPositions.value,
      payloads: payloadsList.value,
      attack_type: attackType.value,
      options: {
        concurrency: concurrency.value,
        delay_ms: delay.value,
      },
    }

    // First generate preview (optional, but good for UI)
    try {
      const preview = await apiClient.intruderGenerate({ template: template.value, config })
      generatedRequests.value = preview.requests
    } catch (e) {
      console.warn('Failed to generate preview', e)
    }

    await apiClient.intruderStart({
      template: template.value,
      config,
    })

    // Start polling
    startPolling()
  } catch (error) {
    console.error('Failed to start attack:', error)
    alert('Failed to start attack. Check console for details.')
    running.value = false
  }
}

const stopAttack = async () => {
  try {
    await apiClient.intruderStop()
  } catch (error) {
    console.error('Failed to stop attack:', error)
  } finally {
    stopPolling()
    running.value = false
  }
}

const startPolling = () => {
  if (pollingInterval.value) clearInterval(pollingInterval.value)
  pollingInterval.value = window.setInterval(async () => {
    await refreshResults()
    // Update progress
    if (totalRequests.value > 0 && completedRequests.value < totalRequests.value) {
      completedRequests.value = Math.min(completedRequests.value + Math.ceil(totalRequests.value / 20), totalRequests.value)
    }
  }, 500)
}

const stopPolling = () => {
  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
    pollingInterval.value = null
  }
}

const refreshResults = async () => {
  try {
    const newResults = await apiClient.intruderResults()
    results.value = newResults
    completedRequests.value = newResults.length
  } catch (error) {
    console.error('Failed to fetch results:', error)
  }
}

const clearResults = async () => {
  try {
    await apiClient.intruderClear()
    results.value = []
    generatedRequests.value = []
    completedRequests.value = 0
    totalRequests.value = 0
    attackProgress.value = 0
  } catch (error) {
    console.error('Failed to clear results:', error)
  }
}

const loadFromRepeater = () => {
  // TODO: Integrate with selected request from traffic tab
  template.value = `POST /api/login HTTP/1.1
Host: example.com
Content-Type: application/json

{
  "username": "§user§",
  "password": "§pass§"
}`
}

const loadCommonPayloads = () => {
  payloadsText.value = `admin
root
test
administrator
guest
' OR '1'='1
" OR "1"="1
1' OR '1'='1
admin' --
admin' #
1; DROP TABLE users--
<scr'+'ipt>alert('XSS')</scr'+'ipt>
../../../etc/passwd
..\\..\\..\\..\\windows\\system32\\config\\sam`
}

const clearPayloads = () => {
  payloadsText.value = ''
}

const getResultClass = (result: IntruderResult) => {
  const status = Math.floor(result.status_code / 100)
  return {
    'bg-i3-green/10 border-l-i3-green': status === 2,
    'bg-i3-orange/10 border-l-i3-orange': status === 3,
    'bg-i3-magenta/10 border-l-i3-magenta': status === 4,
    'bg-i3-red/10 border-l-i3-red': status === 5,
  }
}

const getStatusBadgeVariant = (statusCode: number): 'cyan' | 'orange' | 'magenta' | 'red' => {
  const status = Math.floor(statusCode / 100)
  if (status === 2) return 'cyan'
  if (status === 3) return 'orange'
  if (status === 4) return 'magenta'
  return 'red'
}

const getStatusText = (statusCode: number): string => {
  const status = Math.floor(statusCode / 100)
  if (status === 2) return 'Success'
  if (status === 3) return 'Redirect'
  if (status === 4) return 'Client Error'
  if (status === 5) return 'Server Error'
  return 'Unknown'
}

const formatBytes = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const formatTime = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

onMounted(() => {
  refreshResults()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="intruder" size="lg" color="cyan" />
        <div>
          <h2 class="text-lg font-bold text-i3-text">Intruder</h2>
          <p class="text-xs text-i3-text-muted">Advanced Payload Fuzzer</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <div v-if="running" class="flex items-center gap-2 px-3 py-1.5 bg-i3-bg border border-i3-border rounded-lg">
          <ProgressRing :progress="progressPercentage" :size="20" :stroke-width="2" variant="cyan" />
          <span class="text-xs text-i3-text">{{ completedRequests }} / {{ totalRequests }} ({{ progressPercentage }}%)</span>
        </div>
        <button
          @click="loadFromRepeater"
          class="flex items-center gap-1.5 px-3 py-1.5 bg-i3-bg border border-i3-border rounded-lg text-xs text-i3-text-secondary hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
        >
          <Icon name="upload" size="xs" />
          <span>Load from Traffic</span>
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 grid grid-cols-2 gap-4 overflow-hidden">
      <!-- Left Panel: Configuration -->
      <div class="flex flex-col gap-4 overflow-hidden">
        <!-- Section 1: Request Template -->
        <div class="panel p-4 flex flex-col">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <span class="flex items-center justify-center w-6 h-6 rounded-full bg-i3-cyan/20 text-i3-cyan text-xs font-bold">1</span>
              <h3 class="text-sm font-bold text-i3-text">Request Template</h3>
            </div>
            <button
              @click="loadFromRepeater"
              class="text-xs px-2 py-1 bg-i3-bg border border-i3-border rounded hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
            >
              Load from Traffic
            </button>
          </div>
          <textarea
            v-model="template"
            placeholder="Paste your HTTP request here...&#10;&#10;Example:&#10;POST /api/login HTTP/1.1&#10;Host: example.com&#10;Content-Type: application/json&#10;&#10;{&#10;  "username": "§user§",&#10;  "password": "§pass§"&#10;}"
            class="w-full flex-1 bg-i3-bg-alt border border-i3-border rounded-lg p-3 text-xs font-mono text-i3-text placeholder:text-i3-text-muted focus:border-i3-cyan focus:ring-1 focus:ring-i3-cyan/20 focus:outline-none resize-none transition-all duration-200"
            spellcheck="false"
          ></textarea>
          <div class="mt-2 flex items-center gap-2 text-xs text-i3-text-muted">
            <Icon name="help" size="xs" />
            <span>Use <code class="px-1.5 py-0.5 bg-i3-cyan/10 text-i3-cyan rounded">§name§</code> to mark payload positions</span>
          </div>
        </div>

        <!-- Section 2: Detected Positions -->
        <div class="panel p-4">
          <div class="flex items-center gap-2 mb-3">
            <span class="flex items-center justify-center w-6 h-6 rounded-full bg-i3-cyan/20 text-i3-cyan text-xs font-bold">2</span>
            <h3 class="text-sm font-bold text-i3-text">Detected Positions</h3>
          </div>
          <div v-if="detectedPositions.length === 0" class="py-6 text-center text-i3-text-muted text-xs">
            <Icon name="info" size="lg" color="muted" class="mb-2" />
            <p>No positions detected. Add §markers§ to your template.</p>
          </div>
          <div v-else class="flex flex-wrap gap-2">
            <div
              v-for="(pos, idx) in detectedPositions"
              :key="idx"
              class="flex items-center gap-2 px-3 py-1.5 bg-i3-bg-alt border border-i3-border rounded-lg"
            >
              <Badge variant="cyan" outline>{{ idx + 1 }}</Badge>
              <code class="text-xs font-mono text-i3-cyan">§{{ pos.name }}§</code>
            </div>
          </div>
        </div>

        <!-- Section 3: Attack Type -->
        <div class="panel p-4">
          <div class="flex items-center gap-2 mb-3">
            <span class="flex items-center justify-center w-6 h-6 rounded-full bg-i3-cyan/20 text-i3-cyan text-xs font-bold">3</span>
            <h3 class="text-sm font-bold text-i3-text">Attack Type</h3>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <label
              v-for="type in attackTypes"
              :key="type.value"
              @click="attackType = type.value"
              :class="[
                'relative p-4 rounded-lg border-2 cursor-pointer transition-all duration-200',
                attackType === type.value
                  ? `bg-${type.variant}-/10 border-${type.variant} shadow-glow-${type.variant}`
                  : 'bg-i3-bg-alt border-i3-border hover:border-i3-border-active'
              ]"
            >
              <input
                type="radio"
                :value="type.value"
                v-model="attackType"
                class="sr-only"
              />
              <div class="flex flex-col items-center text-center gap-2">
                <Icon :name="type.icon" size="xl" :color="attackType === type.value ? type.variant : 'default'" />
                <div class="font-bold text-sm text-i3-text">{{ type.label }}</div>
                <div class="text-xs text-i3-text-muted leading-tight">{{ type.description }}</div>
              </div>
            </label>
          </div>
        </div>

        <!-- Section 4: Payloads -->
        <div class="panel p-4 flex flex-col flex-1">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <span class="flex items-center justify-center w-6 h-6 rounded-full bg-i3-cyan/20 text-i3-cyan text-xs font-bold">4</span>
              <h3 class="text-sm font-bold text-i3-text">Payloads</h3>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="loadCommonPayloads"
                class="flex items-center gap-1.5 px-2 py-1 bg-i3-bg border border-i3-border rounded text-xs text-i3-text-secondary hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
              >
                <Icon name="download" size="xs" />
                <span>Load Common</span>
              </button>
              <button
                @click="clearPayloads"
                class="p-1.5 bg-i3-bg border border-i3-border rounded text-i3-text-muted hover:border-i3-red hover:text-i3-red transition-all duration-200"
              >
                <Icon name="delete" size="xs" />
              </button>
            </div>
          </div>
          <div class="relative flex-1">
            <div class="absolute left-0 top-0 bottom-0 w-8 bg-i3-bg-alt border-r border-i3-border rounded-l-lg flex flex-col items-center gap-1 py-2 text-xs text-i3-text-muted">
              <span v-for="(payload, idx) in payloadsList.slice(0, 15)" :key="idx">{{ idx + 1 }}</span>
              <span v-if="payloadsList.length > 15" class="text-i3-cyan">...</span>
            </div>
            <textarea
              v-model="payloadsText"
              placeholder="Enter payloads (one per line)...&#10;&#10;Example:&#10;admin&#10;root&#10;test&#10;' OR '1'='1&#10;1; DROP TABLE users--"
              class="w-full h-full bg-i3-bg-alt border border-i3-border rounded-lg pl-10 pr-3 py-3 text-xs font-mono text-i3-text placeholder:text-i3-text-muted focus:border-i3-cyan focus:ring-1 focus:ring-i3-cyan/20 focus:outline-none resize-none transition-all duration-200"
              spellcheck="false"
            ></textarea>
          </div>
          <div class="mt-2 text-xs text-i3-text-muted">
            {{ payloadsList.length }} payload{{ payloadsList.length !== 1 ? 's' : '' }} loaded
          </div>
        </div>

        <!-- Section 5: Options -->
        <div class="panel p-4">
          <div class="flex items-center gap-2 mb-3">
            <span class="flex items-center justify-center w-6 h-6 rounded-full bg-i3-cyan/20 text-i3-cyan text-xs font-bold">5</span>
            <h3 class="text-sm font-bold text-i3-text">Options</h3>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-2">
              <label class="text-xs text-i3-text-muted">Concurrency</label>
              <input
                type="number"
                v-model.number="concurrency"
                min="1"
                max="100"
                class="w-full bg-i3-bg-alt border border-i3-border rounded-lg px-3 py-2 text-sm text-i3-text focus:border-i3-cyan focus:outline-none transition-all duration-200"
              >
            </div>
            <div class="flex flex-col gap-2">
              <label class="text-xs text-i3-text-muted">Delay (ms)</label>
              <input
                type="number"
                v-model.number="delay"
                min="0"
                class="w-full bg-i3-bg-alt border border-i3-border rounded-lg px-3 py-2 text-sm text-i3-text focus:border-i3-cyan focus:outline-none transition-all duration-200"
              >
            </div>
          </div>
        </div>

        <!-- Launch Section -->
        <div class="panel p-4">
          <div class="flex gap-3">
            <button
              @click="startAttack"
              :disabled="!canGenerate || running"
              :class="[
                'flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg text-sm font-bold uppercase tracking-wider transition-all duration-200',
                !canGenerate || running
                  ? 'bg-i3-bg border-i3-border text-i3-text-muted cursor-not-allowed'
                  : 'bg-gradient-to-r from-i3-cyan to-i3-cyan/80 text-i3-bg shadow-neon-cyan hover:shadow-glow-cyan-lg hover:-translate-y-0.5'
              ]"
            >
              <Icon v-if="running" name="spinner" size="sm" spin />
              <Icon v-else name="play" size="sm" />
              <span>{{ running ? 'Attack Running...' : `Start Attack (${estimatedRequests} requests)` }}</span>
            </button>
            <button
              v-if="running"
              @click="stopAttack"
              class="flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg bg-i3-red/20 border border-i3-red text-i3-red text-sm font-bold uppercase tracking-wider hover:bg-i3-red/30 hover:shadow-glow-red transition-all duration-200"
            >
              <Icon name="stop" size="sm" />
              <span>Stop Attack</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Right Panel: Results -->
      <div class="panel flex flex-col overflow-hidden">
        <!-- Results Header -->
        <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border">
          <div class="flex items-center gap-2">
            <Icon name="list" size="sm" color="cyan" />
            <h3 class="text-sm font-bold text-i3-text">Attack Results</h3>
          </div>
          <div class="flex items-center gap-2">
            <div class="flex items-center gap-1 px-2 py-1 bg-i3-bg border border-i3-border rounded text-xs text-i3-text-muted">
              <span class="text-i3-green">{{ interestingResults.length }} interesting</span>
              <span class="text-i3-text-muted">/</span>
              <span>{{ results.length }} total</span>
            </div>
            <button
              @click="refreshResults"
              class="p-2 bg-i3-bg border border-i3-border rounded text-i3-text-muted hover:border-i3-cyan hover:text-i3-cyan transition-all duration-200"
              title="Refresh results"
            >
              <Icon name="refresh" size="xs" />
            </button>
            <button
              @click="clearResults"
              class="p-2 bg-i3-bg border border-i3-border rounded text-i3-text-muted hover:border-i3-red hover:text-i3-red transition-all duration-200"
              title="Clear results"
            >
              <Icon name="delete" size="xs" />
            </button>
          </div>
        </div>

        <!-- Results Content -->
        <div class="flex-1 overflow-auto">
          <!-- Empty State -->
          <div v-if="results.length === 0" class="h-full flex flex-col items-center justify-center py-12">
            <Icon name="empty" size="xl" color="muted" class="mb-3" />
            <div class="text-center">
              <div class="text-lg font-semibold text-i3-text">No results yet</div>
              <div class="text-sm text-i3-text-muted mt-1">Generate an attack to see results here</div>
            </div>
          </div>

          <!-- Results Table -->
          <table v-else class="w-full text-sm text-left border-collapse">
            <thead class="sticky top-0 bg-i3-bg-alt z-10 text-xs uppercase text-i3-text-muted font-semibold shadow-elevation-sm">
              <tr>
                <th class="p-3 border-b border-i3-border w-16">#</th>
                <th class="p-3 border-b border-i3-border">Payload</th>
                <th class="p-3 border-b border-i3-border w-20">Status</th>
                <th class="p-3 border-b border-i3-border w-24 text-right">Length</th>
                <th class="p-3 border-b border-i3-border w-24 text-right">Time</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-i3-border">
              <tr
                v-for="(result, idx) in results"
                :key="idx"
                :class="['transition-all duration-200 hover:bg-i3-bg-panel/50', getResultClass(result)]"
              >
                <td class="p-3 text-i3-text-muted font-mono text-xs">{{ idx + 1 }}</td>
                <td class="p-3">
                  <code class="px-2 py-1 bg-i3-bg-alt border border-i3-border rounded text-xs font-mono text-i3-cyan">{{ result.payload }}</code>
                </td>
                <td class="p-3">
                  <Badge :variant="getStatusBadgeVariant(result.status_code)">
                    {{ result.status_code }}
                  </Badge>
                  <span class="ml-2 text-xs text-i3-text-muted">({{ getStatusText(result.status_code) }})</span>
                </td>
                <td class="p-3 text-right font-mono text-xs text-i3-text-secondary">{{ formatBytes(result.response_length) }}</td>
                <td class="p-3 text-right font-mono text-xs text-i3-text-secondary">{{ formatTime(result.duration_ms) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Generated Requests Preview -->
        <div v-if="generatedRequests.length > 0" class="border-t border-i3-border bg-i3-bg-alt p-4">
          <div class="flex items-center justify-between mb-3">
            <h4 class="text-xs font-bold uppercase tracking-wider text-i3-text-muted">Generated Requests Preview</h4>
            <span class="text-xs text-i3-text-muted">{{ generatedRequests.length }} request{{ generatedRequests.length !== 1 ? 's' : '' }} generated</span>
          </div>
          <div class="max-h-40 overflow-auto space-y-2">
            <details
              v-for="(req, idx) in generatedRequests.slice(0, 10)"
              :key="idx"
              class="group"
            >
              <summary class="flex items-center gap-2 cursor-pointer px-3 py-2 bg-i3-bg border border-i3-border rounded hover:border-i3-cyan transition-all duration-200">
                <Icon name="chevron-down" size="xs" color="muted" class="group-hover:text-i3-cyan transition-colors" />
                <span class="text-xs font-mono text-i3-text">Request {{ idx + 1 }}</span>
              </summary>
              <pre class="mt-2 px-3 py-2 bg-i3-bg-alt border border-i3-border rounded text-xs font-mono text-i3-text-secondary whitespace-pre-wrap break-all">{{ req }}</pre>
            </details>
            <div v-if="generatedRequests.length > 10" class="text-center text-xs text-i3-text-muted py-2">
              ... and {{ generatedRequests.length - 10 }} more
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
