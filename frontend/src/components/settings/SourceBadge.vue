<script setup lang="ts">
import { computed } from 'vue'
import type { SettingsScope } from '@/types/ipc'

const props = defineProps<{ scope: SettingsScope; overridden?: boolean }>()

const LABEL: Record<SettingsScope, string> = {
  managed: 'managed',
  local: 'local',
  project: 'project',
  user: 'user',
}

const cls = computed(() =>
  props.overridden
    ? 'bg-neutral-500/10 text-neutral-400 line-through'
    : 'bg-blue-500/10 text-blue-600 dark:text-blue-400',
)
</script>

<template>
  <span class="rounded px-1.5 py-0.5 text-[11px] font-medium" :class="cls">
    {{ LABEL[scope] }}
  </span>
</template>
