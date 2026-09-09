<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message, Thread } from '@/types/ipc'
import ToolUseBlock from './ToolUseBlock.vue'

const props = defineProps<{ thread: Thread; message: Message }>()

// A subagent can run for dozens of turns. It collapses to one card in the main
// transcript so the parent conversation stays readable.
const expanded = ref(false)

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
      <span class="text-xs font-semibold text-violet-600 dark:text-violet-400">
        Subagent{{ thread.agentName ? `: ${thread.agentName}` : '' }}
      </span>
      <span class="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400">
        {{ thread.messageCount }} messages · {{ tokens }} tok
        <template v-if="thread.costUsd != null"> · ${{ thread.costUsd.toFixed(4) }}</template>
      </span>
      <span class="text-xs text-neutral-400">{{ expanded ? '−' : '+' }}</span>
    </button>
    <div v-if="expanded" class="border-t border-violet-300/60 p-3 dark:border-violet-900/60">
      <ToolUseBlock :message="message" />
      <p class="mt-2 text-[11px] text-neutral-500 dark:text-neutral-400">
        The subagent's own messages are inlined in this transcript below, marked as sidechain.
      </p>
    </div>
  </div>
</template>
