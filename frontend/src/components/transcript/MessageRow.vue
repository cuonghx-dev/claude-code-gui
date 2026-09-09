<script setup lang="ts">
import { computed } from 'vue'
import type { Message, Thread } from '@/types/ipc'
import ToolUseBlock from './ToolUseBlock.vue'
import ToolResultBlock from './ToolResultBlock.vue'
import ThinkingBlock from './ThinkingBlock.vue'
import SubagentGroup from './SubagentGroup.vue'
import TurnMeta from './TurnMeta.vue'
import { renderMarkdown } from '@/utils/markdown'

const props = defineProps<{
  message: Message
  /** The subagent conversation this message's Task call spawned, if any. */
  thread?: Thread | null
  projectName: string
  sessionId: string
}>()

const html = computed(() => (props.message.content ? renderMarkdown(props.message.content) : ''))

const roleLabel = computed(() => {
  switch (props.message.role) {
    case 'user':
      return 'You'
    case 'assistant':
      return 'Claude'
    default:
      return 'System'
  }
})

const time = computed(() =>
  props.message.timestamp
    ? new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(new Date(props.message.timestamp))
    : '',
)
</script>

<template>
  <article class="px-6 py-3">
    <header
      v-if="message.kind === 'text' || message.isTurnHead"
      class="mb-1 flex items-baseline gap-2"
    >
      <span
        class="text-xs font-semibold"
        :class="message.role === 'user' ? 'text-blue-600 dark:text-blue-400' : 'text-neutral-700 dark:text-neutral-200'"
      >
        {{ roleLabel }}
      </span>
      <span class="text-[11px] text-neutral-400">{{ time }}</span>
      <TurnMeta :message="message" class="ml-auto" />
    </header>

    <div
      v-if="message.kind === 'text'"
      class="prose prose-sm max-w-none dark:prose-invert"
      v-html="html"
    />
    <ThinkingBlock v-else-if="message.kind === 'thinking'" :message="message" />
    <template v-else-if="message.kind === 'tool-use'">
      <SubagentGroup
        v-if="thread"
        :thread="thread"
        :message="message"
        :project-name="projectName"
        :session-id="sessionId"
      />
      <ToolUseBlock v-else :message="message" />
    </template>
    <ToolResultBlock v-else-if="message.kind === 'tool-result'" :message="message" />
    <p v-else-if="message.kind === 'image'" class="text-xs italic text-neutral-500">
      [image]
    </p>
    <p
      v-else
      class="rounded bg-neutral-100 px-3 py-1.5 text-xs text-neutral-500 dark:bg-neutral-900 dark:text-neutral-400"
    >
      {{ message.content }}
    </p>
  </article>
</template>
