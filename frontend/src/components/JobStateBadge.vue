<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ state: string }>()

// The CLI adds states faster than this app can follow, so anything unknown
// falls back to a neutral chip instead of disappearing.
const TONE: Record<string, string> = {
  done: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
  running: 'bg-blue-500/10 text-blue-600 dark:text-blue-400',
  working: 'bg-blue-500/10 text-blue-600 dark:text-blue-400',
  blocked: 'bg-amber-500/10 text-amber-600 dark:text-amber-400',
  failed: 'bg-red-500/10 text-red-600 dark:text-red-400',
}

const cls = computed(
  () => TONE[props.state] ?? 'bg-neutral-500/10 text-neutral-600 dark:text-neutral-400',
)
</script>

<template>
  <span class="rounded px-1.5 py-0.5 text-[11px] font-medium" :class="cls">{{ state }}</span>
</template>
