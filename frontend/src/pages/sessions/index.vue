<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { Folder, Pencil, Trash2 } from 'lucide-vue-next'
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
  <div class="flex h-full flex-col">
    <PageHeader title="Sessions" subtitle="~/.claude/projects">
      <template #actions>
        <button type="button" class="ccg-btn-primary" @click="pickAndCreate">+ Add project</button>
      </template>
    </PageHeader>
    <p v-if="errorMessage" class="ccg-alert-error mx-7 mt-4 px-3 py-2 text-[12.5px]" role="alert">
      {{ errorMessage }}
    </p>
    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="data"
      skeleton="cards"
    >
      <template #default="{ data: items }">
        <EmptyState v-if="!items?.length" title="No projects yet. Run claude in any directory, or add one.">
          <button type="button" class="ccg-btn-primary" @click="pickAndCreate">+ Add project</button>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li
            v-for="p in items"
            :key="p.name"
            class="group ccg-card ccg-card-hover relative flex min-w-0 flex-col gap-2.5 px-4 py-3.5"
          >
            <RouterLink
              :to="`/sessions/project/${encodeURIComponent(p.name)}`"
              class="absolute inset-0 rounded-[9px]"
              :aria-label="`Open ${basename(p.workingDir)}`"
            />
            <div class="flex items-center gap-2">
              <Folder class="h-4 w-4 flex-none" :stroke-width="1.5" style="color: var(--ccg-muted);" />
              <span class="min-w-0 flex-1 truncate text-[13.5px] font-medium text-ink">
                {{ basename(p.workingDir) }}
              </span>
              <div
                class="relative z-10 -my-1 -mr-1.5 flex flex-none items-center gap-0.5 opacity-0 transition-opacity duration-[120ms] ease-out group-hover:opacity-100 focus-within:opacity-100"
              >
                <button
                  type="button"
                  class="ccg-icon-btn"
                  :aria-label="`Rename ${p.name}`"
                  title="Rename"
                  @click="renaming = { name: p.name, value: p.name }"
                >
                  <Pencil class="h-4 w-4" :stroke-width="1.5" />
                </button>
                <button
                  type="button"
                  class="ccg-icon-btn ccg-icon-btn-danger"
                  :aria-label="`Delete ${p.name}`"
                  title="Delete"
                  @click="confirmingDelete = p.name"
                >
                  <Trash2 class="h-4 w-4" :stroke-width="1.5" />
                </button>
              </div>
            </div>
            <p class="ccg-path truncate" :title="p.workingDir">{{ p.workingDir }}</p>
            <p class="text-[12px]" style="color: var(--ccg-subtle);">
              {{ p.sessionCount }} session{{ p.sessionCount === 1 ? '' : 's' }} · {{ relativeTime(p.lastActive) }}
            </p>
          </li>
        </ul>
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
        class="fixed inset-0 z-50 flex items-center justify-center p-4" style="background: rgba(31, 30, 27, .18);"
        @click.self="renaming = null"
      >
        <div class="ccg-card w-[420px] p-5 shadow-xl">
          <h3 class="text-[14px] font-semibold text-ink">Rename project entry</h3>
          <p class="mt-1 text-[12.5px]" style="color: var(--ccg-muted);">
            Renames the encoded directory under <code class="font-mono">~/.claude/projects/</code>. Does not touch the working dir.
          </p>
          <input
            v-model="renaming.value"
            class="ccg-input mt-3 font-mono text-[12.5px]"
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
  </div>
</template>

<style scoped>
.ccg-icon-btn {
  display: inline-flex;
  height: 26px;
  width: 26px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--ccg-muted);
  transition: background-color 120ms ease-out, color 120ms ease-out;
}
.ccg-icon-btn:hover {
  background: var(--ccg-hover);
  color: var(--ccg-ink);
}
.ccg-icon-btn-danger:hover {
  background: var(--ccg-error-bg);
  color: var(--ccg-error);
}
</style>
