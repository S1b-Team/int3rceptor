<template>
  <div class="request-detail" v-if="entry">
    <h2>Request #{{ entry.request.id }}</h2>
    <p>
      <strong>{{ entry.request.method }}</strong>
      <span>{{ entry.request.url }}</span>
    </p>

    <section>
      <h3>Request Headers</h3>
      <CodeViewer :content="formatHeaders(entry.request.headers)" />
    </section>

    <section>
      <h3>Request Body</h3>
      <div v-if="isBinary(entry.request.body)" class="binary-placeholder">
        [Binary data: {{ entry.request.body.length }} bytes]
      </div>
      <CodeViewer 
        v-else 
        :content="formatBody(entry.request.body)" 
        :content-type="getContentType(entry.request.headers)" 
      />
    </section>

    <section v-if="entry.response">
      <h3>Response ({{ entry.response.status_code }})</h3>
      <p>Duration: {{ entry.response.duration_ms }} ms</p>
      <CodeViewer :content="formatHeaders(entry.response.headers)" />
      
      <div class="body-section">
        <div v-if="isBinary(entry.response.body)" class="binary-placeholder">
          [Binary data: {{ entry.response.body.length }} bytes]
        </div>
        <CodeViewer 
          v-else 
          :content="formatBody(entry.response.body)" 
          :content-type="getContentType(entry.response.headers)" 
        />
      </div>
    </section>
  </div>
  <div class="request-detail empty" v-else>
    Select a request to inspect details.
  </div>
</template>

<script setup lang="ts">
import { toRefs } from 'vue';
import type { CaptureEntry } from '@/types';
import CodeViewer from './CodeViewer.vue';

const props = defineProps<{
  entry: CaptureEntry | null;
}>();
const { entry } = toRefs(props);

const formatHeaders = (headers: [string, string][]) =>
  headers.map(([key, value]) => `${key}: ${value}`).join('\n');

const getContentType = (headers: [string, string][]) => {
  const ct = headers.find(([k]) => k.toLowerCase() === 'content-type');
  return ct ? ct[1] : undefined;
};

const formatBody = (body: number[]) => {
  if (!body.length) {
    return '';
  }
  try {
    return new TextDecoder().decode(new Uint8Array(body));
  } catch (error) {
    return '';
  }
};

const isBinary = (body: number[]) => {
  // Simple heuristic: check for null bytes or non-printable chars in first 100 bytes
  const slice = body.slice(0, 100);
  return slice.some(b => b === 0 || (b < 32 && b !== 9 && b !== 10 && b !== 13));
};
</script>

<style scoped>
.request-detail {
  height: 100%;
  padding: 1rem;
  overflow: auto;
}

section {
  margin-top: 1rem;
}

pre {
  background: rgba(15, 23, 42, 0.8);
  padding: 0.6rem;
  border-radius: 4px;
  font-size: 0.85rem;
  overflow-x: auto;
}

.binary-placeholder {
  background: rgba(15, 23, 42, 0.4);
  padding: 1rem;
  border: 1px dashed rgba(148, 163, 184, 0.3);
  border-radius: 4px;
  color: #94a3b8;
  font-family: monospace;
  text-align: center;
}

.body-section {
  margin-top: 0.5rem;
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #94a3b8;
}
</style>
