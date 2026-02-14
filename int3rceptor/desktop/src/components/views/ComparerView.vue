<script setup lang="ts">
import { ref, computed } from 'vue'
import { apiClient, type CompareMode, type DiffChange } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'

interface ModeOption {
  value: CompareMode
  label: string
  icon: string
  description: string
}

const modeOptions: ModeOption[] = [
  { value: 'lines', label: 'Lines', icon: 'align-left', description: 'Compare line by line' },
  { value: 'words', label: 'Words', icon: 'type', description: 'Compare word by word' },
  { value: 'chars', label: 'Characters', icon: 'hash', description: 'Compare character by character' }
]

const left = ref('')
const right = ref('')
const mode = ref<CompareMode>('lines')
const changes = ref<DiffChange[]>([])
const hasDiff = ref(false)
const viewMode = ref<'side-by-side' | 'unified'>('side-by-side')
const showModeDropdown = ref(false)
const showLineNumbers = ref(true)
const ignoreWhitespace = ref(false)
const ignoreCase = ref(false)

const selectedMode = computed(() => {
  return modeOptions.find(opt => opt.value === mode.value) || modeOptions[0]
})

const diffStats = computed(() => {
  const additions = changes.value.filter(c => c.tag === 'insert').length
  const deletions = changes.value.filter(c => c.tag === 'delete').length
  const modifications = changes.value.filter(c => c.tag === 'equal').length
  return { additions, deletions, modifications, total: changes.value.length }
})

const compare = async () => {
  if (!left.value && !right.value) return

  try {
    const res = await apiClient.comparerDiff(left.value, right.value, mode.value)
    changes.value = res.changes
    hasDiff.value = true
  } catch (e) {
    console.error('Diff failed:', e)
  }
}

const clear = () => {
  left.value = ''
  right.value = ''
  changes.value = []
  hasDiff.value = false
}

