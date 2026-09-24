<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft,
  FolderOpen,
  GitBranch,
  Pencil,
  Plus,
  RefreshCcw,
  Settings,
  Trash2,
} from 'lucide-vue-next'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useSessionDelete,
  useSessionRename,
  useSessionsForProject,
} from '@/composables/useSessions'
import { useProject, useProjectGitStatus } from '@/composables/useProjects'
import { revealInFinder } from '@/utils/ipc'
import type { SessionSummary } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const project = useProject(projectName)
const sessions = useSessionsForProject(projectName)
const git = useProjectGitStatus(projectName)
const renameSession = useSessionRename()
const deleteSession = useSessionDelete()

const errorMessage = ref('')
const editingId = ref('')
const editName = ref('')
const pendingDelete = ref<SessionSummary | null>(null)
const confirmOpen = computed({
  get: () => pendingDelete.value !== null,
  set: (v: boolean) => {
    if (!v) pendingDelete.value = null
  },
})

function errorText(e: unknown) {
  return (e as { message?: string })?.message ?? String(e)
}

function sessionLabel(s: SessionSummary) {
  return s.title ?? s.preview ?? `Session ${s.sessionId.slice(0, 8)}…`
}

async function startRename(s: SessionSummary) {
  editingId.value = s.sessionId
  editName.value = s.title ?? s.preview ?? ''
  await nextTick()
  document.getElementById(`rename-${s.sessionId}`)?.focus()
}

async function commitRename() {
  const sessionId = editingId.value
  const next = editName.value.trim()
  editingId.value = ''
  if (!sessionId || !next) return
  errorMessage.value = ''
  try {
    await renameSession.mutateAsync({ projectName: projectName.value, sessionId, newName: next })
  } catch (e) {
    errorMessage.value = errorText(e)
  }
}

async function confirmDelete() {
  const s = pendingDelete.value
  if (!s) return
  errorMessage.value = ''
  try {
    await deleteSession.mutateAsync({ projectName: projectName.value, sessionId: s.sessionId })
    if (s.sessionId === activeSessionId.value) {
      router.replace(`/sessions/project/${encodeURIComponent(projectName.value)}`)
    }
  } catch (e) {
    errorMessage.value = errorText(e)
  }
}

async function reveal(path: string | null | undefined) {
  if (!path) return
  errorMessage.value = ''
  try {
    await revealInFinder(path)
  } catch (e) {
    errorMessage.value = errorText(e)
  }
}

const activeSessionId = computed(
  () => (route.params as { sessionId?: string }).sessionId ?? '',
)

function basename(p: string | null | undefined) {
  if (!p) return ''
  const trimmed = p.replace(/\/+$/, '')
  const i = trimmed.lastIndexOf('/')
  return i >= 0 ? trimmed.slice(i + 1) : trimmed
}

