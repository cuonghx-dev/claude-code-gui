<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    message?: string
    confirmLabel?: string
    cancelLabel?: string
    danger?: boolean
  }>(),
  { confirmLabel: 'Confirm', cancelLabel: 'Cancel', danger: false },
)
const emit = defineEmits<{
  'update:open': [boolean]
  confirm: []
}>()

function close() {
  emit('update:open', false)
}
function confirm() {
  emit('confirm')
  close()
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click.self="close"
    >
      <div
        class="w-[420px] rounded-lg border border-neutral-200 bg-white p-5 shadow-xl dark:border-neutral-800 dark:bg-neutral-900"
        role="dialog"
        aria-modal="true"
      >
        <h3 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">
          {{ props.title }}
        </h3>
        <p
          v-if="props.message"
          class="mt-2 text-sm text-neutral-600 dark:text-neutral-400"
        >
          {{ props.message }}
        </p>
        <div class="mt-5 flex justify-end gap-2">
          <button
            type="button"
            class="rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:hover:bg-neutral-700"
            @click="close"
          >
            {{ props.cancelLabel }}
          </button>
          <button
            type="button"
            class="rounded-md px-3 py-1.5 text-sm text-white"
            :class="
              props.danger
                ? 'bg-red-600 hover:bg-red-700'
                : 'bg-violet-600 hover:bg-violet-700'
            "
            @click="confirm"
          >
            {{ props.confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
