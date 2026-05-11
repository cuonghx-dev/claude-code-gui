<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { ChevronRight, Folder, Pencil, Trash2 } from 'lucide-vue-next'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useProjectCreate,
  useProjectDelete,
  useProjectRename,
  useProjectsList,
} from '@/composables/useProjects'

const { isPending, isError, error, data } = useProjectsList()
const create = useProjectCreate()
const remove = useProjectDelete()
const rename = useProjectRename()

const errorMessage = ref('')
const confirmingDelete = ref<string | null>(null)
const renaming = ref<{ name: string; value: string } | null>(null)

async function pickAndCreate() {
  errorMessage.value = ''
  try {
    const picked = await openDialog({
      directory: true,
      multiple: false,
      title: 'Pick a project working directory',
    })
    if (typeof picked !== 'string') return
    await create.mutateAsync(picked)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onConfirmDelete() {
  if (!confirmingDelete.value) return
  errorMessage.value = ''
  try {
    await remove.mutateAsync(confirmingDelete.value)
    confirmingDelete.value = null
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onConfirmRename() {
  if (!renaming.value) return
  errorMessage.value = ''
  try {
    await rename.mutateAsync({
      name: renaming.value.name,
      newName: renaming.value.value.trim(),
    })
    renaming.value = null
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

function basename(p: string) {
  const trimmed = p.replace(/\/+$/, '')
  const i = trimmed.lastIndexOf('/')
  return i >= 0 ? trimmed.slice(i + 1) : trimmed
}

function relativeTime(iso: string | null | undefined) {
  if (!iso) return 'never'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return 'never'
  const diff = (Date.now() - d.getTime()) / 1000
  if (diff < 60) return 'Just now'
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`
  return d.toLocaleDateString()
}
</script>

<template>
  <PageHeader title="Sessions" subtitle="Pick a project to view past Claude Code sessions">
    <template #actions>
      <button type="button" class="ccg-btn-primary" @click="pickAndCreate">+ Add project</button>
    </template>
  </PageHeader>
  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState
          v-if="!items?.length"
          title="No projects"
          hint="Open `claude` in any directory or use Add project."
        />
        <ul
          v-else
          class="divide-y rounded-lg border bg-surface-card"
          style="border-color: var(--ccg-hairline);"
        >
          <li
            v-for="p in items"
            :key="p.name"
            class="group relative flex items-center gap-3 px-4 py-3 hover:bg-canvas-soft"
            style="border-color: var(--ccg-hairline);"
          >
            <RouterLink
              :to="`/sessions/project/${encodeURIComponent(p.name)}`"
              class="absolute inset-0"
              :aria-label="`Open ${basename(p.workingDir)}`"
            />
            <Folder class="h-5 w-5 shrink-0" style="color: var(--ccg-muted);" />
            <div class="min-w-0 flex-1">
              <h3 class="truncate text-sm font-medium text-ink">{{ basename(p.workingDir) }}</h3>
              <p class="truncate text-xs" style="color: var(--ccg-muted);">{{ p.workingDir }}</p>
            </div>
            <div class="hidden shrink-0 text-right text-xs sm:block" style="color: var(--ccg-muted);">
              <div>{{ p.sessionCount }} session{{ p.sessionCount === 1 ? '' : 's' }}</div>
              <div>{{ relativeTime(p.lastActive) }}</div>
            </div>
            <div class="relative z-10 flex shrink-0 items-center gap-1">
              <button
                type="button"
                class="rounded-md p-1.5 hover:bg-surface-strong"
                style="color: var(--ccg-muted);"
                :aria-label="`Rename ${p.name}`"
                @click="renaming = { name: p.name, value: p.name }"
              >
                <Pencil class="h-4 w-4" />
              </button>
              <button
                type="button"
                class="rounded-md p-1.5 hover:bg-surface-strong"
                style="color: var(--ccg-error);"
                :aria-label="`Delete ${p.name}`"
                @click="confirmingDelete = p.name"
              >
                <Trash2 class="h-4 w-4" />
              </button>
              <ChevronRight class="h-4 w-4" style="color: var(--ccg-muted-soft);" />
            </div>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>

  <ConfirmDialog
    :open="!!confirmingDelete"
    title="Delete project entry?"
    :message="confirmingDelete ? `Remove ~/.claude/projects/${confirmingDelete}/ permanently. The actual working directory is not touched.` : ''"
    confirm-label="Delete"
    danger
    @update:open="(v: boolean) => { if (!v) confirmingDelete = null }"
    @confirm="onConfirmDelete"
  />

  <Teleport to="body">
    <div
      v-if="renaming"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      @click.self="renaming = null"
    >
      <div class="w-[420px] rounded-lg border border-neutral-200 bg-white p-5 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <h3 class="text-sm font-semibold">Rename project entry</h3>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          Renames the encoded directory under <code>~/.claude/projects/</code>. Does not touch the working dir.
        </p>
        <input
          v-model="renaming.value"
          class="ccg-input mt-3 font-mono text-xs"
          @keyup.enter="onConfirmRename"
        />
        <div class="mt-4 flex justify-end gap-2">
          <button type="button" class="ccg-btn-ghost" @click="renaming = null">Cancel</button>
          <button
            type="button"
            class="ccg-btn-primary"
            :disabled="rename.isPending.value || !renaming.value.trim()"
            @click="onConfirmRename"
          >
            {{ rename.isPending.value ? 'Saving…' : 'Save' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
