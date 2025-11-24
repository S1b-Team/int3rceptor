<template>
  <div class="request-list">
    <table>
      <thead>
        <tr>
          <th>ID</th>
          <th>Method</th>
          <th>URL</th>
          <th>Time</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="entry in items"
          :key="entry.request.id"
          :class="{ selected: entry.request.id === selectedId }"
          @click="() => emit('select', entry.request.id)"
        >
          <td>{{ entry.request.id }}</td>
          <td>{{ entry.request.method }}</td>
          <td class="url">{{ entry.request.url }}</td>
          <td>{{ formatTime(entry.request.timestamp_ms) }}</td>
        </tr>
        <tr v-if="!items.length">
          <td colspan="4" class="empty">No requests captured yet.</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { toRefs } from 'vue';
import type { CaptureEntry } from '@/types';

const props = defineProps<{
  items: CaptureEntry[];
  selectedId: number | null;
}>();
const { items, selectedId } = toRefs(props);
const emit = defineEmits<{ select: [number] }>();

const formatTime = (ts: number) => {
  const date = new Date(ts);
  return date.toLocaleTimeString();
};
</script>

<style scoped>
.request-list {
  height: 100%;
  overflow: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  padding: 0.4rem 0.6rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
}

tr.selected {
  background: rgba(56, 189, 248, 0.2);
}

tr:hover {
  background: rgba(56, 189, 248, 0.15);
}

.url {
  max-width: 420px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty {
  text-align: center;
  padding: 2rem 0;
}
</style>
