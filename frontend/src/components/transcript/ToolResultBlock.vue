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

const footer = computed(() => `${lineCount.value} lines${sizeLabel.value ? ` · ${sizeLabel.value}` : ''}`)
</script>

<template>
  <div
    class="overflow-hidden rounded-lg border bg-white"
    :style="{ borderColor: message.isError ? 'var(--ccg-error-border)' : 'var(--ccg-hairline)' }"
  >
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-2 text-left font-mono text-[12px] transition-colors duration-[120ms] ease-out hover:bg-canvas-soft"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span style="color: var(--ccg-subtle);">{{ expanded ? '▾' : '▸' }}</span>
      <span
        class="shrink-0 font-medium"
        :style="{ color: message.isError ? 'var(--ccg-error)' : 'var(--ccg-ink)' }"
      >
        {{ message.isError ? 'error' : 'result' }}
      </span>
      <span class="min-w-0 flex-1 truncate" style="color: var(--ccg-muted);">{{ firstLine }}</span>
      <span v-if="!expanded" class="shrink-0" style="color: var(--ccg-muted-soft);">{{ footer }}</span>
    </button>
    <div
      v-if="expanded"
      class="border-t px-3 py-2 font-mono text-[12px] leading-[1.6]"
      style="border-color: #F0EDE7; background: var(--ccg-canvas-soft); color: var(--ccg-body);"
    >
      <pre class="max-h-96 overflow-auto whitespace-pre">{{ text }}</pre>
      <div style="color: var(--ccg-muted-soft);">
        … {{ footer }}<template v-if="message.toolResultTruncated"> · truncated, the rest is not loaded</template>
      </div>
    </div>
  </div>
</template>