const copyToClipboard = async (text: string, label: string) => {
  try {
    await navigator.clipboard.writeText(text)
    console.log(`${label} copied to clipboard`)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}

const exportDiff = () => {
  const diffText = changes.value.map(change => {
    const prefix = change.tag === 'insert' ? '+' : change.tag === 'delete' ? '-' : ' '
    return prefix + change.value
  }).join('\n')

  const blob = new Blob([diffText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `diff-${Date.now()}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

const getLineNumbers = (text: string) => {
  return text.split('\n').map((_, i) => i + 1)
}

const renderSideBySideDiff = () => {
  const leftLines = left.value.split('\n')
  const rightLines = right.value.split('\n')
  const maxLines = Math.max(leftLines.length, rightLines.length)

  return Array.from({ length: maxLines }, (_, i) => {
    const leftLine = leftLines[i] || ''
    const rightLine = rightLines[i] || ''
    const isDifferent = leftLine !== rightLine

    return {
      index: i + 1,
      left: leftLine,
      right: rightLine,
      isDifferent,
      changeType: isDifferent ? (leftLine && !rightLine ? 'delete' : !leftLine && rightLine ? 'insert' : 'modify') : 'equal'
    }
  })
}

const sideBySideDiff = computed(() => {
  return renderSideBySideDiff()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="git-compare" size="lg" color="cyan" />
        <h2 class="text-lg font-bold text-i3-text">Comparer</h2>
      </div>
      <div class="flex items-center gap-2">
        <Badge v-if="hasDiff" variant="green">
          {{ diffStats.additions }} additions
        </Badge>
        <Badge v-if="hasDiff" variant="red">
          {{ diffStats.deletions }} deletions
        </Badge>
      </div>
    </div>

    <!-- Controls Bar -->
    <div class="flex items-center gap-4 px-4 py-3 bg-i3-bg-alt border border-i3-border rounded-lg">
      <!-- Mode Selector -->
      <div class="relative">
        <button
          @click="showModeDropdown = !showModeDropdown"
          class="flex items-center gap-2 px-4 py-2 bg-i3-bg border border-i3-border rounded-lg hover:border-i3-cyan transition-all duration-200"
        >
          <Icon :name="selectedMode.icon" size="sm" />
          <span class="font-medium">{{ selectedMode.label }}</span>
          <Icon name="chevron-down" size="xs" :class="{ 'rotate-180': showModeDropdown }" />
        </button>

        <!-- Dropdown -->
        <div
          v-if="showModeDropdown"
          class="absolute top-full left-0 mt-2 w-64 bg-i3-bg-alt border border-i3-border rounded-lg shadow-xl z-50 overflow-hidden"
        >
          <button
            v-for="opt in modeOptions"
            :key="opt.value"
            @click="mode = opt.value; showModeDropdown = false"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-i3-cyan/10 transition-colors',
              mode === opt.value ? 'bg-i3-cyan/20 border-l-2 border-i3-cyan' : ''
            ]"
          >
            <Icon :name="opt.icon" size="sm" />
            <div class="flex-1">
              <div class="font-medium text-i3-text">{{ opt.label }}</div>
              <div class="text-xs text-i3-text-muted">{{ opt.description }}</div>
            </div>
          </button>
        </div>
      </div>

      <!-- View Mode Toggle -->
      <div class="flex bg-i3-bg rounded-lg border border-i3-border overflow-hidden">
        <button
          @click="viewMode = 'side-by-side'"
          :class="[
            'px-4 py-2 font-medium transition-all duration-200',
            viewMode === 'side-by-side'
              ? 'bg-i3-cyan text-i3-bg shadow-glow-cyan'
              : 'text-i3-text hover:bg-i3-bg-alt'
          ]"
        >
          <Icon name="columns" size="xs" />
          <span>Side-by-Side</span>
        </button>
        <button
          @click="viewMode = 'unified'"
          :class="[
            'px-4 py-2 font-medium transition-all duration-200',
            viewMode === 'unified'
              ? 'bg-i3-magenta text-i3-bg shadow-glow-magenta'
              : 'text-i3-text hover:bg-i3-bg-alt'
          ]"
        >
          <Icon name="file-text" size="xs" />
          <span>Unified</span>
        </button>
      </div>

      <!-- Options -->
      <div class="flex items-center gap-4">
        <label class="flex items-center gap-2 cursor-pointer group">
          <input
            v-model="showLineNumbers"
            type="checkbox"
            class="accent-i3-cyan peer"
          />
          <div class="w-4 h-4 bg-i3-bg border border-i3-border rounded peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
          <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Line Numbers</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer group">
          <input
            v-model="ignoreWhitespace"
            type="checkbox"
            class="accent-i3-cyan peer"
          />
          <div class="w-4 h-4 bg-i3-bg border border-i3-border rounded peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
          <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Ignore Whitespace</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer group">
          <input
            v-model="ignoreCase"
            type="checkbox"
            class="accent-i3-cyan peer"
          />
          <div class="w-4 h-4 bg-i3-bg border border-i3-border rounded peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
          <span class="text-sm text-i3-text-secondary group-hover:text-i3-cyan transition-colors">Ignore Case</span>
        </label>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-2 ml-auto">
        <Button
          @click="clear"
          variant="secondary"
          size="sm"
        >
          <Icon name="trash" size="xs" />
          <span>Clear</span>
        </Button>
        <Button
          v-if="hasDiff"
          @click="exportDiff"
          variant="secondary"
          size="sm"
        >
          <Icon name="download" size="xs" />
          <span>Export</span>
        </Button>
        <Button
          @click="compare"
          variant="primary"
          size="sm"
          class="shadow-neon-cyan"
        >
          <Icon name="git-compare" size="xs" />
          <span>Compare</span>
        </Button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 min-h-0">
      <!-- Input Mode -->
      <div v-if="!hasDiff" class="h-full flex gap-4">
        <!-- Original (Left) -->
        <div class="flex-1 flex flex-col gap-2">
          <div class="flex items-center justify-between px-2">
            <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
              <Icon name="file" size="xs" />
              <span>Original (Left)</span>
            </label>
            <div class="flex items-center gap-2">
              <span class="text-xs text-i3-text-muted">{{ left.length }} chars</span>
              <Button
                v-if="left.value"
                @click="copyToClipboard(left, 'Original')"
                variant="secondary"
                size="xs"
              >
                <Icon name="copy" size="xs" />
                <span>Copy</span>
              </Button>
            </div>
          </div>
          <div class="flex-1 relative">
            <textarea
              v-model="left"
              class="w-full h-full bg-i3-bg border border-i3-border rounded-lg p-4 font-mono text-sm resize-none focus:outline-none focus:border-i3-cyan focus:ring-2 focus:ring-i3-cyan/20 transition-all duration-200"
              placeholder="Paste original text..."
              spellcheck="false"
            ></textarea>
            <div
              v-if="showLineNumbers && left.value"
              class="absolute top-4 left-4 bottom-4 w-8 bg-i3-bg-alt border-r border-i3-border rounded-l-lg overflow-hidden pointer-events-none"
            >
              <div class="text-xs text-i3-text-muted font-mono text-right pr-2 leading-6">
                {{ getLineNumbers(left.value).join('\n') }}
              </div>
            </div>
          </div>
        </div>

        <!-- Modified (Right) -->
        <div class="flex-1 flex flex-col gap-2">
          <div class="flex items-center justify-between px-2">
            <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
              <Icon name="edit" size="xs" />
              <span>Modified (Right)</span>
            </label>
            <div class="flex items-center gap-2">
              <span class="text-xs text-i3-text-muted">{{ right.length }} chars</span>
              <Button
                v-if="right.value"
                @click="copyToClipboard(right, 'Modified')"
                variant="secondary"
                size="xs"
              >
                <Icon name="copy" size="xs" />
                <span>Copy</span>
              </Button>
            </div>
          </div>
          <div class="flex-1 relative">
            <textarea
              v-model="right"
              class="w-full h-full bg-i3-bg border border-i3-border rounded-lg p-4 font-mono text-sm resize-none focus:outline-none focus:border-i3-magenta focus:ring-2 focus:ring-i3-magenta/20 transition-all duration-200"
              placeholder="Paste modified text..."
              spellcheck="false"
            ></textarea>
            <div
              v-if="showLineNumbers && right.value"
              class="absolute top-4 left-4 bottom-4 w-8 bg-i3-bg-alt border-r border-i3-border rounded-l-lg overflow-hidden pointer-events-none"
            >
              <div class="text-xs text-i3-text-muted font-mono text-right pr-2 leading-6">
                {{ getLineNumbers(right.value).join('\n') }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Side-by-Side Diff View -->
      <div v-else-if="hasDiff && viewMode === 'side-by-side'" class="h-full flex gap-4 overflow-auto">
        <!-- Left Panel (Original) -->
        <div class="flex-1 flex flex-col gap-2">
          <div class="flex items-center justify-between px-2">
            <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
              <Icon name="file" size="xs" />
              <span>Original</span>
            </label>
            <Button
              @click="hasDiff = false"
              variant="secondary"
              size="xs"
            >
              <Icon name="edit" size="xs" />
              <span>Edit</span>
            </Button>
          </div>
          <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg overflow-auto">
            <table class="w-full font-mono text-sm">
              <tbody>
                <tr
                  v-for="line in sideBySideDiff"
                  :key="line.index"
                  :class="{
                    'bg-i3-red/10': line.changeType === 'delete',
                    'bg-i3-green/10': line.changeType === 'insert',
                    'bg-i3-orange/10': line.changeType === 'modify'
                  }"
                >
                  <td
                    v-if="showLineNumbers"
                    class="w-12 px-2 py-1 text-right text-i3-text-muted border-r border-i3-border/30"
                  >
                    {{ line.index }}
                  </td>
                  <td
                    class="px-4 py-1 whitespace-pre"
                    :class="{
                      'text-i3-red line-through decoration-i3-red/50': line.changeType === 'delete',
                      'text-i3-text-secondary': line.changeType === 'equal'
                    }"
                  >
                    {{ line.left }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Right Panel (Modified) -->
        <div class="flex-1 flex flex-col gap-2">
          <div class="flex items-center justify-between px-2">
            <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
              <Icon name="edit" size="xs" />
              <span>Modified</span>
            </label>
            <Button
              @click="hasDiff = false"
              variant="secondary"
              size="xs"
            >
              <Icon name="edit" size="xs" />
              <span>Edit</span>
            </Button>
          </div>
          <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg overflow-auto">
            <table class="w-full font-mono text-sm">
              <tbody>
                <tr
                  v-for="line in sideBySideDiff"
                  :key="line.index"
                  :class="{
                    'bg-i3-red/10': line.changeType === 'delete',
                    'bg-i3-green/10': line.changeType === 'insert',
                    'bg-i3-orange/10': line.changeType === 'modify'
                  }"
                >
                  <td
                    v-if="showLineNumbers"
                    class="w-12 px-2 py-1 text-right text-i3-text-muted border-r border-i3-border/30"
                  >
                    {{ line.index }}
                  </td>
                  <td
                    class="px-4 py-1 whitespace-pre"
                    :class="{
                      'text-i3-green': line.changeType === 'insert',
                      'text-i3-text-secondary': line.changeType === 'equal'
                    }"
                  >
                    {{ line.right }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- Unified Diff View -->
      <div v-else-if="hasDiff && viewMode === 'unified'" class="h-full flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
            <Icon name="git-compare" size="xs" />
            <span>Differences</span>
          </label>
          <div class="flex items-center gap-2">
            <Button
              @click="hasDiff = false"
              variant="secondary"
              size="xs"
            >
              <Icon name="edit" size="xs" />
              <span>Edit</span>
            </Button>
          </div>
        </div>
        <div class="flex-1 bg-i3-bg border border-i3-border rounded-lg overflow-auto p-4 font-mono text-sm whitespace-pre-wrap">
          <template v-for="(change, i) in changes" :key="i">
            <span
              v-if="change.tag === 'equal'"
              class="text-i3-text-secondary"
            >{{ change.value }}</span>
            <span
              v-else-if="change.tag === 'delete'"
              class="bg-i3-red/20 text-i3-red line-through decoration-i3-red/50 px-1 rounded"
            >{{ change.value }}</span>
            <span
              v-else-if="change.tag === 'insert'"
              class="bg-i3-green/20 text-i3-green px-1 rounded"
            >{{ change.value }}</span>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
