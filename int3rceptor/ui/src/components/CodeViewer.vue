<template>
  <div class="code-viewer">
    <div class="toolbar">
      <span class="lang-badge">{{ language }}</span>
      <button @click="copyToClipboard" class="copy-btn">
        {{ copied ? 'Copied!' : 'Copy' }}
      </button>
    </div>
    <pre><code ref="codeBlock" :class="`language-${language}`">{{ formattedCode }}</code></pre>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';

const props = defineProps<{
  content: string;
  contentType?: string;
}>();

const codeBlock = ref<HTMLElement | null>(null);
const copied = ref(false);

const language = computed(() => {
  if (!props.contentType) return 'plaintext';
  if (props.contentType.includes('json')) return 'json';
  if (props.contentType.includes('html') || props.contentType.includes('xml')) return 'xml';
  if (props.contentType.includes('css')) return 'css';
  if (props.contentType.includes('javascript') || props.contentType.includes('js')) return 'javascript';
  return 'plaintext';
});

const formattedCode = computed(() => {
  if (!props.content) return '';
  if (language.value === 'json') {
    try {
      return JSON.stringify(JSON.parse(props.content), null, 2);
    } catch {
      return props.content;
    }
  }
  // Basic HTML/XML formatter could go here, but for now return raw
  return props.content;
});

const highlight = () => {
  if (codeBlock.value) {
    // Reset highlight
    codeBlock.value.removeAttribute('data-highlighted');
    hljs.highlightElement(codeBlock.value);
  }
};

const copyToClipboard = async () => {
  await navigator.clipboard.writeText(formattedCode.value);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
};

watch(() => props.content, () => {
  nextTick(highlight);
});

onMounted(() => {
  highlight();
});
</script>

<style scoped>
.code-viewer {
  position: relative;
  background: #282c34;
  border-radius: 6px;
  overflow: hidden;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.25rem 0.5rem;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.lang-badge {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: #abb2bf;
  font-family: monospace;
}

.copy-btn {
  background: transparent;
  border: none;
  color: #abb2bf;
  font-size: 0.75rem;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

pre {
  margin: 0;
  padding: 1rem;
  overflow-x: auto;
  font-family: 'Fira Code', monospace;
  font-size: 0.9rem;
  line-height: 1.5;
}

code {
  background: transparent !important;
  padding: 0 !important;
}
</style>
