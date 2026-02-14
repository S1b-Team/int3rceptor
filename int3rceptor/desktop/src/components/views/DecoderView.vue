<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { apiClient, type EncodingType, type TransformOperation } from '../../api/client'
import Icon from '../shared/Icon.vue'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'

interface EncodingOption {
  value: EncodingType
  label: string
  icon: string
  description: string
}

const encodingOptions: EncodingOption[] = [
  { value: 'base64', label: 'Base64', icon: 'file-code', description: 'Standard Base64 encoding/decoding' },
  { value: 'url', label: 'URL', icon: 'link', description: 'URL encoding/decoding' },
  { value: 'hex', label: 'Hex', icon: 'hash', description: 'Hexadecimal encoding/decoding' },
  { value: 'html', label: 'HTML', icon: 'code', description: 'HTML entity encoding/decoding' },
  { value: 'rot13', label: 'ROT13', icon: 'refresh-cw', description: 'ROT13 cipher' },
  { value: 'binary', label: 'Binary', icon: 'cpu', description: 'Binary encoding/decoding' },
  { value: 'octal', label: 'Octal', icon: 'terminal', description: 'Octal encoding/decoding' },
  { value: 'ascii85', label: 'Ascii85', icon: 'file-text', description: 'Ascii85 (Base85) encoding' },
  { value: 'uuencode', label: 'UUencode', icon: 'archive', description: 'UUencode encoding' },
  { value: 'quoted-printable', label: 'Quoted-Printable', icon: 'mail', description: 'Quoted-Printable encoding' }
]

const input = ref('')
const output = ref('')
const encoding = ref<EncodingType>('base64')
const operation = ref<TransformOperation>('encode')
const error = ref<string | null>(null)
const showEncodingDropdown = ref(false)
const showHistory = ref(false)
const history = ref<Array<{ input: string; output: string; encoding: EncodingType; operation: TransformOperation; timestamp: number }>>([])
const favorites = ref<Array<{ input: string; encoding: EncodingType; operation: TransformOperation; label: string }>>([])
const isSwapping = ref(false)

const selectedEncoding = computed(() => {
  return encodingOptions.find(opt => opt.value === encoding.value) || encodingOptions[0]
})

const transform = async () => {
  if (!input.value) {
    output.value = ''
    error.value = null
    return
  }

  try {
    const res = await apiClient.encodingTransform(input.value, encoding.value, operation.value)
    if (res.error) {
      error.value = res.error
      output.value = ''
    } else {
      output.value = res.text
      error.value = null
      // Add to history
      history.value.unshift({
        input: input.value,
        output: res.text,
        encoding: encoding.value,
        operation: operation.value,
        timestamp: Date.now()
      })
      // Keep only last 20 items
      if (history.value.length > 20) {
        history.value = history.value.slice(0, 20)
      }
    }
  } catch (e) {
    error.value = 'Transformation failed: ' + (e as Error).message
    console.error(e)
  }
}

// Auto-transform when inputs change
watch([input, encoding, operation], () => {
  transform()
}, { debounce: 300 })

