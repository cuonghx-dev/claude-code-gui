<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message } from '@/types/ipc'

const props = defineProps<{ message: Message }>()

const expanded = ref(false)

const input = computed(() => {
  const v = props.message.toolInput
  if (v == null) return ''
  return typeof v === 'string' ? v : JSON.stringify(v, null, 2)
})

/** One-line gist for the collapsed row: the argument that identifies the call. */
const summary = computed(() => {
  const v = props.message.toolInput
  if (v == null || typeof v !== 'object') return ''
  const o = v as Record<string, unknown>
  const first = o.command ?? o.file_path ?? o.pattern ?? o.path ?? o.url ?? o.description
  return typeof first === 'string' ? first : ''
})

const lines = (s: unknown) => (typeof s === 'string' && s ? s.split('\n').length : 0)

// Line counts from the call's own arguments; a rough gist, not a real diff.
const diffStats = computed(() => {
  const v = props.message.toolInput
  if (v == null || typeof v !== 'object' || Array.isArray(v)) return null
  const o = v as Record<string, unknown>
  switch (props.message.toolName) {
    case 'Edit':
      return { add: lines(o.new_string), del: lines(o.old_string) }
    case 'MultiEdit': {
      const edits = Array.isArray(o.edits) ? (o.edits as Record<string, unknown>[]) : []
      return edits.reduce<{ add: number; del: number }>(
        (acc, e) => ({ add: acc.add + lines(e?.new_string), del: acc.del + lines(e?.old_string) }),
        { add: 0, del: 0 },
      )
    }
    case 'Write':
      return { add: lines(o.content), del: 0 }
    default:
      return null
  }
})
</script>

<template>
  <div class="overflow-hidden rounded-lg border bg-white" style="border-color: var(--ccg-hairline);">
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-2 text-left font-mono text-[12px] transition-colors duration-[120ms] ease-out hover:bg-canvas-soft"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span style="color: var(--ccg-subtle);">{{ expanded ? '▾' : '▸' }}</span>
      <span class="shrink-0 font-medium text-ink">{{ message.toolName ?? 'tool' }}</span>
      <span class="min-w-0 flex-1 truncate" style="color: var(--ccg-muted);">{{ summary }}</span>
      <template v-if="diffStats">
        <span v-if="diffStats.add" style="color: var(--ccg-success);">+{{ diffStats.add }}</span>
        <span v-if="diffStats.del" style="color: var(--ccg-error);">−{{ diffStats.del }}</span>
      </template>
    </button>
    <pre
      v-if="expanded && input"
      class="overflow-x-auto border-t px-3 py-2 font-mono text-[12px] leading-[1.6]"
      style="border-color: #F0EDE7; background: var(--ccg-canvas-soft); color: var(--ccg-body);"
    >{{ input }}</pre>
  </div>
</template>
