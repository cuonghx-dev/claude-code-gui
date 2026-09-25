<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message, Thread } from '@/types/ipc'
import ToolUseBlock from './ToolUseBlock.vue'
import MessageRow from './MessageRow.vue'
import { useThreadMessages } from '@/composables/useSessions'

const props = defineProps<{
  thread: Thread
  message: Message
  projectName: string
  sessionId: string
}>()

// A subagent can run for dozens of turns, so it collapses to one card and its
// transcript is only fetched when opened.
const expanded = ref(false)

const sub = useThreadMessages(
  () => props.projectName,
  () => props.sessionId,
  () => props.thread.id,
  () => expanded.value && props.thread.source === 'subagent-file',
)

const tokens = computed(() => {
  const u = props.thread.usage
  return (Number(u.input) + Number(u.output)).toLocaleString()
})
</script>

<template>
  <div
    class="overflow-hidden rounded-lg border"
    style="background: var(--ccg-purple-bg); border-color: var(--ccg-purple-border);"
  >
    <button
      type="button"
      class="flex w-full items-center gap-2.5 px-3 py-2.5 text-left"
      :aria-expanded="expanded"
      :title="`${tokens} tok${thread.costUsd != null ? ` · $${thread.costUsd.toFixed(4)}` : ''}`"
      @click="expanded = !expanded"
    >
      <span
        class="shrink-0 rounded px-1.5 py-0.5 font-mono text-[10.5px] font-medium text-white"
        style="background: var(--ccg-purple);"
      >subagent</span>
      <span class="shrink-0 font-mono text-[12.5px] font-medium text-ink">{{ thread.agentName ?? 'agent' }}</span>
      <span class="min-w-0 flex-1 truncate text-[12.5px]" style="color: var(--ccg-muted);">
        {{ thread.description }}
      </span>
      <span class="shrink-0 text-[12px]" style="color: var(--ccg-purple);">
        {{ thread.messageCount }} messages {{ expanded ? '▾' : '▸' }}
      </span>
    </button>

    <div v-if="expanded" class="border-t bg-white" style="border-color: var(--ccg-purple-border);">
      <div class="p-3">
        <ToolUseBlock :message="message" />
      </div>
      <p v-if="sub.isPending.value" class="px-3 pb-3 text-[12px]" style="color: var(--ccg-subtle);">
        Loading subagent transcript…
      </p>
      <div v-else-if="sub.data.value?.items.length" class="max-h-[32rem] overflow-y-auto pb-3">
        <MessageRow
          v-for="m in sub.data.value.items"
          :key="m.id"
          :message="m"
          :project-name="projectName"
          :session-id="sessionId"
          nested
        />
      </div>
      <p v-else class="px-3 pb-3 text-[12px]" style="color: var(--ccg-subtle);">
        This subagent's messages are inlined in the main transcript.
      </p>
    </div>
  </div>
</template>