const copyToClipboard = async (text: string, label: string) => {
  try {
    await navigator.clipboard.writeText(text)
    // Could add a toast notification here
    console.log(`${label} copied to clipboard`)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}

const copyInput = () => copyToClipboard(input.value, 'Input')
const copyOutput = () => copyToClipboard(output.value, 'Output')

const swap = async () => {
  isSwapping.value = true
  await new Promise(resolve => setTimeout(resolve, 150))
  const temp = input.value
  input.value = output.value
  output.value = temp
  operation.value = operation.value === 'encode' ? 'decode' : 'encode'
  await new Promise(resolve => setTimeout(resolve, 150))
  isSwapping.value = false
}

const clearAll = () => {
  input.value = ''
  output.value = ''
  error.value = null
}

const addToFavorites = () => {
  const label = prompt('Enter a label for this favorite:')
  if (!label) return
  favorites.value.push({
    input: input.value,
    encoding: encoding.value,
    operation: operation.value,
    label
  })
}

const loadFromHistory = (item: typeof history.value[0]) => {
  input.value = item.input
  encoding.value = item.encoding
  operation.value = item.operation
}

const loadFromFavorite = (item: typeof favorites.value[0]) => {
  input.value = item.input
  encoding.value = item.encoding
  operation.value = item.operation
}

const removeFromHistory = (index: number) => {
  history.value.splice(index, 1)
}

const removeFromFavorites = (index: number) => {
  favorites.value.splice(index, 1)
}

const formatTimestamp = (timestamp: number) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-i3-bg-alt border-b border-i3-border rounded-lg">
      <div class="flex items-center gap-3">
        <Icon name="file-code" size="lg" color="cyan" />
        <h2 class="text-lg font-bold text-i3-text">Decoder / Encoder</h2>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="cyan">{{ operation === 'encode' ? 'Encode' : 'Decode' }}</Badge>
        <Badge :variant="selectedEncoding.icon === 'file-code' ? 'cyan' : 'orange'">
          {{ selectedEncoding.label }}
        </Badge>
      </div>
    </div>

    <!-- Controls Bar -->
    <div class="flex items-center gap-4 px-4 py-3 bg-i3-bg-alt border border-i3-border rounded-lg">
      <!-- Codec Selector -->
      <div class="relative">
        <button
          @click="showEncodingDropdown = !showEncodingDropdown"
          class="flex items-center gap-2 px-4 py-2 bg-i3-bg border border-i3-border rounded-lg hover:border-i3-cyan transition-all duration-200"
        >
          <Icon :name="selectedEncoding.icon" size="sm" />
          <span class="font-medium">{{ selectedEncoding.label }}</span>
          <Icon name="chevron-down" size="xs" :class="{ 'rotate-180': showEncodingDropdown }" />
        </button>

        <!-- Dropdown -->
        <div
          v-if="showEncodingDropdown"
          class="absolute top-full left-0 mt-2 w-64 bg-i3-bg-alt border border-i3-border rounded-lg shadow-xl z-50 overflow-hidden"
        >
          <div class="max-h-80 overflow-auto">
            <button
              v-for="opt in encodingOptions"
              :key="opt.value"
              @click="encoding = opt.value; showEncodingDropdown = false"
              :class="[
                'w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-i3-cyan/10 transition-colors',
                encoding === opt.value ? 'bg-i3-cyan/20 border-l-2 border-i3-cyan' : ''
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
      </div>

      <!-- Operation Toggle -->
      <div class="flex bg-i3-bg rounded-lg border border-i3-border overflow-hidden">
        <button
          @click="operation = 'encode'"
          :class="[
            'px-6 py-2 font-medium transition-all duration-200',
            operation === 'encode'
              ? 'bg-i3-cyan text-i3-bg shadow-glow-cyan'
              : 'text-i3-text hover:bg-i3-bg-alt'
          ]"
        >
          <Icon name="lock" size="xs" />
          <span>Encode</span>
        </button>
        <button
          @click="operation = 'decode'"
          :class="[
            'px-6 py-2 font-medium transition-all duration-200',
            operation === 'decode'
              ? 'bg-i3-magenta text-i3-bg shadow-glow-magenta'
              : 'text-i3-text hover:bg-i3-bg-alt'
          ]"
        >
          <Icon name="unlock" size="xs" />
          <span>Decode</span>
        </button>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-2 ml-auto">
        <Button
          @click="showHistory = !showHistory"
          variant="secondary"
          size="sm"
        >
          <Icon name="history" size="xs" />
          <span>History</span>
          <Badge v-if="history.length > 0" variant="cyan" size="sm">{{ history.length }}</Badge>
        </Button>
        <Button
          @click="addToFavorites"
          variant="secondary"
          size="sm"
          :disabled="!input.value"
        >
          <Icon name="star" size="xs" />
          <span>Favorite</span>
        </Button>
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

    <!-- Main Content -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- Input Panel -->
      <div class="flex-1 flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
            <Icon name="edit" size="xs" />
            <span>Input</span>
          </label>
          <div class="flex items-center gap-2">
            <span class="text-xs text-i3-text-muted">{{ input.length }} chars</span>
            <Button
              v-if="input.value"
              @click="copyInput"
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
            v-model="input"
            class="w-full h-full bg-i3-bg border border-i3-border rounded-lg p-4 font-mono text-sm resize-none focus:outline-none focus:border-i3-cyan focus:ring-2 focus:ring-i3-cyan/20 transition-all duration-200"
            placeholder="Type or paste content here..."
            spellcheck="false"
          ></textarea>
        </div>
      </div>

      <!-- Swap Button -->
      <div class="flex flex-col justify-center">
        <button
          @click="swap"
          :class="[
            'p-3 rounded-lg border transition-all duration-300',
            isSwapping
              ? 'bg-i3-cyan/20 border-i3-cyan shadow-glow-cyan rotate-180'
              : 'bg-i3-bg-alt border-i3-border hover:border-i3-cyan hover:bg-i3-cyan/10'
          ]"
          title="Swap Input/Output"
        >
          <Icon name="arrow-right" size="md" />
        </button>
      </div>

      <!-- Output Panel -->
      <div class="flex-1 flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <label class="text-sm font-medium text-i3-text-secondary flex items-center gap-2">
            <Icon name="file-text" size="xs" />
            <span>Output</span>
          </label>
          <div class="flex items-center gap-2">
            <span class="text-xs text-i3-text-muted">{{ output.length }} chars</span>
            <Button
              v-if="output.value"
              @click="copyOutput"
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
            v-model="output"
            readonly
            class="w-full h-full bg-i3-bg-alt border border-i3-border rounded-lg p-4 font-mono text-sm resize-none focus:outline-none transition-all duration-200"
            :class="{
              'border-i3-red ring-2 ring-i3-red/20': error,
              'border-i3-green': output.value && !error
            }"
            placeholder="Output will appear here..."
          ></textarea>

          <!-- Error Display -->
          <div
            v-if="error"
            class="absolute bottom-4 left-4 right-4 bg-i3-red/10 border border-i3-red rounded-lg p-3"
          >
            <div class="flex items-start gap-2">
              <Icon name="alert-triangle" size="sm" color="red" />
              <div class="flex-1">
                <div class="text-sm font-bold text-i3-red mb-1">Error</div>
                <div class="text-xs text-i3-red/80">{{ error }}</div>
              </div>
              <button
                @click="error = null"
                class="p-1 hover:bg-i3-red/20 rounded transition-colors"
              >
                <Icon name="x" size="xs" />
              </button>
            </div>
          </div>

          <!-- Success Indicator -->
          <div
            v-if="output.value && !error"
            class="absolute top-4 right-4 flex items-center gap-1 px-3 py-1.5 bg-i3-green/10 border border-i3-green rounded-full"
          >
            <Icon name="check-circle" size="xs" color="green" />
            <span class="text-xs font-medium text-i3-green">Success</span>
          </div>
        </div>
      </div>
    </div>

    <!-- History Sidebar -->
    <div
      v-if="showHistory"
      class="fixed right-0 top-0 bottom-0 w-96 bg-i3-bg-alt border-l border-i3-border shadow-xl z-50 flex flex-col"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-i3-border">
        <h3 class="text-lg font-bold text-i3-text flex items-center gap-2">
          <Icon name="history" size="md" color="cyan" />
          <span>History</span>
        </h3>
        <button
          @click="showHistory = false"
          class="p-2 rounded hover:bg-i3-bg transition-colors"
        >
          <Icon name="x" size="md" />
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-i3-border">
        <button
          @click="showHistory = 'recent'"
          :class="[
            'flex-1 px-4 py-2 text-sm font-medium transition-colors',
            showHistory === 'recent' ? 'text-i3-cyan border-b-2 border-i3-cyan' : 'text-i3-text-muted'
          ]"
        >
          Recent
        </button>
        <button
          @click="showHistory = 'favorites'"
          :class="[
            'flex-1 px-4 py-2 text-sm font-medium transition-colors',
            showHistory === 'favorites' ? 'text-i3-cyan border-b-2 border-i3-cyan' : 'text-i3-text-muted'
          ]"
        >
          Favorites
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-auto p-4">
        <!-- Recent History -->
        <div v-if="showHistory === 'recent'" class="space-y-3">
          <div v-if="history.length === 0" class="text-center py-8 text-i3-text-muted">
            <Icon name="history" size="xl" color="muted" class="mb-2" />
            <p>No recent history</p>
          </div>
          <div
            v-for="(item, index) in history"
            :key="index"
            class="bg-i3-bg border border-i3-border rounded-lg p-3 hover:border-i3-cyan/50 transition-colors group"
          >
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <Badge :variant="item.operation === 'encode' ? 'cyan' : 'magenta'" outline size="sm">
                  {{ item.operation }}
                </Badge>
                <Badge variant="gray" outline size="sm">
                  {{ encodingOptions.find(e => e.value === item.encoding)?.label }}
                </Badge>
              </div>
              <div class="flex items-center gap-1">
                <button
                  @click="loadFromHistory(item)"
                  class="p-1.5 rounded hover:bg-i3-cyan/10 transition-colors"
                  title="Load"
                >
                  <Icon name="upload" size="xs" />
                </button>
                <button
                  @click="removeFromHistory(index)"
                  class="p-1.5 rounded hover:bg-i3-red/10 transition-colors"
                  title="Remove"
                >
                  <Icon name="delete" size="xs" />
                </button>
              </div>
            </div>
            <div class="text-xs text-i3-text-muted mb-1">
              {{ formatTimestamp(item.timestamp) }}
            </div>
            <div class="text-sm text-i3-text-secondary truncate mb-1">
              {{ item.input }}
            </div>
            <div class="text-sm text-i3-cyan truncate">
              {{ item.output }}
            </div>
          </div>
        </div>

        <!-- Favorites -->
        <div v-if="showHistory === 'favorites'" class="space-y-3">
          <div v-if="favorites.length === 0" class="text-center py-8 text-i3-text-muted">
            <Icon name="star" size="xl" color="muted" class="mb-2" />
            <p>No favorites yet</p>
            <p class="text-xs mt-1">Click "Favorite" to save a transformation</p>
          </div>
          <div
            v-for="(item, index) in favorites"
            :key="index"
            class="bg-i3-bg border border-i3-border rounded-lg p-3 hover:border-i3-cyan/50 transition-colors group"
          >
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <Icon name="star" size="xs" color="orange" />
                <span class="font-medium text-i3-text">{{ item.label }}</span>
              </div>
              <div class="flex items-center gap-1">
                <button
                  @click="loadFromFavorite(item)"
                  class="p-1.5 rounded hover:bg-i3-cyan/10 transition-colors"
                  title="Load"
                >
                  <Icon name="upload" size="xs" />
                </button>
                <button
                  @click="removeFromFavorites(index)"
                  class="p-1.5 rounded hover:bg-i3-red/10 transition-colors"
                  title="Remove"
                >
                  <Icon name="delete" size="xs" />
                </button>
              </div>
            </div>
            <div class="flex items-center gap-2 mb-2">
              <Badge :variant="item.operation === 'encode' ? 'cyan' : 'magenta'" outline size="sm">
                {{ item.operation }}
              </Badge>
              <Badge variant="gray" outline size="sm">
                {{ encodingOptions.find(e => e.value === item.encoding)?.label }}
              </Badge>
            </div>
            <div class="text-sm text-i3-text-secondary truncate">
              {{ item.input }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
