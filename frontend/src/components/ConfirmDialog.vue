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
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
      style="background: rgba(31, 30, 27, .18);"
      @click.self="close"
    >
      <div
        class="relative w-full max-w-[420px] rounded-[12px] bg-white p-5"
        style="box-shadow: 0 24px 60px rgba(40, 30, 20, .3), 0 0 0 1px rgba(0, 0, 0, .08);"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
      >
        <button
          type="button"
          class="absolute right-3 top-3 inline-flex h-7 w-7 items-center justify-center rounded-[6px] transition-colors hover:bg-[#F3F1EC]"
          style="color: var(--ccg-subtle);"
          aria-label="Close dialog"
          @click="close"
        >
          <X class="h-4 w-4" :stroke-width="1.5" aria-hidden="true" />
        </button>
        <h3 :id="titleId" class="pr-8 text-[15px] font-semibold text-ink">
          {{ props.title }}
        </h3>
        <p
          v-if="props.message"
          class="mt-2 text-[13px] leading-[1.5]"
          style="color: var(--ccg-body);"
        >
          {{ props.message }}
        </p>
        <div class="mt-5 flex justify-end gap-2">
          <button type="button" class="ccg-btn-ghost" @click="close">
            {{ props.cancelLabel }}
          </button>
          <button
            ref="confirmRef"
            type="button"
            :class="props.danger ? 'ccg-btn-primary ccg-btn-destructive' : 'ccg-btn-primary'"
            @click="confirm"
          >
            {{ props.confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ccg-btn-destructive {
  background: var(--ccg-error);
}
.ccg-btn-destructive:hover:not(:disabled) {
  background: #962f25;
}
</style>
