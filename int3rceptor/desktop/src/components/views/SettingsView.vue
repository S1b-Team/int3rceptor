<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { apiClient } from '../../api/client'
import Button from '../base/Button.vue'

const proxyConfig = ref({
  port: 8080,
  host: '127.0.0.1',
  interceptHttps: true,
  http2: true
})

const ipFilter = ref({
  enabled: false,
  allowList: ['127.0.0.1'],
  blockList: []
})

const uiConfig = ref({
  theme: 'cyberpunk',
  animations: true,
  notifications: true
})

const showCertGuide = ref(false)

onMounted(async () => {
  try {
    const settings = await apiClient.getSettings()
    if (settings) {
      proxyConfig.value = settings.proxy
      uiConfig.value = settings.ui
    }
  } catch (e) {
    console.error('Failed to load settings', e)
  }
})

async function saveSettings() {
  try {
    await apiClient.updateSettings({
      proxy: proxyConfig.value,
      ui: uiConfig.value
    })

    console.log('Settings saved')

    // Show success feedback
    const btn = document.getElementById('save-btn')
    if (btn) {
      const originalText = btn.innerText
      btn.innerText = 'Saved!'
      setTimeout(() => btn.innerText = originalText, 2000)
    }
  } catch (e) {
    console.error('Failed to save settings', e)
    alert('Failed to save settings')
  }
}

async function downloadCert() {
  try {
    const response = await fetch(`${apiClient.baseUrl}/ca-cert`)
    if (!response.ok) throw new Error('Failed to download')
    const blob = await response.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'interceptor-ca.pem'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Failed to download certificate', e)
    alert('Failed to download certificate')
  }
}
</script>

