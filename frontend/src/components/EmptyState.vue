<script setup lang="ts">
defineProps<{
  title: string
  hint?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-neutral-300 px-6 py-12 text-center dark:border-neutral-700">
    <p class="text-sm font-medium text-neutral-700 dark:text-neutral-300">{{ title }}</p>
    <p v-if="hint" class="text-xs text-neutral-500 dark:text-neutral-400">{{ hint }}</p>
  </div>
</template>
