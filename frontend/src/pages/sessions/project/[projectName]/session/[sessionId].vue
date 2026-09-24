<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import ChatTerminal from '@/components/ChatTerminal.vue'
import ContextPanel from '@/components/ContextPanel.vue'
import TeamPanel from '@/components/TeamPanel.vue'
import MessageList from '@/components/transcript/MessageList.vue'
import CheckpointsPanel from '@/components/CheckpointsPanel.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useProject } from '@/composables/useProjects'
import {
  useSessionMessages,
  useSessionsForProject,
  useSessionThreads,
} from '@/composables/useSessions'
import { useSettings } from '@/composables/useSettings'
import { useTerminalReplaysForSession } from '@/composables/useCliHistory'
import { asPermissionMode, PERMISSION_MODES } from '@/lib/permissionModes'
import type { PermissionMode, TerminalOpts } from '@/types/ipc'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const project = useProject(projectName)
const settings = useSettings()

const sessions = useSessionsForProject(projectName)
const title = computed(
  () => sessions.data.value?.find((s) => s.sessionId === sessionId.value)?.title ?? null,
)

const transcript = useSessionMessages(projectName, sessionId)
// PTY snapshots recorded when this session ran inside the app.
const replays = useTerminalReplaysForSession(sessionId)
const threads = useSessionThreads(projectName, sessionId)

const messages = computed(() => transcript.data.value?.pages.flatMap((p) => p.items) ?? [])
const total = computed(() => transcript.data.value?.pages[0]?.total ?? 0)

// Subagent turns are summarized by their card; showing them inline as well
// doubles the transcript. Off by default, one toggle away.
const showSidechains = ref(false)

// Checkpoints belong to the session, so they live beside the transcript rather
// than on a route of their own.
const showCheckpoints = ref(false)

const resuming = ref(false)
const showReplays = ref(false)
watch(sessionId, () => {
  showReplays.value = false
})

const fmtReplay = (iso: string | null) =>
  iso
    ? new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'short' }).format(new Date(iso))
    : '—'
// PTY session id, set once the terminal spawns; keys the live context events.
const ptyId = ref('')

// '' = let the CLI apply its own settings. Seeded from the user's default so
// the picker shows what will actually happen.
const permissionMode = ref<PermissionMode | ''>('')
watch(
  () => settings.data.value?.defaultPermissionMode,
  (v) => {
    if (!resuming.value) permissionMode.value = asPermissionMode(v) ?? ''
  },
  { immediate: true },
)
watch(resuming, (on) => {
  if (!on) ptyId.value = ''
})
watch(sessionId, () => {
  resuming.value = false
})

const terminalOpts = computed<TerminalOpts | null>(() => {
  if (!resuming.value || !project.data.value) return null
  return {
    agentSlug: null,
    cols: 100,
    rows: 32,
    workingDir: project.data.value.workingDir,
    model: null,
    permissionMode: permissionMode.value || null,
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
        <h2 class="truncate text-sm font-semibold" :title="title ?? sessionId">
          <template v-if="title">{{ title }}</template>
          <template v-else>Session <span class="font-mono">{{ sessionId.slice(0, 8) }}…</span></template>
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
        <button type="button" class="ccg-btn-ghost" @click="showCheckpoints = !showCheckpoints">
          {{ showCheckpoints ? 'Hide checkpoints' : 'Checkpoints' }}
        </button>
        <template v-if="replays.data.value?.length">
          <RouterLink
            v-if="replays.data.value.length === 1"
            :to="`/terminals/${replays.data.value[0].id}`"
            class="ccg-btn-ghost"
            title="Replay the terminal output recorded when this session ran here"
          >
            Terminal replay
          </RouterLink>
          <div v-else class="relative">
            <button type="button" class="ccg-btn-ghost" @click="showReplays = !showReplays">
              Terminal replays ({{ replays.data.value.length }})
            </button>
            <ul
              v-if="showReplays"
              class="absolute right-0 z-20 mt-1 w-64 rounded-md border border-neutral-200 bg-white py-1 text-xs shadow-lg dark:border-neutral-800 dark:bg-neutral-900"
            >
              <li v-for="r in replays.data.value" :key="r.id">
                <RouterLink
                  :to="`/terminals/${r.id}`"
                  class="flex items-center justify-between gap-2 px-3 py-1.5 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                  @click="showReplays = false"
                >
                  <span>{{ fmtReplay(r.endedAt) }}</span>
                  <span class="text-neutral-400">
                    {{ r.lineCount.toLocaleString() }} lines<template v-if="r.exitCode"> · exit {{ r.exitCode }}</template>
                  </span>
                </RouterLink>
              </li>
            </ul>
          </div>
        </template>
        <select
          v-if="!resuming"
          v-model="permissionMode"
          class="ccg-input py-1 text-xs"
          aria-label="Permission mode for the resumed session"
          title="Permission mode"
        >
          <option value="">CLI default</option>
          <option v-for="m in PERMISSION_MODES" :key="m" :value="m">{{ m }}</option>
        </select>
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

    <div v-if="resuming && terminalOpts" class="flex min-h-0 flex-1">
      <ChatTerminal
        :opts="terminalOpts"
        class="min-h-0 min-w-0 flex-1"
        @ready="(id) => (ptyId = id)"
      />
      <ContextPanel
        :session-id="ptyId || undefined"
        class="w-72 shrink-0 overflow-y-auto border-l border-neutral-200 p-3 dark:border-neutral-800"
      />
    </div>

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
      <div v-else class="flex min-h-0 flex-1">
        <MessageList
          class="min-w-0 flex-1"
          :messages="messages"
          :threads="threads.data.value ?? []"
          :has-next-page="!!transcript.hasNextPage.value"
          :is-fetching-next-page="transcript.isFetchingNextPage.value"
          :hide-sidechains="!showSidechains"
          :project-name="projectName"
          :session-id="sessionId"
          @load-more="transcript.fetchNextPage()"
        />
        <aside
          v-if="showCheckpoints"
          class="w-[28rem] shrink-0 overflow-y-auto border-l border-neutral-200 p-4 dark:border-neutral-800"
        >
          <h3 class="mb-3 text-sm font-semibold">Checkpoints</h3>
          <CheckpointsPanel :project-name="projectName" :session-id="sessionId" />
        </aside>
      </div>
      <TeamPanel :session-id="sessionId" class="mx-6 mb-4 shrink-0" />
    </template>
  </section>
</template>
