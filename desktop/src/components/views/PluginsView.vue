<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { apiClient } from '../../api/client'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'

const plugins = ref<any[]>([])

onMounted(async () => {
  await loadPlugins()
})

async function loadPlugins() {
  try {
    const data = await apiClient.getPlugins()
    plugins.value = data
  } catch (e) {
    console.error('Failed to load plugins', e)
  }
}

async function togglePlugin(plugin: any) {
  try {
    await apiClient.togglePlugin(plugin.name, !plugin.enabled)
    await loadPlugins()
  } catch (e) {
    console.error('Failed to toggle plugin', e)
  }
}

async function reloadPlugin(plugin: any) {
  try {
    await apiClient.reloadPlugin(plugin.name)
    await loadPlugins()
  } catch (e) {
    console.error('Failed to reload plugin', e)
  }
}

function removePlugin(plugin: any) {
  // Not implemented in backend yet
  console.log('Remove plugin:', plugin.name)
}

const fileInput = ref<HTMLInputElement | null>(null)

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
    const file = target.files?.[0]
    if (!file) return
    try {
      await apiClient.uploadPlugin(file)
      await loadPlugins()
      // Reset input
      target.value = ''
    } catch (e) {
      console.error('Failed to upload plugin', e)
      alert('Failed to upload plugin: ' + e)
    }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex justify-between items-center mb-6">
      <div>
        <h2 class="text-2xl font-heading font-bold text-i3-cyan flex items-center gap-3">
          <span class="text-3xl">🧩</span> PLUGINS
        </h2>
        <p class="text-i3-text-dim text-sm mt-1 font-mono">WASM EXTENSION MANAGER</p>
      </div>
      <div class="flex gap-3">
        <Button variant="secondary" @click="loadPlugins">
          <span class="mr-2">↻</span> Refresh
        </Button>
        <input
          type="file"
          ref="fileInput"
          accept=".wasm"
          class="hidden"
          @change="handleFileUpload"
        />
        <Button variant="primary" @click="fileInput?.click()">
          <span class="mr-2">+</span> Load Plugin
        </Button>
      </div>
    </div>

    <!-- Plugin List -->
    <div class="flex-1 overflow-y-auto pr-2 space-y-4">
      <div v-if="plugins.length === 0" class="text-center py-12 border-2 border-dashed border-i3-border/30 rounded-lg">
        <div class="text-4xl mb-4 opacity-50">🧩</div>
        <h3 class="text-xl font-heading text-i3-text-dim mb-2">No Plugins Loaded</h3>
        <p class="text-i3-text-dim/60 max-w-md mx-auto">
          Load WASM plugins to extend INT3RCEPTOR's capabilities.
          <br>Plugins can intercept traffic, decode payloads, and more.
        </p>
      </div>

      <div v-for="plugin in plugins" :key="plugin.name"
           class="bg-i3-bg-soft border border-i3-border rounded-lg p-4 hover:border-i3-cyan/50 transition-colors group relative overflow-hidden">

        <!-- Active Indicator -->
        <div class="absolute top-0 left-0 w-1 h-full transition-colors"
             :class="plugin.enabled ? 'bg-i3-cyan shadow-[0_0_10px_rgba(6,182,212,0.5)]' : 'bg-i3-border'"></div>

        <div class="flex justify-between items-start pl-3">
          <div class="flex-1">
            <div class="flex items-center gap-3 mb-1">
              <h3 class="text-lg font-bold text-i3-text-bright font-heading tracking-wide">
                {{ plugin.name }}
              </h3>
              <Badge :variant="plugin.enabled ? 'cyan' : 'gray'">
                {{ plugin.enabled ? 'ACTIVE' : 'DISABLED' }}
              </Badge>
              <span class="text-xs font-mono text-i3-text-dim bg-black/30 px-2 py-0.5 rounded">
                v{{ plugin.version }}
              </span>
            </div>
            <p class="text-i3-text-dim text-sm mb-3 font-mono leading-relaxed">
              {{ plugin.description }}
            </p>

            <!-- Hooks (Mockup for now) -->
            <div class="flex gap-2">
              <span class="text-[10px] uppercase font-bold text-i3-primary/70 bg-i3-primary/10 px-1.5 py-0.5 rounded border border-i3-primary/20">
                onRequest
              </span>
              <span class="text-[10px] uppercase font-bold text-i3-secondary/70 bg-i3-secondary/10 px-1.5 py-0.5 rounded border border-i3-secondary/20">
                onResponse
              </span>
            </div>
          </div>

          <div class="flex items-center gap-2 opacity-80 group-hover:opacity-100 transition-opacity">
            <button @click="togglePlugin(plugin)"
                    class="p-2 rounded hover:bg-i3-bg-hover transition-colors"
                    :title="plugin.enabled ? 'Disable' : 'Enable'">
              <span v-if="plugin.enabled" class="text-i3-cyan">◼ Stop</span>
              <span v-else class="text-i3-green">▶ Start</span>
            </button>
            <div class="w-px h-6 bg-i3-border mx-1"></div>
            <button @click="reloadPlugin(plugin)" class="p-2 text-i3-text-dim hover:text-i3-text-bright hover:bg-i3-bg-hover rounded transition-colors" title="Reload">
              ↻
            </button>
            <button @click="removePlugin(plugin)" class="p-2 text-i3-red hover:bg-i3-red/10 rounded transition-colors" title="Remove">
              🗑
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
