<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message } from '@/types/ipc'

const props = defineProps<{ message: Message }>()

// Tool results are the highest-leakage surface in the app: Bash and Read
// output can carry anything the agent touched. Collapsed by default; the Rust
// side has already capped what crossed the boundary.
const expanded = ref(false)

const text = computed(() => {
  const v = props.message.toolResult
  if (v == null) return ''
  if (typeof v === 'string') return v
  if (Array.isArray(v)) {
    return v
      .map((b) => (typeof b === 'string' ? b : ((b as { text?: string })?.text ?? JSON.stringify(b))))
      .join('\n')
  }
  return JSON.stringify(v, null, 2)
})

const lineCount = computed(() => (text.value ? text.value.split('\n').length : 0))
const firstLine = computed(() => text.value.split('\n', 1)[0] ?? '')

const sizeLabel = computed(() => {
  const b = props.message.toolResultBytes
  if (!b) return ''
  return b > 1024 ? `${Math.round(b / 1024)} KB` : `${b} B`
})
</script>

<template>
  <div
    class="rounded-lg border"
    :class="
      message.isError
        ? 'border-red-300 dark:border-red-900'
        : 'border-neutral-200 dark:border-neutral-800'
    "
  >
    <button
      type="button"
      class="flex w-full items-baseline gap-2 px-3 py-2 text-left hover:bg-neutral-50 dark:hover:bg-neutral-800/50"
      @click="expanded = !expanded"
    >
      <span
        class="text-xs font-semibold"
        :class="message.isError ? 'text-red-600 dark:text-red-400' : 'text-neutral-500 dark:text-neutral-400'"
      >
        {{ message.isError ? 'error' : 'result' }}
      </span>
      <span class="min-w-0 flex-1 truncate font-mono text-xs text-neutral-500 dark:text-neutral-400">
        {{ firstLine }}
      </span>
      <span class="shrink-0 text-[11px] text-neutral-400">
        {{ lineCount }} lines<template v-if="sizeLabel"> · {{ sizeLabel }}</template>
      </span>
      <span class="text-xs text-neutral-400">{{ expanded ? '−' : '+' }}</span>
    </button>
    <div v-if="expanded" class="border-t border-neutral-200 dark:border-neutral-800">
      <pre class="max-h-96 overflow-auto p-3 font-mono text-xs">{{ text }}</pre>
      <p
        v-if="message.toolResultTruncated"
        class="border-t border-neutral-200 px-3 py-2 text-[11px] text-neutral-500 dark:border-neutral-800 dark:text-neutral-400"
      >
        Truncated — the full result was {{ sizeLabel }} and is not loaded.
      </p>
    </div>
  </div>
</template>
