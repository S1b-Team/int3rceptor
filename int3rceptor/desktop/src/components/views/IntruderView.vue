<template>
  <div class="intruder-container">
    <div class="intruder-header">
      <h2>🎯 Intruder - Advanced Payload Fuzzer</h2>
      <p class="subtitle">Automate attacks with precision targeting</p>
    </div>

    <div class="intruder-main">
      <!-- Left Panel: Configuration -->
      <div class="config-panel">
        <section class="template-section">
          <div class="section-header">
            <h3>1. Request Template</h3>
            <button @click="loadFromRepeater" class="btn-secondary btn-sm">
              Load from Selected Request
            </button>
          </div>
          <textarea
            v-model="template"
            placeholder="Paste your HTTP request here...&#10;&#10;Example:&#10;POST /api/login HTTP/1.1&#10;Host: example.com&#10;Content-Type: application/json&#10;&#10;{&#10;  &quot;username&quot;: &quot;§user§&quot;,&#10;  &quot;password&quot;: &quot;§pass§&quot;&#10;}"
            class="template-editor"
            spellcheck="false"
          ></textarea>
          <div class="template-help">
            <span class="help-icon">ℹ️</span>
            <span>Use <code>§name§</code> to mark payload positions</span>
          </div>
        </section>

        <section class="positions-section">
          <h3>2. Detected Positions</h3>
          <div v-if="detectedPositions.length === 0" class="empty-state">
            No positions detected. Add §markers§ to your template.
          </div>
          <div v-else class="positions-list">
            <div
              v-for="(pos, idx) in detectedPositions"
              :key="idx"
              class="position-item"
            >
              <span class="position-badge">{{ idx + 1 }}</span>
              <code class="position-name">§{{ pos.name }}§</code>
            </div>
          </div>
        </section>

        <section class="attack-type-section">
          <h3>3. Attack Type</h3>
          <div class="attack-types">
            <label
              v-for="type in attackTypes"
              :key="type.value"
              class="attack-type-card"
              :class="{ active: attackType === type.value }"
            >
              <input
                type="radio"
                :value="type.value"
                v-model="attackType"
                hidden
              />
              <div class="card-icon">{{ type.icon }}</div>
              <div class="card-title">{{ type.label }}</div>
              <div class="card-description">{{ type.description }}</div>
            </label>
          </div>
        </section>

        <section class="payloads-section">
          <div class="section-header">
            <h3>4. Payloads</h3>
            <div class="payload-actions">
              <button @click="loadCommonPayloads" class="btn-secondary btn-sm">
                Load Common
              </button>
              <button @click="clearPayloads" class="btn-secondary btn-sm">
                Clear
              </button>
            </div>
          </div>
          <textarea
            v-model="payloadsText"
            placeholder="Enter payloads (one per line)...&#10;&#10;Example:&#10;admin&#10;root&#10;test&#10;' OR '1'='1&#10;1; DROP TABLE users--"
            class="payloads-editor"
            spellcheck="false"
          ></textarea>
          <div class="payload-count">
            {{ payloadsList.length }} payload{{ payloadsList.length !== 1 ? 's' : '' }} loaded
          </div>
        </section>

        <section class="options-section">
          <h3>5. Options</h3>
          <div class="options-grid">
            <div class="option-item">
              <label>Concurrency</label>
              <input type="number" v-model.number="concurrency" min="1" max="100" />
            </div>
            <div class="option-item">
              <label>Delay (ms)</label>
              <input type="number" v-model.number="delay" min="0" />
            </div>
          </div>
        </section>

        <section class="launch-section">
          <div class="launch-actions">
            <button
              @click="startAttack"
              :disabled="!canGenerate || running"
              class="btn-primary btn-large"
            >
              <span v-if="running">🚀 Attack Running...</span>
              <span v-else>🚀 Start Attack ({{ estimatedRequests }} requests)</span>
            </button>
            <button
              v-if="running"
              @click="stopAttack"
              class="btn-danger btn-large"
            >
              🛑 Stop Attack
            </button>
          </div>
        </section>
      </div>

      <!-- Right Panel: Results -->
      <div class="results-panel">
        <div class="results-header">
          <h3>Attack Results</h3>
          <div class="results-actions">
            <button @click="refreshResults" class="btn-secondary btn-sm">
              🔄 Refresh
            </button>
            <button @click="clearResults" class="btn-secondary btn-sm">
              🗑️ Clear
            </button>
          </div>
        </div>

        <div v-if="results.length === 0" class="empty-results">
          <div class="empty-icon">📊</div>
          <p>No results yet</p>
          <p class="empty-hint">Generate an attack to see results here</p>
        </div>

        <div v-else class="results-table-container">
          <table class="results-table">
            <thead>
              <tr>
                <th>#</th>
                <th>Payload</th>
                <th>Status</th>
                <th>Length</th>
                <th>Time (ms)</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(result, idx) in results"
                :key="idx"
                :class="getResultClass(result)"
              >
                <td>{{ idx + 1 }}</td>
                <td class="payload-cell">
                  <code>{{ result.payload }}</code>
                </td>
                <td>
                  <span class="status-badge" :class="`status-${getStatusClass(result.status_code)}`">
                    {{ result.status_code }}
                  </span>
                </td>
                <td>{{ formatBytes(result.response_length) }}</td>
                <td>{{ result.duration_ms }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-if="generatedRequests.length > 0" class="generated-preview">
          <h4>Generated Requests Preview</h4>
          <div class="preview-count">
            {{ generatedRequests.length }} request{{ generatedRequests.length !== 1 ? 's' : '' }} generated
          </div>
          <div class="preview-list">
            <details v-for="(req, idx) in generatedRequests.slice(0, 5)" :key="idx">
              <summary>Request {{ idx + 1 }}</summary>
              <pre class="request-preview">{{ req }}</pre>
            </details>
            <div v-if="generatedRequests.length > 5" class="preview-more">
              ... and {{ generatedRequests.length - 5 }} more
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { apiClient, type AttackType, type IntruderConfig, type IntruderResult } from '../../api/client';

const template = ref('');
const payloadsText = ref('');
const attackType = ref<AttackType>('Sniper');
const running = ref(false);
const results = ref<IntruderResult[]>([]);
const generatedRequests = ref<string[]>([]);
const concurrency = ref(1);
const delay = ref(0);
const pollingInterval = ref<number | null>(null);

const attackTypes = [
  {
    value: 'Sniper' as AttackType,
    label: 'Sniper',
    icon: '🎯',
    description: 'One payload set, iterate each position',
  },
  {
    value: 'Battering' as AttackType,
    label: 'Battering Ram',
    icon: '🏰',
    description: 'Same payload in all positions',
  },
  {
    value: 'Pitchfork' as AttackType,
    label: 'Pitchfork',
    icon: '🔱',
    description: 'Multiple sets, parallel iteration',
  },
  {
    value: 'ClusterBomb' as AttackType,
    label: 'Cluster Bomb',
    icon: '💣',
    description: 'All combinations (Cartesian product)',
  },
];

const detectedPositions = computed(() => {
  const regex = /§(\w+)§/g;
  const matches = [...template.value.matchAll(regex)];
  const unique = new Set(matches.map(m => m[1]).filter((n): n is string => !!n));
  return Array.from(unique).map((name) => ({
    name,
    start: 0,
    end: 0,
  }));
});

const payloadsList = computed(() => {
  return payloadsText.value
    .split('\n')
    .map(p => p.trim())
    .filter(p => p.length > 0);
});

const estimatedRequests = computed(() => {
  const positions = detectedPositions.value.length;
  const payloads = payloadsList.value.length;

  if (positions === 0 || payloads === 0) return 0;

  switch (attackType.value) {
    case 'Sniper':
      return positions * payloads;
    case 'Battering':
      return payloads;
    case 'Pitchfork':
      return payloads;
    case 'ClusterBomb':
      return Math.pow(payloads, positions);
    default:
      return 0;
  }
});

const canGenerate = computed(() => {
  return (
    template.value.trim().length > 0 &&
    detectedPositions.value.length > 0 &&
    payloadsList.value.length > 0
  );
});

const startAttack = async () => {
  if (!canGenerate.value) return;

  running.value = true;
  generatedRequests.value = [];
  results.value = [];

  try {
    const config: IntruderConfig = {
      positions: detectedPositions.value,
      payloads: payloadsList.value,
      attack_type: attackType.value,
      options: {
        concurrency: concurrency.value,
        delay_ms: delay.value,
      },
    };

    // First generate preview (optional, but good for UI)
    try {
      const preview = await apiClient.intruderGenerate({ template: template.value, config });
      generatedRequests.value = preview.requests;
    } catch (e) {
      console.warn("Failed to generate preview", e);
    }

    await apiClient.intruderStart({
      template: template.value,
      config,
    });

    // Start polling
    startPolling();
  } catch (error) {
    console.error('Failed to start attack:', error);
    alert('Failed to start attack. Check console for details.');
    running.value = false;
  }
};

const stopAttack = async () => {
  try {
    await apiClient.intruderStop();
  } catch (error) {
    console.error('Failed to stop attack:', error);
  } finally {
    stopPolling();
    running.value = false;
  }
};

const startPolling = () => {
  if (pollingInterval.value) clearInterval(pollingInterval.value);
  pollingInterval.value = window.setInterval(async () => {
    await refreshResults();
    // TODO: Check if attack finished from backend status
  }, 500);
};

const stopPolling = () => {
  if (pollingInterval.value) {
    clearInterval(pollingInterval.value);
    pollingInterval.value = null;
  }
};

const refreshResults = async () => {
  try {
    results.value = await apiClient.intruderResults();
  } catch (error) {
    console.error('Failed to fetch results:', error);
  }
};

const clearResults = async () => {
  try {
    await apiClient.intruderClear();
    results.value = [];
    generatedRequests.value = [];
  } catch (error) {
    console.error('Failed to clear results:', error);
  }
};

const loadFromRepeater = () => {
  // TODO: Integrate with selected request from traffic tab
  template.value = `POST /api/login HTTP/1.1
Host: example.com
Content-Type: application/json

{
  "username": "§user§",
  "password": "§pass§"
}`;
};

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
..\\..\\..\\windows\\system32\\config\\sam`;
};

const clearPayloads = () => {
  payloadsText.value = '';
};

const getResultClass = (result: IntruderResult) => {
  const status = Math.floor(result.status_code / 100);
  return {
    'result-success': status === 2,
    'result-redirect': status === 3,
    'result-client-error': status === 4,
    'result-server-error': status === 5,
  };
};

const getStatusClass = (statusCode: number) => {
  return Math.floor(statusCode / 100);
};

const formatBytes = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

onMounted(() => {
  refreshResults();
});

onUnmounted(() => {
  stopPolling();
});
</script>

<style scoped>
.intruder-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  color: #e2e8f0;
}

.intruder-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.2);
  background: linear-gradient(135deg, rgba(56, 189, 248, 0.1), rgba(139, 92, 246, 0.1));
}

.intruder-header h2 {
  margin: 0 0 0.5rem 0;
  color: #38bdf8;
  font-size: 1.5rem;
}

.subtitle {
  margin: 0;
  color: #94a3b8;
  font-size: 0.9rem;
}

.intruder-main {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  overflow: hidden;
}

.config-panel,
.results-panel {
  padding: 1.5rem;
  overflow-y: auto;
}

.config-panel {
  border-right: 1px solid rgba(148, 163, 184, 0.2);
}

section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.section-header h3 {
  margin: 0;
  color: #e2e8f0;
  font-size: 1.1rem;
}

.payload-actions {
  display: flex;
  gap: 0.5rem;
}

h3 {
  color: #e2e8f0;
  margin-bottom: 0.75rem;
  font-size: 1.1rem;
}

.template-editor,
.payloads-editor {
  width: 100%;
  min-height: 200px;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: 6px;
  padding: 1rem;
  color: #e2e8f0;
  font-family: 'Fira Code', 'Courier New', monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  resize: vertical;
}

.template-editor:focus,
.payloads-editor:focus {
  outline: none;
  border-color: #38bdf8;
  box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.1);
}

.template-help {
  margin-top: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #94a3b8;
  font-size: 0.85rem;
}

.help-icon {
  font-size: 1rem;
}

.template-help code {
  background: rgba(56, 189, 248, 0.1);
  padding: 0.2rem 0.4rem;
  border-radius: 3px;
  color: #38bdf8;
  font-family: 'Fira Code', monospace;
}

.empty-state {
  padding: 2rem;
  text-align: center;
  color: #64748b;
  background: rgba(15, 23, 42, 0.4);
  border: 1px dashed rgba(148, 163, 184, 0.3);
  border-radius: 6px;
}

.positions-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.position-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
}

.position-badge {
  background: #38bdf8;
  color: #0f172a;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 700;
}

.position-name {
  color: #38bdf8;
  font-family: 'Fira Code', monospace;
  font-size: 0.9rem;
}

.attack-types {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.attack-type-card {
  background: rgba(15, 23, 42, 0.6);
  border: 2px solid rgba(148, 163, 184, 0.2);
  border-radius: 8px;
  padding: 1rem;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
}

.attack-type-card:hover {
  border-color: rgba(56, 189, 248, 0.5);
  transform: translateY(-2px);
}

.attack-type-card.active {
  background: rgba(56, 189, 248, 0.1);
  border-color: #38bdf8;
  box-shadow: 0 0 20px rgba(56, 189, 248, 0.2);
}

.card-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.card-title {
  font-weight: 600;
  color: #e2e8f0;
  margin-bottom: 0.25rem;
}

.card-description {
  font-size: 0.8rem;
  color: #94a3b8;
}

.payload-count {
  margin-top: 0.5rem;
  color: #94a3b8;
  font-size: 0.85rem;
}

.btn-primary,
.btn-secondary {
  border: none;
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s;
}

.btn-primary {
  background: linear-gradient(135deg, #38bdf8, #8b5cf6);
  color: white;
  width: 100%;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(56, 189, 248, 0.3);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: transparent;
  border: 1px solid rgba(148, 163, 184, 0.4);
  color: #94a3b8;
}

.btn-secondary:hover {
  border-color: #38bdf8;
  color: #38bdf8;
}

.btn-sm {
  padding: 0.4rem 0.8rem;
  font-size: 0.85rem;
}

.btn-large {
  padding: 1rem 1.5rem;
  font-size: 1.1rem;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.results-header h3 {
  margin: 0;
}

.results-actions {
  display: flex;
  gap: 0.5rem;
}

.empty-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: #64748b;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-hint {
  font-size: 0.9rem;
  margin-top: 0.5rem;
}

.results-table-container {
  overflow-x: auto;
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 6px;
}

.results-table {
  width: 100%;
  border-collapse: collapse;
}

.results-table thead {
  background: rgba(15, 23, 42, 0.8);
  position: sticky;
  top: 0;
}

.results-table th {
  padding: 0.75rem;
  text-align: left;
  font-weight: 600;
  color: #94a3b8;
  border-bottom: 1px solid rgba(148, 163, 184, 0.2);
}

.results-table td {
  padding: 0.75rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
}

.results-table tbody tr:hover {
  background: rgba(56, 189, 248, 0.05);
}

.result-success {
  background: rgba(34, 197, 94, 0.05);
}

.result-client-error {
  background: rgba(251, 191, 36, 0.05);
}

.result-server-error {
  background: rgba(239, 68, 68, 0.05);
}

.payload-cell code {
  background: rgba(56, 189, 248, 0.1);
  padding: 0.2rem 0.4rem;
  border-radius: 3px;
  color: #38bdf8;
  font-family: 'Fira Code', monospace;
  font-size: 0.85rem;
}

.status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.85rem;
}

.status-2 {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
}

.status-3 {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

.status-4 {
  background: rgba(251, 191, 36, 0.2);
  color: #fbbf24;
}

.status-5 {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.generated-preview {
  margin-top: 2rem;
  padding: 1rem;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 6px;
}

.generated-preview h4 {
  margin: 0 0 1rem 0;
  color: #e2e8f0;
}

.preview-count {
  color: #94a3b8;
  font-size: 0.9rem;
  margin-bottom: 1rem;
}

.preview-list details {
  margin-bottom: 0.5rem;
}

.preview-list summary {
  cursor: pointer;
  padding: 0.5rem;
  background: rgba(56, 189, 248, 0.1);
  border-radius: 4px;
  color: #38bdf8;
  font-weight: 600;
}

.preview-list summary:hover {
  background: rgba(56, 189, 248, 0.2);
}

.request-preview {
  margin: 0.5rem 0 0 0;
  padding: 1rem;
  background: rgba(15, 23, 42, 0.8);
  border-radius: 4px;
  font-family: 'Fira Code', monospace;
  font-size: 0.85rem;
  overflow-x: auto;
  color: #e2e8f0;
}

.preview-more {
  padding: 0.5rem;
  text-align: center;
  color: #64748b;
  font-style: italic;
}

.options-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.option-item {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.option-item label {
  color: #94a3b8;
  font-size: 0.9rem;
}

.option-item input {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(148, 163, 184, 0.3);
  padding: 0.5rem;
  border-radius: 4px;
  color: #e2e8f0;
}

.launch-actions {
  display: flex;
  gap: 1rem;
}

.btn-danger {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.4);
  padding: 1rem 1.5rem;
  font-size: 1.1rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s;
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.3);
  border-color: #f87171;
}
</style>
