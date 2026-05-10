<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useProjectCreate,
  useProjectDelete,
  useProjectsList,
} from '@/composables/useProjects'

const { isPending, isError, error, data } = useProjectsList()
const create = useProjectCreate()
const remove = useProjectDelete()

const errorMessage = ref('')
const confirmingDelete = ref<string | null>(null)

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
          class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <li v-for="p in items" :key="p.name" class="flex items-stretch">
            <RouterLink
              :to="`/sessions/project/${encodeURIComponent(p.name)}`"
              class="flex-1 px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800"
            >
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.workingDir }}</span>
                <span class="text-xs text-neutral-400">
                  {{ p.sessionCount }} session{{ p.sessionCount === 1 ? '' : 's' }}
                </span>
              </div>
              <p class="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">
                Last active: {{ p.lastActive?.slice(0, 16) ?? 'never' }}
              </p>
            </RouterLink>
            <button
              type="button"
              class="px-3 text-xs text-red-600 hover:underline dark:text-red-400"
              @click="confirmingDelete = p.name"
            >
              Delete
            </button>
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
</template>
