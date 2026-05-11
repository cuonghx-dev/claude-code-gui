<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { ArrowLeft, Plus, RefreshCcw, Settings } from 'lucide-vue-next'
import { useSessionsForProject } from '@/composables/useSessions'
import { useProject } from '@/composables/useProjects'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const project = useProject(projectName)
const sessions = useSessionsForProject(projectName)

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
        <li v-for="s in sessions.data.value ?? []" :key="s.sessionId">
          <RouterLink
            :to="`/sessions/project/${encodeURIComponent(projectName)}/session/${s.sessionId}`"
            class="block rounded-md border-l-2 px-3 py-2 transition hover:bg-neutral-50 dark:hover:bg-neutral-800/60"
            :class="s.sessionId === activeSessionId
              ? 'border-amber-500 bg-amber-500/5'
              : 'border-transparent bg-neutral-50/40 dark:bg-neutral-900'"
          >
            <div class="truncate text-sm font-medium">
              {{ truncate(s.preview ?? `Session ${s.sessionId.slice(0, 8)}…`, 60) }}
            </div>
            <div class="mt-1 flex items-center gap-2 text-[11px] text-neutral-500 dark:text-neutral-400">
              <span>{{ s.messageCount }} messages</span>
              <span>{{ relativeTime(s.lastMessageAt) }}</span>
            </div>
          </RouterLink>
        </li>
      </ul>
    </aside>
    <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <RouterView />
    </main>
  </div>
</template>
