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

// Summed over what has been paged in; subagent-file threads are separate
// transcripts, while legacy sidechains are already among the messages.
const cost = computed(() => {
  let sum = 0
  for (const m of messages.value) sum += m.costUsd ?? 0
  for (const t of threads.data.value ?? []) {
    if (t.source === 'subagent-file') sum += t.costUsd ?? 0
  }
  return sum
})
const costLabel = computed(() => {
  if (!cost.value) return ''
  const partial = transcript.hasNextPage.value ? '≥' : ''
  return `${partial}$${cost.value.toFixed(2)}`
})

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
      class="flex shrink-0 items-center gap-2.5 border-b px-[22px] py-3.5"
      style="border-color: var(--ccg-hairline-soft);"
    >
      <div class="flex min-w-0 flex-1 flex-col gap-0.5">
        <h2 class="truncate text-[15px] font-semibold text-ink" :title="title ?? sessionId">
          <template v-if="title">{{ title }}</template>
          <template v-else>Session <span class="font-mono">{{ sessionId.slice(0, 8) }}…</span></template>
        </h2>
        <p class="truncate text-[12px]" style="color: var(--ccg-subtle);">
          {{ total.toLocaleString() }} messages
          <template v-if="threads.data.value?.length">
            · {{ threads.data.value.length }} subagent{{ threads.data.value.length === 1 ? '' : 's' }}
          </template>
          <template v-if="costLabel"> · {{ costLabel }}</template>
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2.5">
        <label
          v-if="!resuming"
          class="flex cursor-pointer select-none items-center gap-[7px] text-[12.5px]"
          style="color: var(--ccg-body);"
        >
          <button
            type="button"
            role="switch"
            class="ccg-switch"
            :aria-checked="showSidechains"
            @click="showSidechains = !showSidechains"
          />
          Subagent messages
        </label>
        <button
          v-if="!resuming"
          type="button"
          class="ccg-btn-ghost ccg-btn-sm"
          :aria-pressed="showCheckpoints"
          @click="showCheckpoints = !showCheckpoints"
        >
          Checkpoints
        </button>
        <template v-if="replays.data.value?.length">
          <RouterLink
            v-if="replays.data.value.length === 1"
            :to="`/terminals/${replays.data.value[0].id}`"
            class="ccg-btn-ghost ccg-btn-sm"
            title="Replay the terminal output recorded when this session ran here"
          >
            Terminal replay
          </RouterLink>
          <div v-else class="relative">
            <button
              type="button"
              class="ccg-btn-ghost ccg-btn-sm"
              :aria-pressed="showReplays"
              @click="showReplays = !showReplays"
            >
              Terminal replays ({{ replays.data.value.length }})
            </button>
            <ul
              v-if="showReplays"
              class="absolute right-0 z-20 mt-1 w-64 rounded-lg border bg-white py-1 text-[12.5px] shadow-lg"
              style="border-color: var(--ccg-hairline);"
            >
              <li v-for="r in replays.data.value" :key="r.id">
                <RouterLink
                  :to="`/terminals/${r.id}`"
                  class="flex items-center justify-between gap-2 px-3 py-1.5 text-ink transition-colors duration-[120ms] ease-out hover:bg-canvas-soft"
                  @click="showReplays = false"
                >
                  <span>{{ fmtReplay(r.endedAt) }}</span>
                  <span class="font-mono text-[11.5px]" style="color: var(--ccg-muted-soft);">
                    {{ r.lineCount.toLocaleString() }} lines<template v-if="r.exitCode"> · exit {{ r.exitCode }}</template>
                  </span>
                </RouterLink>
              </li>
            </ul>
          </div>
        </template>
        <div
          v-if="!resuming"
          class="flex overflow-hidden rounded-[7px] border"
          style="border-color: var(--ccg-ink);"
        >
          <div class="relative flex h-7 items-center bg-white">
            <select
              v-model="permissionMode"
              class="h-full cursor-pointer appearance-none bg-transparent pl-2.5 pr-6 font-mono text-[11.5px] text-ink outline-none"
              aria-label="Permission mode for the resumed session"
              title="Permission mode"
            >
              <option value="">CLI default</option>
              <option v-for="m in PERMISSION_MODES" :key="m" :value="m">{{ m }}</option>
            </select>
            <span
              class="pointer-events-none absolute right-2.5 font-mono text-[11.5px]"
              style="color: var(--ccg-muted);"
            >▾</span>
          </div>
          <button
            type="button"
            class="h-7 px-3 text-[12.5px] font-medium text-white transition-colors duration-[120ms] ease-out hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            style="background: var(--ccg-ink);"
            :disabled="!project.data.value"
            @click="resuming = true"
          >
            Resume in terminal
          </button>
        </div>
        <button v-else type="button" class="ccg-btn-ghost ccg-btn-sm" @click="resuming = false">
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
        class="w-72 shrink-0 overflow-y-auto border-l p-3"
        style="border-color: var(--ccg-hairline-soft); background: var(--ccg-canvas-soft);"
      />
    </div>

    <template v-else>
      <div v-if="transcript.isPending.value" class="flex max-w-[860px] flex-col gap-3 px-[22px] py-5">
        <div v-for="i in 4" :key="i" class="ccg-skeleton h-14 rounded-lg" />
      </div>
      <p v-else-if="transcript.isError.value" class="ccg-alert-error mx-[22px] mt-5 px-3 py-2 text-[12.5px]" role="alert">
        {{ (transcript.error.value as Error)?.message ?? 'Failed to read the transcript' }}
      </p>
      <EmptyState
        v-else-if="!messages.length"
        title="This session's transcript holds no renderable turns."
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
          class="flex w-[28rem] shrink-0 flex-col gap-3 overflow-y-auto border-l px-4 py-4"
          style="border-color: var(--ccg-hairline-soft); background: var(--ccg-canvas-soft);"
        >
          <h3 class="ccg-section-label">Checkpoints</h3>
          <CheckpointsPanel :project-name="projectName" :session-id="sessionId" />
        </aside>
      </div>
      <TeamPanel :session-id="sessionId" class="mx-[22px] mb-4 max-w-[816px] shrink-0" />
    </template>
  </section>
</template>
