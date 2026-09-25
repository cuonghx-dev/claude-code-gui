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
import { useProjectWatcher } from '@/composables/useProjectWatcher'
import { revealInFinder } from '@/utils/ipc'
import type { SessionSummary } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const project = useProject(projectName)
const sessions = useSessionsForProject(projectName)
const git = useProjectGitStatus(projectName)
// Live-refresh project-scoped settings, .mcp.json, CLAUDE.md and git status
// while this project is open.
useProjectWatcher(() => project.data.value?.workingDir ?? undefined)
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
    <aside
      class="flex w-[260px] shrink-0 flex-col border-r"
      style="background: #F9F8F4; border-color: var(--ccg-hairline-soft);"
    >
      <header class="flex items-start gap-1.5 px-4 pb-2.5 pt-4">
        <RouterLink
          to="/sessions"
          class="ccg-icon-btn -ml-1.5 mt-px"
          aria-label="Back to projects"
          title="All projects"
        >
          <ArrowLeft class="h-4 w-4" :stroke-width="1.5" />
        </RouterLink>
        <div class="flex min-w-0 flex-1 flex-col gap-[3px]">
          <div class="truncate text-[15px] font-semibold text-ink" :title="project.data.value?.workingDir ?? projectName">
            {{ basename(project.data.value?.workingDir) || projectName }}
          </div>
          <div class="truncate font-mono text-[11px]" style="color: var(--ccg-subtle);">
            {{ project.data.value?.workingDir }}
          </div>
        </div>
      </header>
      <div
        v-if="git.data.value"
        class="mx-4 mb-2.5 flex items-center gap-2.5 rounded-[7px] border bg-white px-2.5 py-[7px] font-mono text-[11.5px]"
        style="border-color: var(--ccg-hairline-soft); color: var(--ccg-muted);"
        data-testid="git-status"
      >
        <span class="min-w-0 truncate text-ink">⎇ {{ git.data.value.branch ?? 'detached' }}</span>
        <span
          v-if="git.data.value.upstream || git.data.value.ahead || git.data.value.behind"
          class="shrink-0"
          title="Commits ahead / behind upstream"
        >↑{{ git.data.value.ahead }} ↓{{ git.data.value.behind }}</span>
        <span
          class="ml-auto shrink-0"
          :style="{ color: git.data.value.clean ? 'var(--ccg-success)' : 'var(--ccg-accent)' }"
        >
          {{ git.data.value.clean ? 'clean' : `${git.data.value.files.length} changed` }}
        </span>
      </div>
      <div class="mx-4 mb-2 flex items-center gap-0.5">
        <button
          type="button"
          class="ccg-btn-ghost ccg-btn-sm mr-auto gap-1"
          disabled
          title="Not yet wired"
        >
          <Plus class="h-3.5 w-3.5" :stroke-width="1.5" />
          New chat
        </button>
        <button
          type="button"
          class="ccg-icon-btn"
          :disabled="!project.data.value?.workingDir"
          aria-label="Reveal project folder"
          title="Reveal in file manager"
          @click="reveal(project.data.value?.workingDir)"
        >
          <FolderOpen class="h-4 w-4" :stroke-width="1.5" />
        </button>
        <RouterLink
          :to="`/sessions/project/${encodeURIComponent(projectName)}/worktrees`"
          class="ccg-icon-btn"
          aria-label="Worktrees"
          title="Worktrees"
        >
          <GitBranch class="h-4 w-4" :stroke-width="1.5" />
        </RouterLink>
        <RouterLink
          :to="`/sessions/project/${encodeURIComponent(projectName)}/settings`"
          class="ccg-icon-btn"
          aria-label="Project settings"
          title="Project settings"
        >
          <Settings class="h-4 w-4" :stroke-width="1.5" />
        </RouterLink>
        <button
          type="button"
          class="ccg-icon-btn"
          :disabled="sessions.isFetching.value"
          aria-label="Refresh sessions"
          title="Refresh"
          @click="() => sessions.refetch()"
        >
          <RefreshCcw class="h-4 w-4" :stroke-width="1.5" :class="sessions.isFetching.value ? 'animate-spin' : ''" />
        </button>
      </div>
      <p v-if="errorMessage" role="alert" class="ccg-alert-error mx-4 mb-2 px-2.5 py-2 text-[12px]">
        {{ errorMessage }}
      </p>
      <ul class="flex flex-1 flex-col gap-0.5 overflow-auto px-2 pb-3">
        <li v-if="!sessions.data.value?.length" class="px-2.5 py-2 text-[13px]" style="color: var(--ccg-subtle);">
          No sessions yet.
        </li>
        <li v-for="s in sessions.data.value ?? []" :key="s.sessionId" class="group relative">
          <div v-if="editingId === s.sessionId" class="rounded-[7px] px-1 py-1" style="background: #E9E5DD;">
            <input
              :id="`rename-${s.sessionId}`"
              v-model="editName"
              class="ccg-input w-full text-[13px]"
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
            class="ccg-session-item flex flex-col gap-[3px] rounded-[7px] px-2.5 py-[9px]"
            :class="{ 'is-active': s.sessionId === activeSessionId }"
          >
            <div class="truncate text-[13px] font-medium text-ink" :title="sessionLabel(s)">
              {{ sessionLabel(s) }}
            </div>
            <div class="truncate text-[11.5px]" style="color: var(--ccg-subtle);">
              {{ s.messageCount }} messages<template v-if="relativeTime(s.lastMessageAt)"> · {{ relativeTime(s.lastMessageAt) }}</template>
            </div>
          </RouterLink>
          <div
            v-if="editingId !== s.sessionId"
            class="ccg-session-actions absolute right-1.5 top-1.5 flex gap-0.5 rounded-md opacity-0 group-hover:opacity-100 focus-within:opacity-100"
          >
            <button
              type="button"
              class="ccg-icon-btn"
              :aria-label="`Rename ${sessionLabel(s)}`"
              title="Rename"
              @click="startRename(s)"
            >
              <Pencil class="h-4 w-4" :stroke-width="1.5" />
            </button>
            <button
              type="button"
              class="ccg-icon-btn ccg-icon-btn-danger"
              :aria-label="`Delete ${sessionLabel(s)}`"
              title="Delete"
              @click="pendingDelete = s"
            >
              <Trash2 class="h-4 w-4" :stroke-width="1.5" />
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

<style scoped>
.ccg-session-item {
  transition: background-color 120ms ease-out;
}
.ccg-session-item:hover,
.group:hover .ccg-session-item {
  background: #EFECE5;
}
.ccg-session-item.is-active,
.group:hover .ccg-session-item.is-active {
  background: #E9E5DD;
}
/* Sits over the title's tail, so it takes the row's background to hide it. */
.ccg-session-actions {
  background: #EFECE5;
  transition: opacity 120ms ease-out;
}
.group:has(.is-active) .ccg-session-actions {
  background: #E9E5DD;
}
.ccg-icon-btn {
  display: inline-flex;
  height: 26px;
  width: 26px;
  flex: none;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--ccg-muted);
  transition: background-color 120ms ease-out, color 120ms ease-out;
}
.ccg-icon-btn:hover:not(:disabled) {
  background: var(--ccg-hover);
  color: var(--ccg-ink);
}
.ccg-icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.ccg-icon-btn-danger:hover:not(:disabled) {
  background: var(--ccg-error-bg);
  color: var(--ccg-error);
}
</style>
