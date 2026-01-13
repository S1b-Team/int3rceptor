<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  variant?: 'cyan' | 'magenta' | 'orange' | 'purple' | 'gray'
  size?: 'sm' | 'md' | 'lg'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'cyan',
  size: 'md',
})

const badgeClasses = computed(() => {
  const base = 'inline-flex items-center justify-center font-medium rounded-full border'

  const variants = {
    cyan: 'bg-i3-cyan bg-opacity-20 text-i3-cyan border-i3-cyan',
    magenta: 'bg-i3-magenta bg-opacity-20 text-i3-magenta border-i3-magenta',
    orange: 'bg-i3-orange bg-opacity-20 text-i3-orange border-i3-orange',
    purple: 'bg-i3-purple bg-opacity-20 text-i3-purple border-i3-purple',
    gray: 'bg-i3-text-muted bg-opacity-20 text-i3-text-secondary border-i3-text-muted',
  }

  const sizes = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm',
    lg: 'px-3 py-1.5 text-base',
  }

  return `${base} ${variants[props.variant]} ${sizes[props.size]}`
})
</script>

<template>
  <span :class="badgeClasses">
    <slot />
  </span>
</template>

<style scoped>
span {
  white-space: nowrap;
  user-select: none;
}
</style>
