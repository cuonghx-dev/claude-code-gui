<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'
import { X } from 'lucide-vue-next'

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

const titleId = useId()
const confirmRef = ref<HTMLButtonElement | null>(null)
let lastFocused: HTMLElement | null = null

function close() {
  emit('update:open', false)
}
function confirm() {
  emit('confirm')
  close()
}

function onKey(e: KeyboardEvent) {
  if (!props.open) return
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
  }
}

watch(
  () => props.open,
  async (v) => {
    if (v) {
      lastFocused = (document.activeElement as HTMLElement | null) ?? null
      await nextTick()
      confirmRef.value?.focus()
    } else {
      lastFocused?.focus?.()
      lastFocused = null
    }
  },
)

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      @click.self="close"
    >
      <div
        class="relative w-full max-w-md rounded-lg border border-neutral-200 bg-white p-5 shadow-xl dark:border-neutral-800 dark:bg-neutral-900"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
      >
        <button
          type="button"
          class="absolute right-3 top-3 inline-flex h-8 w-8 items-center justify-center rounded-md text-neutral-500 hover:bg-neutral-100 hover:text-neutral-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-violet-500 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-100"
          aria-label="Close dialog"
          @click="close"
        >
          <X class="h-4 w-4" aria-hidden="true" />
        </button>
        <h3 :id="titleId" class="pr-8 text-base font-semibold text-neutral-900 dark:text-neutral-100">
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
            class="ccg-btn-ghost"
            @click="close"
          >
            {{ props.cancelLabel }}
          </button>
          <button
            ref="confirmRef"
            type="button"
            class="rounded-md px-3 py-1.5 text-sm text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-neutral-900"
            :class="
              props.danger
                ? 'bg-red-600 hover:bg-red-700 focus-visible:ring-red-500'
                : 'bg-violet-600 hover:bg-violet-700 focus-visible:ring-violet-500'
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
