<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ state: string }>()

// The CLI adds states faster than this app can follow, so anything unknown
// falls back to a neutral chip instead of disappearing.
const TONE: Record<string, { color: string; background: string }> = {
  done: { color: 'var(--ccg-success)', background: 'var(--ccg-success-bg)' },
  running: { color: 'var(--ccg-blue)', background: '#E6EDF6' },
  working: { color: 'var(--ccg-blue)', background: '#E6EDF6' },
  blocked: { color: 'var(--ccg-warning)', background: 'var(--ccg-warning-bg)' },
  failed: { color: 'var(--ccg-error)', background: 'var(--ccg-error-bg)' },
}

const style = computed(
  () => TONE[props.state] ?? { color: 'var(--ccg-muted)', background: 'var(--ccg-surface-strong)' },
)
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 whitespace-nowrap rounded px-1.5 py-0.5 font-mono text-[10.5px] font-medium"
    :style="style"
  >
    <span class="h-[6px] w-[6px] flex-none rounded-full" style="background: currentColor;" />
    {{ state }}
  </span>
</template>
