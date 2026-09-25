<script lang="ts">
import type { Component } from 'vue'

export interface NavItem {
  to: string
  label: string
  icon: Component
  count: () => number | undefined
}
</script>

<script setup lang="ts">
import { RouterLink } from 'vue-router'

defineProps<{ item: NavItem }>()
</script>

<template>
  <RouterLink
    :to="item.to"
    class="ccg-nav-item flex h-7 items-center gap-2 rounded-[6px] px-2.5 text-[13px] text-ink"
    active-class="ccg-nav-item-active"
  >
    <component :is="item.icon" class="h-4 w-4 flex-none" :stroke-width="1.5" style="color: var(--ccg-muted);" />
    <span class="flex-1 truncate">{{ item.label }}</span>
    <span
      v-if="item.count() !== undefined"
      class="font-mono text-[11px] tabular-nums"
      style="color: var(--ccg-muted-soft);"
      :aria-label="`${item.count()} ${item.label.toLowerCase()}`"
    >
      {{ item.count() }}
    </span>
  </RouterLink>
</template>

<style scoped>
.ccg-nav-item {
  transition: background-color 120ms ease-out;
}
.ccg-nav-item:hover {
  background: var(--ccg-hover);
}
.ccg-nav-item-active,
.ccg-nav-item-active:hover {
  background: var(--ccg-active);
  font-weight: 600;
}
</style>
