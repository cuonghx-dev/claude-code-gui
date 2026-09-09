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
</script>

<template>
  <div class="rounded-lg border border-neutral-200 dark:border-neutral-800">
    <button
      type="button"
      class="flex w-full items-baseline gap-2 px-3 py-2 text-left hover:bg-neutral-50 dark:hover:bg-neutral-800/50"
      @click="expanded = !expanded"
    >
      <span class="font-mono text-xs font-semibold text-violet-600 dark:text-violet-400">
        {{ message.toolName ?? 'tool' }}
      </span>
      <span class="min-w-0 flex-1 truncate font-mono text-xs text-neutral-500 dark:text-neutral-400">
        {{ summary }}
      </span>
      <span class="text-xs text-neutral-400">{{ expanded ? '−' : '+' }}</span>
    </button>
    <pre
      v-if="expanded && input"
      class="overflow-x-auto border-t border-neutral-200 p-3 font-mono text-xs dark:border-neutral-800"
    >{{ input }}</pre>
  </div>
</template>
