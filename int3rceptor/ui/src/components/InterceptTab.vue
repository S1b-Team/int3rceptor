<template>
  <section class="intercept-panel">
    <header class="intercept-header">
      <div>
        <h2>Intercept Queue</h2>
        <p class="hint">
          When enabled, in-scope requests pause here until you forward, edit, or drop them.
        </p>
      </div>
      <label class="toggle">
        <input type="checkbox" :checked="enabled" @change="onToggle" />
        <span>{{ enabled ? "Intercept on" : "Intercept off" }}</span>
      </label>
    </header>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="!items.length" class="empty">
      {{ enabled ? "Waiting for held requests…" : "Enable intercept to hold traffic." }}
    </div>

    <div v-for="item in items" :key="item.id" class="held-card">
      <div class="held-meta">
        <strong>{{ item.method }}</strong>
        <span class="url">{{ item.url }}</span>
        <span class="id">#{{ item.id }}</span>
      </div>
      <textarea
        v-model="edits[item.id]"
        rows="6"
        spellcheck="false"
        :placeholder="item.body_text || ''"
      ></textarea>
      <div class="actions">
        <button @click="forward(item.id)">Forward</button>
        <button class="secondary" @click="applyEdit(item)">Edit &amp; Forward</button>
        <button class="danger" @click="drop(item.id)">Drop</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from "vue";
import { useApi } from "@/composables/useApi";

type HeldItem = {
  id: number;
  method: string;
  url: string;
  body_text?: string | null;
  headers: [string, string][] | { name: string; value: string }[];
};

const {
  listIntercept,
  setInterceptEnabled,
  forwardIntercept,
  dropIntercept,
  editIntercept,
} = useApi();

const enabled = ref(false);
const items = ref<HeldItem[]>([]);
const edits = reactive<Record<number, string>>({});
const error = ref<string | null>(null);
let timer: number | undefined;

async function refresh() {
  try {
    const data = await listIntercept();
    enabled.value = !!data.enabled;
    items.value = data.items || [];
    for (const item of items.value) {
      if (edits[item.id] === undefined) {
        edits[item.id] = item.body_text || "";
      }
    }
    error.value = null;
  } catch (e: any) {
    error.value = e?.message || "Failed to load intercept queue";
  }
}

async function onToggle(event: Event) {
  const checked = (event.target as HTMLInputElement).checked;
  try {
    const data = await setInterceptEnabled(checked);
    enabled.value = !!data.enabled;
    await refresh();
  } catch (e: any) {
    error.value = e?.message || "Failed to toggle intercept";
  }
}

async function forward(id: number) {
  await forwardIntercept(id);
  await refresh();
}

async function drop(id: number) {
  await dropIntercept(id);
  await refresh();
}

async function applyEdit(item: HeldItem) {
  await editIntercept(item.id, {
    method: item.method,
    url: item.url,
    body: edits[item.id] ?? item.body_text ?? "",
  });
  await refresh();
}

onMounted(() => {
  refresh();
  timer = window.setInterval(refresh, 1000);
});

onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<style scoped>
.intercept-panel {
  padding: 1rem 2rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.intercept-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}
.hint {
  color: #94a3b8;
  margin: 0.25rem 0 0;
  font-size: 0.9rem;
}
.toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.25);
  border-radius: 999px;
  padding: 0.4rem 0.8rem;
}
.held-card {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 0.75rem;
  padding: 0.85rem;
  background: rgba(30, 41, 59, 0.55);
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}
.held-meta {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}
.url {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #cbd5e1;
}
.id {
  color: #64748b;
  font-size: 0.85rem;
}
textarea {
  width: 100%;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  background: #0f172a;
  color: #e2e8f0;
  border: 1px solid rgba(148, 163, 184, 0.25);
  border-radius: 0.5rem;
  padding: 0.6rem;
}
.actions {
  display: flex;
  gap: 0.5rem;
}
button {
  background: #38bdf8;
  color: #0f172a;
  border: none;
  border-radius: 0.45rem;
  padding: 0.4rem 0.8rem;
  cursor: pointer;
  font-weight: 600;
}
button.secondary {
  background: #334155;
  color: #e2e8f0;
}
button.danger {
  background: #ef4444;
  color: white;
}
.empty,
.error {
  padding: 1rem;
  border-radius: 0.5rem;
}
.empty {
  color: #94a3b8;
  border: 1px dashed rgba(148, 163, 184, 0.3);
}
.error {
  background: rgba(239, 68, 68, 0.15);
  color: #fecaca;
}
</style>
