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
  /** Rendered inside a subagent card: tighter gutters. */
  nested?: boolean
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

const speakerColor = computed(() =>
  props.message.role === 'assistant' ? 'var(--ccg-accent)' : 'var(--ccg-ink)',
)

const time = computed(() =>
  props.message.timestamp
    ? new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(new Date(props.message.timestamp))
    : '',
)

const showHeader = computed(() => props.message.kind === 'text' || props.message.isTurnHead)
</script>

<template>
  <!-- Rows are virtualized, so the 18px turn gap / 8px block gap live in padding. -->
  <article
    class="max-w-[860px]"
    :class="[nested ? 'px-3' : 'px-[22px]', showHeader ? 'pt-[18px]' : 'pt-2']"
  >
    <header v-if="showHeader" class="mb-1.5 text-[12px]" style="color: var(--ccg-subtle);">
      <span class="font-semibold" :style="{ color: speakerColor }">{{ roleLabel }}</span>
      <template v-if="time"> · {{ time }}</template>
      <template v-if="message.usage"> · <TurnMeta :message="message" /></template>
    </header>

    <div v-if="message.kind === 'text'" class="ccg-md prose prose-sm max-w-none" v-html="html" />
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
    <p v-else-if="message.kind === 'image'" class="text-[12.5px] italic" style="color: var(--ccg-subtle);">
      [image]
    </p>
    <p
      v-else
      class="rounded-md px-3 py-1.5 font-mono text-[11.5px]"
      style="background: var(--ccg-surface-strong); color: var(--ccg-muted);"
    >
      {{ message.content }}
    </p>
  </article>
</template>

<style scoped>
.ccg-md {
  font-size: 14px;
  line-height: 1.55;
  --tw-prose-body: var(--ccg-ink);
  --tw-prose-headings: var(--ccg-ink);
  --tw-prose-bold: var(--ccg-ink);
  --tw-prose-links: var(--ccg-accent);
  --tw-prose-code: var(--ccg-purple);
  --tw-prose-quotes: var(--ccg-body);
  --tw-prose-bullets: var(--ccg-muted-soft);
  --tw-prose-counters: var(--ccg-subtle);
  --tw-prose-hr: var(--ccg-hairline-soft);
  --tw-prose-pre-bg: var(--ccg-canvas-soft);
  --tw-prose-pre-code: var(--ccg-body);
}
.ccg-md :deep(p) {
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}
.ccg-md :deep(pre) {
  border: 1px solid var(--ccg-hairline);
  border-radius: 8px;
  font-size: 12.5px;
}
</style>
