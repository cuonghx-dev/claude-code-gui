<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { Message, Thread } from '@/types/ipc'
import MessageRow from './MessageRow.vue'

const props = defineProps<{
  messages: Message[]
  threads: Thread[]
  hasNextPage: boolean
  isFetchingNextPage: boolean
  /** Hide sidechain messages, which the subagent cards already summarize. */
  hideSidechains: boolean
  projectName: string
  sessionId: string
}>()

const emit = defineEmits<{ loadMore: [] }>()

const visible = computed(() =>
  props.hideSidechains ? props.messages.filter((m) => !m.isSidechain) : props.messages,
)

// Threads hang off the Task tool_use they answer.
const threadByToolUse = computed(() => {
  const map = new Map<string, Thread>()
  for (const t of props.threads) {
    if (t.parentToolUseId) map.set(t.parentToolUseId, t)
  }
  return map
})

const parent = ref<HTMLElement | null>(null)

// Row heights range from one word to a capped tool result, so sizes are
// measured rather than assumed; the estimate only seeds the scrollbar.
const virtualizer = useVirtualizer(
  computed(() => ({
    count: visible.value.length,
    getScrollElement: () => parent.value,
    estimateSize: () => 96,
    overscan: 8,
    getItemKey: (i: number) => visible.value[i]?.id ?? i,
  })),
)

const rows = computed(() => virtualizer.value.getVirtualItems())

// Fetch the next page once the last rendered row is near the end.
watch(rows, (items) => {
  const last = items[items.length - 1]
  if (!last) return
  if (last.index >= visible.value.length - 10 && props.hasNextPage && !props.isFetchingNextPage) {
    emit('loadMore')
  }
})
</script>

<template>
  <div ref="parent" class="min-h-0 flex-1 overflow-y-auto">
    <div class="relative w-full" :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <div
        v-for="row in rows"
        :key="String(row.key)"
        :ref="(el) => virtualizer.measureElement(el as Element)"
        :data-index="row.index"
        class="absolute left-0 top-0 w-full"
        :style="{ transform: `translateY(${row.start}px)` }"
      >
        <MessageRow
          :message="visible[row.index]"
          :project-name="projectName"
          :session-id="sessionId"
          :thread="
            visible[row.index].toolUseId ? (threadByToolUse.get(visible[row.index].toolUseId!) ?? null) : null
          "
        />
      </div>
    </div>
    <p v-if="isFetchingNextPage" class="px-6 py-3 text-xs text-neutral-500">Loading more…</p>
  </div>
</template>
