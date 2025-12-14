<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'success'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  disabled?: boolean
  hexagonal?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  loading: false,
  disabled: false,
  hexagonal: false,
})

const buttonClasses = computed(() => {
  const base = 'font-semibold transition-all duration-150 inline-flex items-center justify-center gap-2'

  const variants = {
    primary: 'bg-i3-cyan text-black hover:shadow-glow-cyan hover:scale-105 active:scale-95',
    secondary: 'border-2 border-i3-cyan text-i3-cyan bg-transparent hover:bg-i3-cyan hover:text-black',
    danger: 'bg-i3-magenta text-white hover:shadow-glow-magenta hover:scale-105 active:scale-95',
    success: 'bg-i3-orange text-black hover:shadow-glow-cyan hover:scale-105 active:scale-95',
  }

  const sizes = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-6 py-2 text-base',
    lg: 'px-8 py-3 text-lg',
  }

  const shape = props.hexagonal ? 'clip-path-hexagon' : 'rounded'

  const state = props.disabled || props.loading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'

  return `${base} ${variants[props.variant]} ${sizes[props.size]} ${shape} ${state}`
})
</script>

<template>
  <button
    :class="buttonClasses"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span v-if="loading" class="animate-spin">⟳</span>
    <slot />
  </button>
</template>

<style scoped>
.clip-path-hexagon {
  clip-path: polygon(
    30% 0%,
    70% 0%,
    100% 50%,
    70% 100%,
    30% 100%,
    0% 50%
  );
  padding: 12px 24px;
}

.clip-path-hexagon:hover {
  filter: brightness(1.1);
}
</style>