<template>
  <div class="h-full flex flex-col max-w-4xl mx-auto">
    <h2 class="text-2xl font-heading font-bold text-i3-cyan mb-6 flex items-center gap-3">
      <span class="text-3xl">⚙️</span> System Configuration
    </h2>

    <div class="flex-1 overflow-auto space-y-8 pr-4">

      <!-- Proxy Settings -->
      <section class="panel bg-i3-bg-alt/50">
        <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">Proxy Listener</h3>
        <div class="grid grid-cols-2 gap-6">
          <div class="space-y-2">
            <label class="text-sm text-i3-text-secondary">Bind Address</label>
            <input v-model="proxyConfig.host" type="text" class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:border-i3-cyan outline-none font-mono">
          </div>
          <div class="space-y-2">
            <label class="text-sm text-i3-text-secondary">Port</label>
            <input v-model="proxyConfig.port" type="number" class="w-full bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:border-i3-cyan outline-none font-mono">
          </div>

          <div class="col-span-2 flex items-center gap-8 mt-2">
            <label class="flex items-center gap-3 cursor-pointer group">
              <div class="relative">
                <input v-model="proxyConfig.interceptHttps" type="checkbox" class="sr-only peer">
                <div class="w-10 h-6 bg-i3-bg border border-i3-border rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:translate-x-4 peer-checked:bg-i3-cyan transition-all"></div>
              </div>
              <span class="text-i3-text group-hover:text-i3-cyan transition-colors">Intercept HTTPS</span>
            </label>

            <label class="flex items-center gap-3 cursor-pointer group">
              <div class="relative">
                <input v-model="proxyConfig.http2" type="checkbox" class="sr-only peer">
                <div class="w-10 h-6 bg-i3-bg border border-i3-border rounded-full peer-checked:bg-i3-cyan/20 peer-checked:border-i3-cyan transition-all"></div>
                <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:translate-x-4 peer-checked:bg-i3-cyan transition-all"></div>
              </div>
              <span class="text-i3-text group-hover:text-i3-cyan transition-colors">Enable HTTP/2</span>
            </label>
          </div>
        </div>
      </section>

      <!-- TLS Certificate -->
      <section class="panel bg-i3-bg-alt/50">
        <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">🔐 TLS Certificate</h3>
        <p class="text-sm text-i3-text-secondary mb-4">
          To intercept HTTPS traffic, you must install the Interceptor CA certificate in your browser or system.
        </p>
        <div class="flex gap-4">
          <Button variant="primary" @click="showCertGuide = true">
            📖 Installation Guide
          </Button>
          <Button variant="secondary" @click="downloadCert">
            ⬇️ Download Certificate
          </Button>
        </div>
      </section>

      <!-- Security Settings -->
      <section class="panel bg-i3-bg-alt/50">
        <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">Security & Access Control</h3>
        <div class="space-y-4">
          <label class="flex items-center gap-3 cursor-pointer group">
            <div class="relative">
              <input v-model="ipFilter.enabled" type="checkbox" class="sr-only peer">
              <div class="w-10 h-6 bg-i3-bg border border-i3-border rounded-full peer-checked:bg-i3-magenta/20 peer-checked:border-i3-magenta transition-all"></div>
              <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:translate-x-4 peer-checked:bg-i3-magenta transition-all"></div>
            </div>
            <span class="text-i3-text group-hover:text-i3-magenta transition-colors">Enable IP Filtering</span>
          </label>

          <div v-if="ipFilter.enabled" class="space-y-2 animate-fade-in">
            <label class="text-sm text-i3-text-secondary">Allowed IPs (comma separated)</label>
            <textarea class="w-full h-20 bg-i3-bg border border-i3-border rounded p-2 text-i3-text focus:border-i3-magenta outline-none font-mono text-sm">127.0.0.1, ::1</textarea>
          </div>
        </div>
      </section>

      <!-- UI Settings -->
      <section class="panel bg-i3-bg-alt/50">
        <h3 class="text-lg font-bold text-i3-text mb-4 border-b border-i3-border pb-2">Interface</h3>
        <div class="grid grid-cols-2 gap-4">
           <label class="flex items-center gap-3 cursor-pointer group">
              <div class="relative">
                <input v-model="uiConfig.animations" type="checkbox" class="sr-only peer">
                <div class="w-10 h-6 bg-i3-bg border border-i3-border rounded-full peer-checked:bg-i3-orange/20 peer-checked:border-i3-orange transition-all"></div>
                <div class="absolute left-1 top-1 w-4 h-4 bg-i3-text-muted rounded-full peer-checked:translate-x-4 peer-checked:bg-i3-orange transition-all"></div>
              </div>
              <span class="text-i3-text group-hover:text-i3-orange transition-colors">UI Animations</span>
            </label>
        </div>
      </section>

    </div>

    <!-- Actions -->
    <div class="mt-6 pt-4 border-t border-i3-border flex justify-end gap-4">
      <Button variant="secondary">Reset Defaults</Button>
      <Button id="save-btn" variant="primary" @click="saveSettings">Save Configuration</Button>
    </div>
  </div>

  <!-- TLS Certificate Modal -->
  <div v-if="showCertGuide" class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="showCertGuide = false">
    <div class="bg-i3-bg-alt border border-i3-border rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-xl font-bold text-i3-cyan">🔐 Install CA Certificate</h3>
        <button @click="showCertGuide = false" class="text-i3-text-muted hover:text-i3-text">✕</button>
      </div>

      <div class="space-y-4 text-sm">
        <p class="text-i3-text-secondary">
          To intercept HTTPS traffic, you must install the Interceptor CA certificate in your browser or system.
        </p>

        <div class="bg-i3-bg p-4 rounded border border-i3-border">
          <h4 class="font-bold text-i3-orange mb-2">📁 Step 1: Download the certificate</h4>
          <Button variant="primary" @click="downloadCert" class="w-full">
            ⬇️ Download interceptor-ca.pem
          </Button>
        </div>

        <div class="bg-i3-bg p-4 rounded border border-i3-border">
          <h4 class="font-bold text-i3-orange mb-2">🐧 Linux (Firefox)</h4>
          <ol class="list-decimal list-inside space-y-1 text-i3-text-secondary">
            <li>Open Firefox → Settings → Privacy & Security</li>
            <li>Scroll to "Certificates" → View Certificates</li>
            <li>Go to "Authorities" tab → Import</li>
            <li>Select the downloaded <code class="text-i3-cyan">interceptor-ca.pem</code></li>
            <li>Check "Trust this CA to identify websites"</li>
          </ol>
        </div>

        <div class="bg-i3-bg p-4 rounded border border-i3-border">
          <h4 class="font-bold text-i3-orange mb-2">🍎 macOS</h4>
          <ol class="list-decimal list-inside space-y-1 text-i3-text-secondary">
            <li>Double-click the downloaded certificate</li>
            <li>Keychain Access will open → Add to "login" keychain</li>
            <li>Find "Interceptor Proxy CA" in Keychain Access</li>
            <li>Double-click → Trust → Set to "Always Trust"</li>
          </ol>
        </div>

        <div class="bg-i3-bg p-4 rounded border border-i3-border">
          <h4 class="font-bold text-i3-orange mb-2">🪟 Windows</h4>
          <ol class="list-decimal list-inside space-y-1 text-i3-text-secondary">
            <li>Double-click the certificate file</li>
            <li>Click "Install Certificate" → Local Machine</li>
            <li>Select "Place all certificates in the following store"</li>
            <li>Browse → "Trusted Root Certification Authorities"</li>
            <li>Finish and confirm the security warning</li>
          </ol>
        </div>

        <div class="bg-i3-bg p-4 rounded border border-i3-cyan/30">
          <h4 class="font-bold text-i3-cyan mb-2">⚠️ Security Note</h4>
          <p class="text-i3-text-secondary">
            This certificate allows Interceptor to decrypt HTTPS traffic. Only install it on systems you control
            and <strong>remove it when not in use</strong> for auditing purposes.
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
