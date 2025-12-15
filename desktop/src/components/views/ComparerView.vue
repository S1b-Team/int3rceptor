<script setup lang="ts">
import { ref } from 'vue'
import { apiClient, type CompareMode, type DiffChange } from '../../api/client'

const left = ref('')
const right = ref('')
const mode = ref<CompareMode>('lines')
const changes = ref<DiffChange[]>([])
const hasDiff = ref(false)

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
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold text-i3-cyan">⚖️ Comparer</h2>

      <div class="flex gap-2">
        <select v-model="mode" class="bg-i3-bg-alt border border-i3-border rounded px-3 py-1 text-i3-text focus:outline-none focus:border-i3-cyan">
          <option value="lines">Lines</option>
          <option value="words">Words</option>
          <option value="chars">Chars</option>
        </select>

        <button
          @click="compare"
          class="px-4 py-1 bg-i3-cyan text-i3-bg font-bold rounded hover:bg-i3-cyan/80 transition-colors"
        >
          Compare
        </button>

        <button
          @click="clear"
          class="px-4 py-1 bg-i3-bg-alt border border-i3-border text-i3-text rounded hover:bg-i3-bg transition-colors"
        >
          Clear
        </button>
      </div>
    </div>

    <div class="flex-1 flex gap-4 min-h-0 overflow-hidden">
      <!-- Inputs -->
      <div v-if="!hasDiff" class="flex-1 flex gap-4">
        <div class="flex-1 flex flex-col gap-2">
          <label class="text-sm font-medium text-i3-text-muted">Original (Left)</label>
          <textarea
            v-model="left"
            class="flex-1 bg-i3-bg border border-i3-border rounded p-3 font-mono text-sm resize-none focus:outline-none focus:border-i3-cyan"
            placeholder="Paste original text..."
          ></textarea>
        </div>
        <div class="flex-1 flex flex-col gap-2">
          <label class="text-sm font-medium text-i3-text-muted">Modified (Right)</label>
          <textarea
            v-model="right"
            class="flex-1 bg-i3-bg border border-i3-border rounded p-3 font-mono text-sm resize-none focus:outline-none focus:border-i3-cyan"
            placeholder="Paste modified text..."
          ></textarea>
        </div>
      </div>

      <!-- Diff Output -->
      <div v-else class="flex-1 flex flex-col gap-2 min-h-0">
        <div class="flex justify-between items-center">
          <label class="text-sm font-medium text-i3-text-muted">Differences</label>
          <button @click="hasDiff = false" class="text-xs text-i3-cyan hover:underline">Edit Inputs</button>
        </div>

        <div class="flex-1 bg-i3-bg border border-i3-border rounded overflow-auto p-4 font-mono text-sm whitespace-pre-wrap">
          <template v-for="(change, i) in changes" :key="i">
            <span
              v-if="change.tag === 'equal'"
              class="text-i3-text-secondary"
            >{{ change.value }}</span>
            <span
              v-else-if="change.tag === 'delete'"
              class="bg-red-500/20 text-red-400 line-through decoration-red-400/50"
            >{{ change.value }}</span>
            <span
              v-else-if="change.tag === 'insert'"
              class="bg-green-500/20 text-green-400"
            >{{ change.value }}</span>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
