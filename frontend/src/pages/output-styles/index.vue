<script setup lang="ts">
import { ref } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import OutputStyleForm from '@/components/forms/OutputStyleForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useOutputStyleCreate,
  useOutputStyleDelete,
  useOutputStylesList,
} from '@/composables/useOutputStyles'
import type { OutputStyleInput, OutputStyleScope } from '@/types/ipc'

const { isPending, isError, error, data } = useOutputStylesList()
const create = useOutputStyleCreate()
const remove = useOutputStyleDelete()

const showForm = ref(false)
const errorMessage = ref('')
const confirmingDelete = ref<{ id: string; scope: OutputStyleScope } | null>(null)

async function onSubmit(input: OutputStyleInput) {
  errorMessage.value = ''
  try {
    await create.mutateAsync(input)
    showForm.value = false
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onConfirmDelete() {
  if (!confirmingDelete.value) return
  errorMessage.value = ''
  try {
    await remove.mutateAsync({ id: confirmingDelete.value.id, scope: confirmingDelete.value.scope })
    confirmingDelete.value = null
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="Output styles" subtitle="Global styles in ~/.claude/output-styles/">
    <template #actions>
      <button type="button" class="ccg-btn-primary" @click="showForm = !showForm">
        {{ showForm ? 'Close' : '+ New style' }}
      </button>
    </template>
  </PageHeader>

  <section v-if="showForm" class="border-b border-neutral-200 p-6 dark:border-neutral-800">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <OutputStyleForm
      draft-key="output-style:new"
      default-scope="global"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="showForm = false"
    />
  </section>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No output styles" />
        <ul v-else class="space-y-3">
          <li
            v-for="s in items"
            :key="`${s.scope}:${s.id}`"
            class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900"
          >
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-sm font-semibold">{{ s.frontmatter.name ?? s.id }}</span>
              <div class="flex items-center gap-2">
                <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide dark:bg-neutral-800">{{ s.scope }}</span>
                <button
                  type="button"
                  class="text-xs text-red-600 hover:underline dark:text-red-400"
                  @click="confirmingDelete = { id: s.id, scope: s.scope }"
                >
                  Delete
                </button>
              </div>
            </div>
            <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
              {{ s.frontmatter.description ?? '—' }}
            </p>
            <pre class="mt-3 max-h-48 overflow-auto rounded border border-neutral-100 bg-neutral-50 p-3 text-xs dark:border-neutral-800 dark:bg-neutral-950">{{ s.body }}</pre>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>

  <ConfirmDialog
    :open="!!confirmingDelete"
    title="Delete output style?"
    :message="confirmingDelete ? `Remove '${confirmingDelete.id}' (${confirmingDelete.scope}) permanently.` : ''"
    confirm-label="Delete"
    danger
    @update:open="(v: boolean) => { if (!v) confirmingDelete = null }"
    @confirm="onConfirmDelete"
  />
</template>
