<template>
  <div class="repeater">
    <h2>Repeater</h2>
    <p v-if="!entry">Select a request to load it into the repeater.</p>
    <div v-else class="editor">
      <div class="field-row">
        <label>Method</label>
        <input v-model="method" type="text" />
      </div>
      <div class="field-row">
        <label>URL</label>
        <input v-model="url" type="text" />
      </div>

      <div class="headers">
        <div class="headers-header">
          <label>Headers</label>
          <div class="header-actions">
            <select v-model="presetSelection" @change="applyPreset">
              <option value="">Add preset…</option>
              <option v-for="preset in presetOptions" :key="preset" :value="preset">
                {{ preset }}
              </option>
            </select>
            <button type="button" class="ghost" @click="addHeader">Custom Header</button>
          </div>
        </div>
        <div v-for="(header, index) in headers" :key="index" class="header-row">
          <input v-model="header.name" placeholder="Name" />
          <input v-model="header.value" placeholder="Value" />
          <button type="button" class="ghost" @click="removeHeader(index)">✕</button>
        </div>
      </div>

      <label>Body</label>
      <textarea v-model="body"></textarea>

      <div class="actions">
        <button :disabled="sending" @click="send">
          {{ sending ? 'Sending…' : 'Send' }}
        </button>
        <span v-if="status" :class="['status', status.ok ? 'ok' : 'warn']">
          {{ status.message }}
        </span>
      </div>

      <div v-if="lastResponse" class="response-preview">
        <h3>Last Response</h3>
        <p>Status {{ lastResponse.status }} · {{ lastResponse.duration_ms }}ms</p>
        <details>
          <summary>Headers</summary>
          <ul>
            <li v-for="([name, value], idx) in lastResponse.headers" :key="idx">
              <strong>{{ name }}:</strong> {{ value }}
            </li>
          </ul>
        </details>
        <label>Body Preview</label>
        <pre>{{ decodePreview(lastResponse.body_preview) }}</pre>
      </div>
    </div>

    <section v-if="history.length" class="history">
      <h3>Replay History</h3>
      <ul>
        <li
          v-for="item in history"
          :key="item.id + '-' + item.timestamp_ms"
          :class="item.ok ? 'ok' : 'warn'"
        >
          <span>#{{ item.id }} · {{ item.status }} · {{ item.duration_ms }}ms</span>
          <small>{{ new Date(item.timestamp_ms).toLocaleTimeString() }}</small>
        </li>
      </ul>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, toRefs, watch, computed } from 'vue';
import type { CaptureEntry, HeaderField, ReplayResult } from '@/types';
import { useApi } from '@/composables/useApi';

const HISTORY_KEY = 'interceptor_repeater_history';
const PRESET_HEADERS = ['Authorization', 'User-Agent', 'Content-Type', 'Accept', 'Cookie', 'Referer'];

type HistoryEntry = ReplayResult & { ok: boolean };

const props = defineProps<{ entry: CaptureEntry | null }>();
const { entry } = toRefs(props);
const { repeatRequest } = useApi();

const method = ref('');
const url = ref('');
const headers = ref<HeaderField[]>([]);
const body = ref('');
const sending = ref(false);
const status = ref<{ ok: boolean; message: string } | null>(null);
const history = ref<HistoryEntry[]>([]);
const presetSelection = ref('');

onMounted(() => {
  if (typeof window === 'undefined') return;
  try {
    const raw = window.localStorage.getItem(HISTORY_KEY);
    if (raw) {
      const parsed: any[] = JSON.parse(raw);
      history.value = parsed.map((item) => ({
        ...item,
        ok: item.ok ?? (item.status >= 200 && item.status < 400),
      }));
    }
  } catch (error) {
    console.warn('Failed to load repeater history', error);
  }
});

watch(
  () => entry.value,
  (value) => {
    if (!value) {
      method.value = '';
      url.value = '';
      body.value = '';
      headers.value = [];
      status.value = null;
      return;
    }
    method.value = value.request.method;
    url.value = value.request.url;
    headers.value = value.request.headers.map(([name, value]) => ({ name, value }));
    body.value = decodeBody(value.request.body);
    status.value = null;
  },
  { immediate: true },
);

const lastResponse = computed(() => history.value[0] ?? null);

const addHeader = () => {
  headers.value.push({ name: '', value: '' });
};

const applyPreset = () => {
  if (!presetSelection.value) {
    return;
  }
  headers.value.push({ name: presetSelection.value, value: '' });
  presetSelection.value = '';
};

const removeHeader = (index: number) => {
  headers.value.splice(index, 1);
};

const persistHistory = () => {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value));
  } catch (error) {
    console.warn('Failed to persist repeater history', error);
  }
};

const send = async () => {
  if (!entry.value) {
    return;
  }
  sending.value = true;
  status.value = null;
  try {
    const response = await repeatRequest(entry.value.request.id, {
      method: method.value,
      url: url.value,
      headers: headers.value.filter((h) => h.name.trim().length),
      body: body.value,
    });
    const result: HistoryEntry = {
      ...response,
      ok: response.status >= 200 && response.status < 400,
    };
    history.value = [result, ...history.value].slice(0, 10);
    persistHistory();
    status.value = { ok: result.ok, message: result.ok ? 'Replay sent' : 'Replay returned error' };
  } catch (error) {
    console.error(error);
    status.value = { ok: false, message: 'Replay failed' };
  } finally {
    sending.value = false;
  }
};

const decodeBody = (payload: number[]) => {
  if (!payload.length) {
    return '';
  }
  try {
    return new TextDecoder().decode(new Uint8Array(payload));
  } catch (error) {
    return '';
  }
};

const decodePreview = (preview: string) => {
  if (!preview) {
    return '<empty>';
  }
  try {
    if (typeof window === 'undefined') {
      return '[preview available in browser]';
    }
    const raw = window.atob(preview);
    const bytes = Uint8Array.from(raw, (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch (error) {
    return '[binary data]';
  }
};

const presetOptions = PRESET_HEADERS;
</script>

<style scoped>
.repeater {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.field-row,
.headers-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  justify-content: space-between;
}

label {
  font-size: 0.85rem;
  text-transform: uppercase;
  color: #94a3b8;
}

input,
textarea,
select {
  width: 100%;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 4px;
  padding: 0.5rem;
  color: #e2e8f0;
}

textarea {
  min-height: 140px;
}

.headers {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 6px;
  padding: 0.5rem;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
}

.header-row {
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

button {
  align-self: flex-start;
  background: rgba(148, 163, 184, 0.3);
  border: none;
  padding: 0.4rem 1.2rem;
  border-radius: 4px;
  color: #cbd5f5;
  cursor: pointer;
}

button.ghost {
  background: transparent;
  border: 1px dashed rgba(148, 163, 184, 0.5);
  padding: 0.25rem 0.6rem;
}

.actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.status.ok {
  color: #4ade80;
}

.status.warn {
  color: #f87171;
}

.response-preview {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 6px;
  padding: 0.75rem;
  background: rgba(15, 23, 42, 0.6);
}

.response-preview pre {
  max-height: 200px;
  overflow: auto;
  background: rgba(15, 23, 42, 0.9);
  padding: 0.5rem;
  border-radius: 4px;
}

.history {
  border-top: 1px solid rgba(148, 163, 184, 0.2);
  padding-top: 0.5rem;
}

.history ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.history li {
  display: flex;
  justify-content: space-between;
  padding: 0.4rem 0;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  font-size: 0.85rem;
}

.history li.ok {
  color: #4ade80;
}

.history li.warn {
  color: #f87171;
}
</style>
