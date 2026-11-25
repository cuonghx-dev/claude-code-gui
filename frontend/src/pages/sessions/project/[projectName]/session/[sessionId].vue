<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
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
import { useTerminalReplaysForSession } from '@/composables/useCliHistory'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const project = useProject(projectName)

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

const showReplays = ref(false)
watch(sessionId, () => {
  showReplays.value = false
})

const fmtReplay = (iso: string | null) =>
  iso
    ? new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'short' }).format(new Date(iso))
    : '—'

// The session is resumed in the user's own terminal; `claude --resume` only
// finds the session from the project's working directory, so cd there first.
const shellQuote = (s: string) => `'${s.replace(/'/g, `'\\''`)}'`
const resumeCommand = computed(() => {
  const resume = `claude --resume ${sessionId.value}`
  const dir = project.data.value?.workingDir
  return dir ? `cd ${shellQuote(dir)} && ${resume}` : resume
})

const copied = ref<'id' | 'command' | null>(null)
let copiedTimer: ReturnType<typeof setTimeout> | undefined
async function copy(kind: 'id' | 'command') {
  await navigator.clipboard.writeText(kind === 'id' ? sessionId.value : resumeCommand.value)
  copied.value = kind
  clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => (copied.value = null), 1500)
}
watch(sessionId, () => {
  copied.value = null
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
          class="flex overflow-hidden rounded-[7px] border"
          style="border-color: var(--ccg-ink);"
        >
          <button
            type="button"
            class="flex h-7 items-center bg-white px-2.5 font-mono text-[11.5px] text-ink transition-colors duration-[120ms] ease-out hover:bg-canvas-soft"
            :title="`Copy session id ${sessionId}`"
            @click="copy('id')"
          >
            {{ copied === 'id' ? 'Copied id' : `${sessionId.slice(0, 8)}…` }}
          </button>
          <button
            type="button"
            class="h-7 px-3 text-[12.5px] font-medium text-white transition-colors duration-[120ms] ease-out hover:opacity-90"
            style="background: var(--ccg-ink);"
            :title="resumeCommand"
            @click="copy('command')"
          >
            {{ copied === 'command' ? 'Copied' : 'Copy resume command' }}
          </button>
        </div>
      </div>
    </header>

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
  </section>
</template>
