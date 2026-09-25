<script setup lang="ts" generic="T">
const props = defineProps<{
  isPending: boolean
  /** Skeleton shape while loading. */
  skeleton?: 'cards' | 'rows'
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
  <div v-if="isPending" role="status" aria-live="polite" aria-label="Loading">
    <slot name="loading">
      <div
        v-if="skeleton === 'cards'"
        class="grid gap-3 px-7 py-5"
        style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));"
      >
        <div v-for="i in 6" :key="i" class="ccg-skeleton h-[124px]" />
      </div>
      <div v-else class="flex flex-col gap-2 px-7 py-5">
        <div v-for="i in 5" :key="i" class="ccg-skeleton h-9" style="border-radius: 7px;" />
      </div>
    </slot>
  </div>
  <div
    v-else-if="isError"
    class="ccg-alert-error mx-7 my-5 px-4 py-3 text-[13px]"
    role="alert"
    aria-live="assertive"
  >
    Error: {{ errorMessage() }}
  </div>
  <slot v-else :data="data" />
</template>