function relativeTime(iso: string | null | undefined) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const s = (Date.now() - d.getTime()) / 1000
  if (s < 60) return 'Just now'
  if (s < 3600) return `${Math.floor(s / 60)}m ago`
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`
  if (s < 604800) return `${Math.floor(s / 86400)}d ago`
  return d.toLocaleDateString()
}

function truncate(s: string | null | undefined, max = 60) {
  if (!s) return ''
  return s.length > max ? s.slice(0, max) + '…' : s
}
</script>

<template>
  <div class="flex h-full min-h-0">
    <aside class="flex w-[360px] shrink-0 flex-col border-r border-neutral-200 dark:border-neutral-800">
      <header class="flex items-center gap-2 border-b border-neutral-200 px-3 py-3 dark:border-neutral-800">
        <RouterLink
          to="/sessions"
          class="rounded-md p-1.5 text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          aria-label="Back to projects"
        >
          <ArrowLeft class="h-4 w-4" />
        </RouterLink>
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-semibold">
            {{ basename(project.data.value?.workingDir) || projectName }}
          </div>
          <div class="truncate text-[11px] text-neutral-500 dark:text-neutral-400">
            {{ project.data.value?.workingDir }}
          </div>
        </div>
        <button
          type="button"
          class="rounded-md p-1.5 text-neutral-500 hover:bg-neutral-100 disabled:opacity-40 dark:hover:bg-neutral-800"
          :disabled="!project.data.value?.workingDir"
          aria-label="Reveal project folder"
          title="Reveal in file manager"
          @click="reveal(project.data.value?.workingDir)"
        >
          <FolderOpen class="h-4 w-4" />
        </button>
        <RouterLink
          :to="`/sessions/project/${encodeURIComponent(projectName)}/worktrees`"
          class="rounded-md p-1.5 text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          aria-label="Worktrees"
        >
          <GitBranch class="h-4 w-4" />
        </RouterLink>
        <RouterLink
          :to="`/sessions/project/${encodeURIComponent(projectName)}/settings`"
          class="rounded-md p-1.5 text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
          aria-label="Project settings"
        >
          <Settings class="h-4 w-4" />
        </RouterLink>
        <button
          type="button"
          class="rounded-md p-1.5 text-neutral-500 hover:bg-neutral-100 disabled:opacity-40 dark:hover:bg-neutral-800"
          :disabled="sessions.isFetching.value"
          aria-label="Refresh sessions"
          @click="() => sessions.refetch()"
        >
          <RefreshCcw class="h-4 w-4" :class="sessions.isFetching.value ? 'animate-spin' : ''" />
        </button>
      </header>
      <div
        v-if="git.data.value"
        class="flex items-center gap-2 border-b border-neutral-200 px-3 py-1.5 text-[11px] text-neutral-500 dark:border-neutral-800 dark:text-neutral-400"
        data-testid="git-status"
      >
        <GitBranch class="h-3 w-3 shrink-0" />
        <span class="truncate font-mono">{{ git.data.value.branch ?? 'detached' }}</span>
        <span v-if="git.data.value.ahead" title="Commits ahead of upstream">↑{{ git.data.value.ahead }}</span>
        <span v-if="git.data.value.behind" title="Commits behind upstream">↓{{ git.data.value.behind }}</span>
        <span
          class="ml-auto shrink-0"
          :class="git.data.value.clean ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'"
        >
          {{ git.data.value.clean ? 'clean' : `${git.data.value.files.length} changed` }}
        </span>
      </div>
      <p
        v-if="errorMessage"
        role="alert"
        class="mx-3 mt-3 rounded-md border border-red-300 bg-red-50 p-2 text-xs text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ errorMessage }}
      </p>
      <div class="p-3">
        <button
          type="button"
          class="flex w-full items-center justify-center gap-2 rounded-md bg-amber-500 px-3 py-2 text-sm font-medium text-amber-950 hover:bg-amber-400 disabled:cursor-not-allowed disabled:opacity-60"
          disabled
          title="Not yet wired"
        >
          <Plus class="h-4 w-4" />
          New Chat
        </button>
      </div>
      <ul class="flex-1 space-y-2 overflow-auto px-3 pb-3">
        <li v-if="!sessions.data.value?.length" class="text-xs text-neutral-500 dark:text-neutral-400">
          No sessions yet.
        </li>
        <li v-for="s in sessions.data.value ?? []" :key="s.sessionId" class="group relative">
          <div
            v-if="editingId === s.sessionId"
            class="rounded-md border-l-2 border-amber-500 bg-amber-500/5 px-3 py-2"
          >
            <input
              :id="`rename-${s.sessionId}`"
              v-model="editName"
              class="ccg-input w-full text-sm"
              aria-label="Session name"
              maxlength="200"
              @keydown.enter.prevent="commitRename"
              @keydown.esc.prevent="editingId = ''"
              @blur="commitRename"
            />
          </div>
          <RouterLink
            v-else
            :to="`/sessions/project/${encodeURIComponent(projectName)}/session/${s.sessionId}`"
            class="block rounded-md border-l-2 px-3 py-2 pr-16 transition hover:bg-neutral-50 dark:hover:bg-neutral-800/60"
            :class="s.sessionId === activeSessionId
              ? 'border-amber-500 bg-amber-500/5'
              : 'border-transparent bg-neutral-50/40 dark:bg-neutral-900'"
          >
            <div class="truncate text-sm font-medium" :title="sessionLabel(s)">
              {{ truncate(sessionLabel(s), 60) }}
            </div>
            <div class="mt-1 flex items-center gap-2 text-[11px] text-neutral-500 dark:text-neutral-400">
              <span>{{ s.messageCount }} messages</span>
              <span>{{ relativeTime(s.lastMessageAt) }}</span>
            </div>
          </RouterLink>
          <div
            v-if="editingId !== s.sessionId"
            class="absolute right-2 top-2 flex gap-0.5 opacity-0 transition focus-within:opacity-100 group-hover:opacity-100"
          >
            <button
              type="button"
              class="rounded p-1 text-neutral-500 hover:bg-neutral-200 dark:hover:bg-neutral-700"
              :aria-label="`Rename ${sessionLabel(s)}`"
              title="Rename"
              @click="startRename(s)"
            >
              <Pencil class="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              class="rounded p-1 text-neutral-500 hover:bg-red-100 hover:text-red-700 dark:hover:bg-red-950/60"
              :aria-label="`Delete ${sessionLabel(s)}`"
              title="Delete"
              @click="pendingDelete = s"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </button>
          </div>
        </li>
      </ul>
    </aside>
    <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <RouterView />
    </main>
    <ConfirmDialog
      v-model:open="confirmOpen"
      title="Delete session?"
      :message="pendingDelete
        ? `“${truncate(sessionLabel(pendingDelete), 80)}” and its subagent transcripts will be permanently deleted. This cannot be undone.`
        : ''"
      confirm-label="Delete"
      danger
      @confirm="confirmDelete"
    />
  </div>
</template>
