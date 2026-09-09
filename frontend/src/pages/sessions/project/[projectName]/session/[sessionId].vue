<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import ChatTerminal from '@/components/ChatTerminal.vue'
import TeamPanel from '@/components/TeamPanel.vue'
import MessageList from '@/components/transcript/MessageList.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useProject } from '@/composables/useProjects'
import { useSessionMessages, useSessionThreads } from '@/composables/useSessions'
import { useSettings } from '@/composables/useSettings'
import type { TerminalOpts } from '@/types/ipc'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const project = useProject(projectName)
const settings = useSettings()

const transcript = useSessionMessages(projectName, sessionId)
const threads = useSessionThreads(projectName, sessionId)

const messages = computed(() => transcript.data.value?.pages.flatMap((p) => p.items) ?? [])
const total = computed(() => transcript.data.value?.pages[0]?.total ?? 0)

// Subagent turns are summarized by their card; showing them inline as well
// doubles the transcript. Off by default, one toggle away.
const showSidechains = ref(false)

const resuming = ref(false)

const terminalOpts = computed<TerminalOpts | null>(() => {
  if (!resuming.value || !project.data.value) return null
  return {
    agentSlug: null,
    cols: 100,
    rows: 32,
    workingDir: project.data.value.workingDir,
    model: null,
    permissionMode:
      (settings.data.value?.defaultPermissionMode as TerminalOpts['permissionMode']) ?? null,
    outputStyleId: null,
    resumeSessionId: sessionId.value,
    commandTemplate: null,
  } as TerminalOpts
})
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col">
    <header
      class="flex shrink-0 items-center gap-3 border-b border-neutral-200 px-6 py-3 dark:border-neutral-800"
    >
      <div class="min-w-0">
        <h2 class="truncate text-sm font-semibold">
          Session <span class="font-mono">{{ sessionId.slice(0, 8) }}…</span>
        </h2>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          {{ total.toLocaleString() }} messages
          <template v-if="threads.data.value?.length">
            · {{ threads.data.value.length }} subagent{{ threads.data.value.length === 1 ? '' : 's' }}
          </template>
        </p>
      </div>
      <div class="ml-auto flex shrink-0 items-center gap-2">
        <label class="flex items-center gap-1.5 text-xs text-neutral-500 dark:text-neutral-400">
          <input v-model="showSidechains" type="checkbox" />
          Subagent messages
        </label>
        <button
          v-if="!resuming"
          type="button"
          class="ccg-btn-ghost"
          :disabled="!project.data.value"
          @click="resuming = true"
        >
          Resume in terminal
        </button>
        <button v-else type="button" class="ccg-btn-ghost" @click="resuming = false">
          Close terminal
        </button>
      </div>
    </header>

    <ChatTerminal
      v-if="resuming && terminalOpts"
      :opts="terminalOpts"
      class="min-h-0 flex-1"
    />

    <template v-else>
      <p v-if="transcript.isPending.value" class="px-6 py-4 text-sm text-neutral-500">
        Loading transcript…
      </p>
      <p v-else-if="transcript.isError.value" class="px-6 py-4 text-sm text-red-600">
        {{ (transcript.error.value as Error)?.message ?? 'Failed to read the transcript' }}
      </p>
      <EmptyState
        v-else-if="!messages.length"
        title="No messages"
        hint="This session's transcript holds no renderable turns."
      />
      <MessageList
        v-else
        :messages="messages"
        :threads="threads.data.value ?? []"
        :has-next-page="!!transcript.hasNextPage.value"
        :is-fetching-next-page="transcript.isFetchingNextPage.value"
        :hide-sidechains="!showSidechains"
        @load-more="transcript.fetchNextPage()"
      />
      <TeamPanel :session-id="sessionId" class="mx-6 mb-4 shrink-0" />
    </template>
  </section>
</template>
