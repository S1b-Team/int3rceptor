<script setup lang="ts">
import { ref } from 'vue'
import { apiClient } from '../../api/client'
import Badge from '../base/Badge.vue'
import Button from '../base/Button.vue'

const method = ref('GET')
const url = ref('https://api.example.com/v1/users')
const requestHeaders = ref('User-Agent: INT3RCEPTOR/3.0\nAccept: application/json\nContent-Type: application/json')
const requestBody = ref('{\n  "query": "test"\n}')

const response = ref<null | {
  status: number
  statusText: string
  headers: Record<string, string>
  body: string
  time: number
  size: number
}>(null)

const isSending = ref(false)

async function sendRequest() {
  isSending.value = true
  response.value = null

  try {
    // Parse headers
    const headers: Record<string, string> = {}
    requestHeaders.value.split('\n').forEach(line => {
      const [key, ...values] = line.split(':')
      if (key && values.length) {
        headers[key.trim()] = values.join(':').trim()
      }
    })

    // Execute Request via Backend
    const res = await apiClient.sendRepeaterRequest({
      method: method.value,
      url: url.value,
      headers: headers,
      body: (method.value !== 'GET' && method.value !== 'HEAD') ? requestBody.value : undefined
    })

    response.value = {
      status: res.status,
      statusText: res.status_text,
      headers: res.headers,
      body: res.body,
      time: res.time_ms,
      size: res.size_bytes
    }

  } catch (e: any) {
    // Error Handling
    response.value = {
      status: 0,
      statusText: 'ERROR',
      headers: {},
      body: `Error: ${e.message || 'Unknown error occurred'}\n\nCheck backend connection.`,
      time: 0,
      size: 0
    }
  } finally {
    isSending.value = false
  }
}


</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Top Bar: URL & Send -->
    <div class="flex gap-4 mb-4 p-4 panel bg-i3-bg-alt/50 items-center">
      <select v-model="method" class="bg-white text-black border border-i3-border rounded px-3 py-2 font-bold focus:border-i3-cyan outline-none cursor-pointer focus:ring-2 focus:ring-i3-cyan transition-colors">
        <option value="GET">GET</option>
        <option value="POST">POST</option>
        <option value="PUT">PUT</option>
        <option value="DELETE">DELETE</option>
        <option value="PATCH">PATCH</option>
        <option value="HEAD">HEAD</option>
        <option value="OPTIONS">OPTIONS</option>
      </select>

      <input
        v-model="url"
        type="text"
        class="flex-1 bg-i3-bg border border-i3-border rounded px-4 py-2 text-i3-text focus:border-i3-cyan outline-none font-mono placeholder:text-i3-text-muted"
        placeholder="Enter URL..."
      >

      <Button variant="primary" @click="sendRequest" :disabled="isSending" class="w-32 justify-center shadow-neon-cyan">
        <span v-if="!isSending" class="flex items-center gap-2">🚀 Send</span>
        <span v-else class="flex items-center gap-2"><span class="animate-spin">⏳</span> Sending</span>
      </Button>
    </div>

    <!-- Main Split Area -->
    <div class="flex-1 flex gap-4 overflow-hidden">

      <!-- Request Editor (Left) -->
      <div class="flex-1 panel p-0 flex flex-col overflow-hidden border-i3-border/50">
        <div class="bg-i3-bg-alt border-b border-i3-border p-2 font-bold text-i3-text-secondary text-sm flex justify-between items-center">
          <span class="flex items-center gap-2"><span class="text-i3-orange">➜</span> REQUEST</span>
        </div>

        <div class="flex-1 flex flex-col overflow-hidden">
          <!-- Headers Section -->
          <div class="h-1/3 flex flex-col border-b border-i3-border">
            <div class="bg-i3-bg-panel px-3 py-1 text-xs text-i3-text-muted font-bold tracking-wider border-b border-i3-border/30">HEADERS</div>
            <textarea
              v-model="requestHeaders"
              class="flex-1 bg-i3-bg p-3 text-sm font-mono text-i3-text resize-none focus:outline-none focus:bg-i3-bg-alt/20 transition-colors"
              placeholder="Key: Value"
              spellcheck="false"
            ></textarea>
          </div>

          <!-- Body Section -->
          <div class="flex-1 flex flex-col">
            <div class="bg-i3-bg-panel px-3 py-1 text-xs text-i3-text-muted font-bold tracking-wider flex justify-between border-b border-i3-border/30">
              <span>BODY</span>
              <span class="text-[10px] text-i3-text-secondary">JSON</span>
            </div>
            <textarea
              v-model="requestBody"
              class="flex-1 bg-i3-bg p-3 text-sm font-mono text-i3-text-secondary resize-none focus:outline-none focus:bg-i3-bg-alt/20 transition-colors"
              placeholder="{}"
              spellcheck="false"
            ></textarea>
          </div>
        </div>
      </div>

      <!-- Response Viewer (Right) -->
      <div class="flex-1 panel p-0 flex flex-col overflow-hidden relative border-i3-border/50">
        <div class="bg-i3-bg-alt border-b border-i3-border p-2 font-bold text-i3-text-secondary text-sm flex justify-between items-center min-h-[41px]">
          <span class="flex items-center gap-2"><span class="text-i3-cyan">←</span> RESPONSE</span>
          <div v-if="response" class="flex gap-3 text-xs items-center">
            <Badge :variant="response.status < 300 ? 'cyan' : response.status < 400 ? 'orange' : 'magenta'">{{ response.status }} {{ response.statusText }}</Badge>
            <span class="text-i3-text-muted font-mono">{{ response.time }}ms</span>
            <span class="text-i3-text-muted font-mono">{{ response.size }} B</span>
          </div>
        </div>

        <div v-if="response" class="flex-1 flex flex-col overflow-hidden animate-fade-in">
           <!-- Response Headers -->
           <div class="max-h-1/3 overflow-auto border-b border-i3-border bg-i3-bg-panel/30">
             <div v-for="(val, key) in response.headers" :key="key" class="px-3 py-1 text-xs font-mono flex gap-2 hover:bg-white/5">
               <span class="text-i3-cyan font-bold min-w-[120px]">{{ key }}:</span>
               <span class="text-i3-text-secondary break-all">{{ val }}</span>
             </div>
           </div>

           <!-- Response Body -->
           <div class="flex-1 overflow-auto bg-i3-bg p-3">
             <pre class="text-xs font-mono text-i3-text whitespace-pre-wrap">{{ response.body }}</pre>
           </div>
        </div>

        <!-- Empty State -->
        <div v-else class="flex-1 flex flex-col items-center justify-center text-i3-text-muted opacity-30">
          <div class="text-6xl mb-4">📡</div>
          <p class="font-heading tracking-widest">READY TO SEND</p>
        </div>
      </div>

    </div>
  </div>
</template>
