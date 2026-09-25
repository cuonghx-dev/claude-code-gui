<script setup lang="ts">
import type { SettingsScope } from '@/types/ipc'

withDefaults(
  defineProps<{
    scope: SettingsScope
    overridden?: boolean
    /** White fill for use on the tinted #F3F1EC panels; default sits on white cards. */
    onPanel?: boolean
    /** Green fill marking the scope a value comes from. */
    active?: boolean
  }>(),
  { overridden: false, onPanel: false, active: false },
)

const LABEL: Record<SettingsScope, string> = {
  managed: 'managed',
  local: 'local',
  project: 'project',
  user: 'user',
}
</script>

<template>
  <span
    class="inline-flex items-center whitespace-nowrap rounded-[3px] px-[5px] py-px font-sans text-[10.5px] leading-[1.4]"
    :class="
      overridden
        ? 'bg-[#F1EEE8] text-[#A29E94] line-through'
        : active
          ? 'bg-[#DDEBDF] text-[#3F7A4E]'
          : onPanel
            ? 'bg-white text-[#6B6860]'
            : 'bg-[#F1EEE8] text-[#6B6860]'
    "
  >
    {{ LABEL[scope] }}
  </span>
</template>
