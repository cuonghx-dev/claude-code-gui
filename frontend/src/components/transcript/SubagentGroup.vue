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
  <div class="rounded-lg border border-violet-300/60 dark:border-violet-900/60">
    <button
      type="button"
      class="flex w-full items-baseline gap-2 px-3 py-2 text-left hover:bg-violet-500/5"
      @click="expanded = !expanded"
    >
      <span class="shrink-0 text-xs font-semibold text-violet-600 dark:text-violet-400">
        Subagent{{ thread.agentName ? `: ${thread.agentName}` : '' }}
      </span>
      <span class="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400">
        <template v-if="thread.description">{{ thread.description }} · </template>
        {{ thread.messageCount }} messages · {{ tokens }} tok
        <template v-if="thread.costUsd != null"> · ${{ thread.costUsd.toFixed(4) }}</template>
      </span>
      <span class="text-xs text-neutral-400">{{ expanded ? '−' : '+' }}</span>
    </button>

    <div v-if="expanded" class="border-t border-violet-300/60 dark:border-violet-900/60">
      <div class="p-3">
        <ToolUseBlock :message="message" />
      </div>
      <p v-if="sub.isPending.value" class="px-3 pb-3 text-xs text-neutral-500">
        Loading subagent transcript…
      </p>
      <div v-else-if="sub.data.value?.items.length" class="max-h-[32rem] overflow-y-auto">
        <MessageRow
          v-for="m in sub.data.value.items"
          :key="m.id"
          :message="m"
          :project-name="projectName"
          :session-id="sessionId"
        />
      </div>
      <p v-else class="px-3 pb-3 text-xs text-neutral-500">
        This subagent's messages are inlined in the main transcript.
      </p>
    </div>
  </div>
</template>
