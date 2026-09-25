<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <header
    class="flex items-center gap-3 border-b px-7 pb-4 pt-5"
    style="border-color: var(--ccg-hairline-soft);"
  >
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <h2 class="ccg-page-title truncate">{{ title }}</h2>
      <p v-if="subtitle || $slots.subtitle" class="ccg-path truncate">
        <slot name="subtitle">{{ subtitle }}</slot>
      </p>
    </div>
    <div class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </header>
</template>
