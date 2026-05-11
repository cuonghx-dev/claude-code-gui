<script setup lang="ts" generic="T">
const props = defineProps<{
  isPending: boolean
  isError: boolean
  error?: unknown
  data?: T
}>()

const errorMessage = () => {
  const e = props.error as { message?: string } | undefined
  return e?.message ?? String(props.error)
}
</script>

<template>
  <div v-if="isPending" class="p-6" role="status" aria-live="polite">
    <p class="text-sm text-neutral-500 dark:text-neutral-400">Loading…</p>
  </div>
  <div v-else-if="isError" class="m-6 rounded-md border border-red-300 bg-red-50 p-4 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200" role="alert" aria-live="assertive">
    Error: {{ errorMessage() }}
  </div>
  <slot v-else :data="data" />
</template>
