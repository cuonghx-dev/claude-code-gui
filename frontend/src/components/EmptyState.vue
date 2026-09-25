<script setup lang="ts">
defineProps<{
  title: string
  hint?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center gap-3 px-6 py-12 text-center">
    <p class="text-[13px]" style="color: var(--ccg-subtle);">{{ title }}</p>
    <p v-if="hint" class="text-[12px]" style="color: var(--ccg-muted-soft);">{{ hint }}</p>
    <slot />
  </div>
</template>
