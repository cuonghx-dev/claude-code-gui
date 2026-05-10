<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useSessionsForProject } from '@/composables/useSessions'
import {
  useProject,
  useProjectDelete,
  useProjectGitStatus,
  useProjectRename,
} from '@/composables/useProjects'
import { useProjectWatcher } from '@/composables/useProjectWatcher'

const route = useRoute()
const router = useRouter()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const { data: project } = useProject(projectName)
const sessions = useSessionsForProject(projectName)
const git = useProjectGitStatus(projectName)
useProjectWatcher(() => project.value?.workingDir)
const rename = useProjectRename()
const remove = useProjectDelete()

const renaming = ref(false)
const renameTo = ref('')
const confirmingDelete = ref(false)
const errorMessage = ref('')

const gitErrorMessage = computed(
  () => (git.error.value as { message?: string } | undefined)?.message ?? 'git error',
)

async function onRename() {
  if (!renameTo.value) {
    renaming.value = false
    return
  }
  errorMessage.value = ''
  try {
    await rename.mutateAsync({ name: projectName.value, newName: renameTo.value })
    router.replace(`/sessions/project/${encodeURIComponent(renameTo.value)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  } finally {
    renaming.value = false
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(projectName.value)
    router.push('/sessions')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader
    :title="project?.workingDir ?? projectName"
    subtitle="Past Claude Code sessions in this project"
  >
    <template #actions>
      <RouterLink
        :to="`/sessions/project/${encodeURIComponent(projectName)}/settings`"
        class="ccg-btn-ghost"
      >
        Project settings
      </RouterLink>
      <button type="button" class="ccg-btn-ghost" @click="renaming = true">Rename</button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>

  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>

  <section v-if="renaming" class="mx-6 mt-4 rounded-md border border-neutral-200 bg-white p-3 dark:border-neutral-800 dark:bg-neutral-900">
    <div class="flex items-center gap-2">
      <input
        v-model="renameTo"
        :placeholder="projectName"
        class="ccg-input flex-1 font-mono text-xs"
      />
      <button type="button" class="ccg-btn-primary" @click="onRename">Save</button>
      <button type="button" class="ccg-btn-ghost" @click="renaming = false">Cancel</button>
    </div>
  </section>

  <section class="grid grid-cols-1 gap-6 p-6 lg:grid-cols-3">
    <div class="lg:col-span-2 space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Sessions</h3>
      <QueryStateBoundary
        :is-pending="sessions.isPending.value"
        :is-error="sessions.isError.value"
        :error="sessions.error.value"
        :data="sessions.data.value"
      >
        <template #default="{ data: items }">
          <EmptyState v-if="!items?.length" title="No sessions" />
          <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
            <li v-for="s in items" :key="s.sessionId">
              <RouterLink :to="`/sessions/project/${encodeURIComponent(projectName)}/session/${s.sessionId}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
                <div class="flex items-baseline justify-between gap-3">
                  <span class="font-mono text-xs">{{ s.sessionId.slice(0, 12) }}…</span>
                  <span class="text-xs text-neutral-400">{{ s.messageCount }} msg · {{ Math.round(Number(s.sizeBytes) / 1024) }} KB</span>
                </div>
                <p class="mt-0.5 line-clamp-1 text-xs text-neutral-500 dark:text-neutral-400">{{ s.preview ?? '—' }}</p>
                <p class="mt-1 text-[11px] text-neutral-400">{{ s.lastMessageAt?.slice(0, 16) ?? '—' }}</p>
              </RouterLink>
            </li>
          </ul>
        </template>
      </QueryStateBoundary>
    </div>

    <aside class="space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Git status</h3>
      <div class="rounded-lg border border-neutral-200 bg-white p-3 text-sm dark:border-neutral-800 dark:bg-neutral-900">
        <p v-if="git.isPending.value" class="text-neutral-500">Checking…</p>
        <p v-else-if="git.isError.value" class="text-red-600 dark:text-red-400">
          {{ gitErrorMessage }}
        </p>
        <template v-else-if="git.data.value">
          <p>
            <span class="text-neutral-500">branch:</span>
            <span class="ml-2 font-mono">{{ git.data.value.branch ?? '— (not a repo)' }}</span>
          </p>
          <p v-if="git.data.value.upstream">
            <span class="text-neutral-500">upstream:</span>
            <span class="ml-2 font-mono">{{ git.data.value.upstream }}</span>
            <span class="ml-2 text-neutral-400">+{{ git.data.value.ahead }} / -{{ git.data.value.behind }}</span>
          </p>
          <p v-if="git.data.value.clean" class="mt-2 text-emerald-700 dark:text-emerald-300">clean</p>
          <ul v-else class="mt-2 max-h-72 overflow-auto font-mono text-xs">
            <li v-for="f in git.data.value.files" :key="f.path" class="flex items-center gap-2">
              <span class="w-6 text-neutral-500">{{ f.status }}</span>
              <span class="break-all">{{ f.path }}</span>
            </li>
          </ul>
        </template>
      </div>
    </aside>
  </section>

  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete project entry?"
    :message="`This removes ~/.claude/projects/${projectName}/ permanently. The actual working directory is not touched.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
