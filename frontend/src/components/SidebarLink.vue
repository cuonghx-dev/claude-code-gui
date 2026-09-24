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
    class="flex items-center gap-2 rounded-md px-3 py-1.5 text-sm text-neutral-700 transition-colors hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
    active-class="bg-neutral-100 font-medium text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
  >
    <component :is="item.icon" class="h-4 w-4" />
    <span class="flex-1">{{ item.label }}</span>
    <span
      v-if="item.count() !== undefined"
      class="rounded bg-neutral-200 px-1.5 py-0.5 text-[10px] tabular-nums text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100"
      :aria-label="`${item.count()} ${item.label.toLowerCase()}`"
    >
      {{ item.count() }}
    </span>
  </RouterLink>
</template>
