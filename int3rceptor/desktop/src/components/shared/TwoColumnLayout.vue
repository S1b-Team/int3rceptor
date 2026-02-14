<script setup lang="ts">
interface Props {
  leftWidth?: string
  rightWidth?: string
  leftPanel?: boolean
  rightPanel?: boolean
  collapsible?: boolean
  defaultCollapsed?: 'left' | 'right' | 'none'
}

const props = withDefaults(defineProps<Props>(), {
  leftWidth: 'flex-1',
  rightWidth: 'w-1/3',
  leftPanel: true,
  rightPanel: true,
  collapsible: false,
  defaultCollapsed: 'none'
})

const emit = defineEmits<{
  'update:collapsed': [value: 'left' | 'right' | 'none']
}>()

const collapsed = ref<'left' | 'right' | 'none'>(props.defaultCollapsed)

function toggleCollapse(panel: 'left' | 'right') {
  if (!props.collapsible) return

  if (collapsed.value === panel) {
    collapsed.value = 'none'
  } else {
    collapsed.value = panel
  }

  emit('update:collapsed', collapsed.value)
}
</script>

<template>
  <div class="two-column-layout h-full flex gap-4 overflow-hidden">
    <!-- Left Panel -->
    <div
      v-if="leftPanel"
      class="panel overflow-hidden flex flex-col transition-all duration-300"
      :class="[
        leftWidth,
        {
          'w-0 min-w-0 p-0 opacity-0': collapsed === 'left' && collapsible,
          'border-i3-border': collapsed !== 'left'
        }
      ]"
    >
      <div v-if="collapsed !== 'left'" class="flex-1 overflow-auto">
        <slot name="left"></slot>
      </div>
    </div>

    <!-- Collapse Toggle (Left) -->
    <button
      v-if="collapsible && leftPanel && rightPanel"
      @click="toggleCollapse('left')"
      class="flex items-center justify-center p-2 rounded hover:bg-i3-bg-alt transition-colors"
      :class="{
        'text-i3-cyan': collapsed !== 'left',
        'text-i3-text-muted': collapsed === 'left'
      }"
      :aria-label="collapsed === 'left' ? 'Expand left panel' : 'Collapse left panel'"
    >
      <svg
        class="w-4 h-4 transition-transform duration-300"
        :class="{ 'rotate-180': collapsed === 'left' }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M15 19l-7-7 7-7"
        />
      </svg>
    </button>

    <!-- Right Panel -->
    <div
      v-if="rightPanel"
      class="panel overflow-hidden flex flex-col transition-all duration-300"
      :class="[
        rightWidth,
        {
          'w-0 min-w-0 p-0 opacity-0': collapsed === 'right' && collapsible,
          'border-i3-border': collapsed !== 'right'
        }
      ]"
    >
      <div v-if="collapsed !== 'right'" class="flex-1 overflow-auto">
        <slot name="right"></slot>
      </div>
    </div>
  </div>
</template>

<style scoped>
.two-column-layout {
  min-height: 0;
}

.two-column-layout > div {
  min-height: 100%;
}
</style>
