<template>
  <div class="status-badge" :class="[statusClass, sizeClass]">
    <span class="status-icon">{{ icon }}</span>
    <span class="status-text">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  status: 'healthy' | 'warning' | 'critical' | 'offline';
  label: string;
  icon?: string;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  icon: '●',
});

const statusClass = computed(() => `status-${props.status}`);
const sizeClass = computed(() => `size-${props.size}`);
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.status-icon {
  font-size: 10px;
  line-height: 1;
}

.status-text {
  font-family: 'Inter', 'Roboto', 'Segoe UI', sans-serif;
}

/* Status variants */
.status-healthy {
  background: rgba(0, 212, 255, 0.15);
  color: #00d4ff;
  border: 1px solid rgba(0, 212, 255, 0.3);
}

.status-warning {
  background: rgba(255, 184, 0, 0.15);
  color: #ffb800;
  border: 1px solid rgba(255, 184, 0, 0.3);
}

.status-critical {
  background: rgba(255, 0, 110, 0.15);
  color: #ff006e;
  border: 1px solid rgba(255, 0, 110, 0.3);
}

.status-offline {
  background: rgba(160, 160, 160, 0.15);
  color: #a0a0a0;
  border: 1px solid rgba(160, 160, 160, 0.3);
}

/* Size variants */
.size-sm {
  padding: 4px 8px;
  font-size: 10px;
}

.size-md {
  padding: 6px 12px;
  font-size: 12px;
}

.size-lg {
  padding: 8px 16px;
  font-size: 13px;
}

/* Hover effects */
.status-healthy:hover {
  background: rgba(0, 212, 255, 0.25);
  border-color: rgba(0, 212, 255, 0.5);
}

.status-warning:hover {
  background: rgba(255, 184, 0, 0.25);
  border-color: rgba(255, 184, 0, 0.5);
}

.status-critical:hover {
  background: rgba(255, 0, 110, 0.25);
  border-color: rgba(255, 0, 110, 0.5);
}

.status-offline:hover {
  background: rgba(160, 160, 160, 0.25);
  border-color: rgba(160, 160, 160, 0.5);
}
</style>
