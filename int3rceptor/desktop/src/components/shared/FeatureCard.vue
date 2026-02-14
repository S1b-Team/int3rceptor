<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  icon: string
  title: string
  description: string
  color?: 'cyan' | 'magenta' | 'orange' | 'purple'
  clickable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  color: 'cyan',
  clickable: false
})

const emit = defineEmits<{
  click: []
}>()

const borderColor = computed(() => {
  const colors = {
    cyan: 'border-i3-cyan',
    magenta: 'border-i3-magenta',
    orange: 'border-i3-orange',
    purple: 'border-i3-purple'
  }
  return colors[props.color]
})

const iconColor = computed(() => {
  const colors = {
    cyan: 'text-i3-cyan',
    magenta: 'text-i3-magenta',
    orange: 'text-i3-orange',
    purple: 'text-i3-purple'
  }
  return colors[props.color]
})

const hoverClass = computed(() => {
  if (!props.clickable) return ''
  return `hover:bg-${props.color === 'cyan' ? 'i3-cyan' : props.color === 'magenta' ? 'i3-magenta' : props.color === 'orange' ? 'i3-orange' : 'i3-purple'}/10 cursor-pointer`
})
</script>

<template>
  <div
    class="panel p-6 transition-all duration-300 group"
    :class="[borderColor, hoverClass]"
    @click="clickable ? emit('click') : undefined"
  >
    <div
      class="text-2xl mb-3 transition-transform duration-300 origin-left group-hover:scale-110"
      :class="iconColor"
    >
      {{ icon }}
    </div>
    <h3 class="text-lg font-bold text-i3-text mb-2 font-heading tracking-wide">
      {{ title }}
    </h3>
    <p class="text-sm text-i3-text-secondary leading-relaxed">
      {{ description }}
    </p>
  </div>
</template>
