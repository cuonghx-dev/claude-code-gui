<script setup lang="ts">
defineProps<{ label: string; hint?: string; error?: string; required?: boolean }>()
</script>

<template>
  <label class="flex flex-col gap-1">
    <span class="text-xs font-medium text-neutral-700 dark:text-neutral-300">
      {{ label }}<span v-if="required" class="ml-0.5 text-red-500">*</span>
    </span>
    <slot />
    <span
      v-if="error"
      class="text-xs text-red-600 dark:text-red-400"
    >
      {{ error }}
    </span>
    <span
      v-else-if="hint"
      class="text-xs text-neutral-500 dark:text-neutral-400"
    >
      {{ hint }}
    </span>
  </label>
</template>
