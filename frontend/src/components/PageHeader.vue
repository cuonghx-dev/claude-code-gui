<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <header class="flex items-center justify-between border-b border-neutral-200 bg-white px-6 py-4 dark:border-neutral-800 dark:bg-neutral-900">
    <div>
      <h2 class="text-lg font-semibold tracking-tight text-neutral-900 dark:text-neutral-100">
        {{ title }}
      </h2>
      <p v-if="subtitle" class="text-sm text-neutral-500 dark:text-neutral-400">
        {{ subtitle }}
      </p>
    </div>
    <div class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </header>
</template>
