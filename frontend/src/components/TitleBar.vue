<script setup lang="ts">
import { computed } from 'vue'
import { useConfig } from '@/composables/useSettings'

/**
 * 38px window chrome. The Tauri window uses `titleBarStyle: Overlay`, so the
 * native traffic lights sit over the left inset; the bar itself is a drag
 * region.
 */
const config = useConfig()
const claudeDir = computed(() => config.data.value?.claudeDirOverride || '~/.claude')
</script>

<template>
  <div
    data-tauri-drag-region
    class="relative flex h-[38px] flex-none select-none items-center border-b pl-[78px] pr-[14px]"
    style="background: var(--ccg-chrome); border-color: var(--ccg-chrome-border);"
  >
    <div
      data-tauri-drag-region
      class="pointer-events-none absolute inset-x-0 text-center text-[12px] font-medium"
      style="color: var(--ccg-muted);"
    >
      Claude Code GUI
    </div>
    <div data-tauri-drag-region class="flex-1" />
    <div class="relative font-mono text-[11px]" style="color: var(--ccg-subtle);">{{ claudeDir }}</div>
  </div>
</template>
