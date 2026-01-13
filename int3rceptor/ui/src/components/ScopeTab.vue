<template>
  <div class="scope-manager">
    <h2>Scope Management</h2>
    <p class="description">
      Control which URLs are captured. If includes is empty, everything is captured (except excludes).
    </p>

    <div class="scope-section">
      <h3>Include Patterns</h3>
      <p class="hint">Only capture URLs containing these patterns</p>
      <div class="pattern-list">
        <div v-for="(pattern, index) in config.includes" :key="`inc-${index}`" class="pattern-item">
          <input v-model="config.includes[index]" placeholder="example.com" />
          <button @click="removeInclude(index)" class="btn-remove">×</button>
        </div>
      </div>
      <button @click="addInclude" class="btn-add">+ Add Include</button>
    </div>

    <div class="scope-section">
      <h3>Exclude Patterns</h3>
      <p class="hint">Never capture URLs containing these patterns</p>
      <div class="pattern-list">
        <div v-for="(pattern, index) in config.excludes" :key="`exc-${index}`" class="pattern-item">
          <input v-model="config.excludes[index]" placeholder="google-analytics.com" />
          <button @click="removeExclude(index)" class="btn-remove">×</button>
        </div>
      </div>
      <button @click="addExclude" class="btn-add">+ Add Exclude</button>
    </div>

    <div class="actions">
      <button @click="save" class="btn-primary" :disabled="saving">
        {{ saving ? 'Saving...' : 'Save Scope' }}
      </button>
      <button @click="loadScope" class="btn-secondary">Reset</button>
    </div>

    <div v-if="message" :class="['message', messageType]">
      {{ message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useApi } from '@/composables/useApi';
import type { ScopeConfig } from '@/types';

const { getScope, setScope } = useApi();

const config = ref<ScopeConfig>({ includes: [], excludes: [] });
const saving = ref(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');

const loadScope = async () => {
  try {
    config.value = await getScope();
  } catch (error) {
    console.error('Failed to load scope:', error);
    message.value = 'Failed to load scope configuration';
    messageType.value = 'error';
  }
};

const save = async () => {
  saving.value = true;
  message.value = '';
  try {
    // Filter out empty strings
    const cleanConfig: ScopeConfig = {
      includes: config.value.includes.filter(p => p.trim() !== ''),
      excludes: config.value.excludes.filter(p => p.trim() !== ''),
    };
    await setScope(cleanConfig);
    config.value = cleanConfig;
    message.value = 'Scope saved successfully!';
    messageType.value = 'success';
    setTimeout(() => (message.value = ''), 3000);
  } catch (error) {
    console.error('Failed to save scope:', error);
    message.value = 'Failed to save scope';
    messageType.value = 'error';
  } finally {
    saving.value = false;
  }
};

const addInclude = () => {
  config.value.includes.push('');
};

const removeInclude = (index: number) => {
  config.value.includes.splice(index, 1);
};

const addExclude = () => {
  config.value.excludes.push('');
};

const removeExclude = (index: number) => {
  config.value.excludes.splice(index, 1);
};

onMounted(() => {
  loadScope();
});
</script>

<style scoped>
.scope-manager {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

h2 {
  color: #38bdf8;
  margin-bottom: 0.5rem;
}

.description {
  color: #94a3b8;
  margin-bottom: 2rem;
}

.scope-section {
  margin-bottom: 2rem;
  padding: 1.5rem;
  background: rgba(15, 23, 42, 0.6);
  border-radius: 8px;
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.scope-section h3 {
  color: #e2e8f0;
  margin-bottom: 0.25rem;
}

.hint {
  font-size: 0.85rem;
  color: #94a3b8;
  margin-bottom: 1rem;
}

.pattern-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.pattern-item {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.pattern-item input {
  flex: 1;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: 4px;
  padding: 0.5rem;
  color: #e2e8f0;
  font-family: monospace;
}

.pattern-item input:focus {
  outline: none;
  border-color: #38bdf8;
}

.btn-remove {
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #f87171;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1.5rem;
  line-height: 1;
  padding: 0;
}

.btn-remove:hover {
  background: rgba(239, 68, 68, 0.3);
}

.btn-add {
  background: transparent;
  border: 1px dashed rgba(148, 163, 184, 0.4);
  color: #94a3b8;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
}

.btn-add:hover {
  border-color: #38bdf8;
  color: #38bdf8;
}

.actions {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
}

.btn-primary {
  background: #38bdf8;
  border: none;
  color: #0f172a;
  padding: 0.6rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 600;
}

.btn-primary:hover:not(:disabled) {
  background: #0ea5e9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: transparent;
  border: 1px solid rgba(148, 163, 184, 0.4);
  color: #94a3b8;
  padding: 0.6rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
}

.btn-secondary:hover {
  border-color: #38bdf8;
  color: #38bdf8;
}

.message {
  margin-top: 1rem;
  padding: 0.75rem 1rem;
  border-radius: 4px;
  font-size: 0.9rem;
}

.message.success {
  background: rgba(34, 197, 94, 0.2);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: #4ade80;
}

.message.error {
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #f87171;
}
</style>
