<script setup lang="ts">
import { ref, watch } from 'vue'
import { apiClient, type EncodingType, type TransformOperation } from '../../api/client'

const input = ref('')
const output = ref('')
const encoding = ref<EncodingType>('base64')
const operation = ref<TransformOperation>('encode')
const error = ref<string | null>(null)

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
    }
  } catch (e) {
    error.value = 'Transformation failed'
    console.error(e)
  }
}

// Auto-transform when inputs change
watch([input, encoding, operation], () => {
  transform()
})

const copyOutput = () => {
  navigator.clipboard.writeText(output.value)
}

const swap = () => {
  input.value = output.value
  operation.value = operation.value === 'encode' ? 'decode' : 'encode'
}
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold text-i3-cyan">🧩 Decoder / Encoder</h2>

      <div class="flex gap-4 bg-i3-bg-alt p-2 rounded border border-i3-border">
        <select v-model="encoding" class="bg-i3-bg border border-i3-border rounded px-2 py-1 text-i3-text focus:outline-none focus:border-i3-cyan">
          <option value="base64">Base64</option>
          <option value="url">URL</option>
          <option value="hex">Hex</option>
          <option value="html">HTML</option>
          <option value="rot13">Rot13</option>
        </select>

        <div class="flex bg-i3-bg rounded border border-i3-border overflow-hidden">
          <button
            @click="operation = 'encode'"
            :class="['px-3 py-1 transition-colors', operation === 'encode' ? 'bg-i3-cyan text-i3-bg font-bold' : 'text-i3-text hover:bg-i3-bg-alt']"
          >
            Encode
          </button>
          <button
            @click="operation = 'decode'"
            :class="['px-3 py-1 transition-colors', operation === 'decode' ? 'bg-i3-cyan text-i3-bg font-bold' : 'text-i3-text hover:bg-i3-bg-alt']"
          >
            Decode
          </button>
        </div>
      </div>
    </div>

    <div class="flex-1 flex gap-4 min-h-0">
      <!-- Input -->
      <div class="flex-1 flex flex-col gap-2">
        <label class="text-sm font-medium text-i3-text-muted">Input</label>
        <textarea
          v-model="input"
          class="flex-1 bg-i3-bg border border-i3-border rounded p-3 font-mono text-sm resize-none focus:outline-none focus:border-i3-cyan"
          placeholder="Type or paste content here..."
        ></textarea>
      </div>

      <!-- Controls -->
      <div class="flex flex-col justify-center gap-2">
        <button
          @click="swap"
          class="p-2 bg-i3-bg-alt border border-i3-border rounded hover:text-i3-cyan hover:border-i3-cyan transition-colors"
          title="Swap Input/Output"
        >
          🔄
        </button>
      </div>

      <!-- Output -->
      <div class="flex-1 flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <label class="text-sm font-medium text-i3-text-muted">Output</label>
          <button
            v-if="output"
            @click="copyOutput"
            class="text-xs text-i3-cyan hover:underline"
          >
            Copy
          </button>
        </div>
        <div class="flex-1 relative">
          <textarea
            v-model="output"
            readonly
            class="w-full h-full bg-i3-bg-alt border border-i3-border rounded p-3 font-mono text-sm resize-none focus:outline-none"
            :class="{ 'border-red-500': error }"
          ></textarea>
          <div v-if="error" class="absolute bottom-2 left-2 right-2 bg-red-500/10 border border-red-500 text-red-500 text-xs p-2 rounded">
            {{ error }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
