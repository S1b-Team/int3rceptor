<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

interface Props {
  orientation?: 'horizontal' | 'vertical'
  defaultSize?: number
  minSize?: number
  maxSize?: number
  leftPanel?: boolean
  rightPanel?: boolean
  topPanel?: boolean
  bottomPanel?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  orientation: 'horizontal',
  defaultSize: 50,
  minSize: 20,
  maxSize: 80,
  leftPanel: true,
  rightPanel: true,
  topPanel: true,
  bottomPanel: true
})

const emit = defineEmits<{
  'update:size': [value: number]
}>()

const isDragging = ref(false)
const currentPosition = ref(props.defaultSize)
const dividerRef = ref<HTMLElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)

function startDrag(e: MouseEvent | TouchEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchmove', onDrag)
  document.addEventListener('touchend', stopDrag)
  e.preventDefault()
}

function onDrag(e: MouseEvent | TouchEvent) {
  if (!isDragging.value || !containerRef.value) return

  const container = containerRef.value
  const rect = container.getBoundingClientRect()

  let position: number

  if (props.orientation === 'horizontal') {
    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX
    position = ((clientX - rect.left) / rect.width) * 100
  } else {
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY
    position = ((clientY - rect.top) / rect.height) * 100
  }

  // Clamp to min/max
  position = Math.max(props.minSize, Math.min(props.maxSize, position))
  currentPosition.value = position
  emit('update:size', position)
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('touchend', stopDrag)
}

function resetSize() {
  currentPosition.value = props.defaultSize
  emit('update:size', props.defaultSize)
}

onMounted(() => {
  currentPosition.value = props.defaultSize
})

onUnmounted(() => {
  stopDrag()
})
</script>

<template>
  <div
    ref="containerRef"
    class="split-view h-full flex overflow-hidden"
    :class="{
      'flex-row': orientation === 'horizontal',
      'flex-col': orientation === 'vertical'
    }"
  >
    <!-- Left/Top Panel -->
    <div
      v-if="leftPanel || topPanel"
      class="overflow-hidden"
      :style="{
        flex: `${currentPosition} 0 0 0`,
        minWidth: orientation === 'horizontal' ? `${minSize}%` : '0',
        minHeight: orientation === 'vertical' ? `${minSize}%` : '0'
      }"
    >
      <slot name="first"></slot>
    </div>

    <!-- Draggable Divider -->
    <div
      ref="dividerRef"
      class="split-divider flex items-center justify-center hover:bg-i3-cyan/20 transition-colors cursor-col-resize"
      :class="{
        'w-1 cursor-col-resize': orientation === 'horizontal',
        'h-1 cursor-row-resize': orientation === 'vertical'
      }"
      @mousedown="startDrag"
      @touchstart="startDrag"
    >
      <div
        class="divider-handle transition-colors"
        :class="{
          'w-3 h-6 rounded-full bg-i3-border hover:bg-i3-cyan': orientation === 'horizontal',
          'h-3 w-6 rounded-full bg-i3-border hover:bg-i3-cyan': orientation === 'vertical'
        }"
      ></div>
    </div>

    <!-- Right/Bottom Panel -->
    <div
      v-if="rightPanel || bottomPanel"
      class="overflow-hidden"
      :style="{
        flex: `${100 - currentPosition} 0 0 0`,
        minWidth: orientation === 'horizontal' ? `${100 - maxSize}%` : '0',
        minHeight: orientation === 'vertical' ? `${100 - maxSize}%` : '0'
      }"
    >
      <slot name="second"></slot>
    </div>
  </div>
</template>

<style scoped>
.split-view {
  min-height: 0;
}

.split-divider {
  position: relative;
  z-index: 10;
  transition: background-color 0.15s;
}

.divider-handle {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  transition: background-color 0.15s, transform 0.15s;
}

.split-divider:hover .divider-handle {
  transform: translate(-50%, -50%) scale(1.2);
  box-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
}

.split-divider:active .divider-handle {
  background-color: var(--color-accent-cyan);
  transform: translate(-50%, -50%) scale(1.3);
}

/* Prevent text selection during drag */
.split-divider:active {
  user-select: none;
  cursor: grabbing;
}

.split-divider:active {
  cursor: grabbing;
}
</style>
