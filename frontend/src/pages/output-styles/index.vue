<script setup lang="ts">
import { computed, ref } from 'vue'
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
import { describe } from '@/utils/description'

const { isPending, isError, error, data } = useOutputStylesList()
const create = useOutputStyleCreate()
const remove = useOutputStyleDelete()

const showForm = ref(false)
const errorMessage = ref('')
const confirmingDelete = ref<{ id: string; scope: OutputStyleScope } | null>(null)
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (s) =>
      s.id.toLowerCase().includes(q) ||
      (s.frontmatter.name?.toLowerCase().includes(q) ?? false) ||
      (s.frontmatter.description?.toLowerCase().includes(q) ?? false),
  )
})

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
  <div class="flex h-full flex-col">
    <PageHeader title="Output styles" subtitle="~/.claude/output-styles/*.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <button
          type="button"
          :class="showForm ? 'ccg-btn-ghost' : 'ccg-btn-primary'"
          @click="showForm = !showForm"
        >
          {{ showForm ? 'Close' : '+ New' }}
        </button>
      </template>
    </PageHeader>

    <section v-if="showForm" class="border-b px-7 py-5" style="border-color: var(--ccg-hairline-soft);">
      <p v-if="errorMessage" class="ccg-alert-error mb-4 px-3 py-2 text-[12.5px]" role="alert">
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
    <p
      v-else-if="errorMessage"
      class="ccg-alert-error mx-7 mt-4 px-3 py-2 text-[12.5px]"
      role="alert"
    >
      {{ errorMessage }}
    </p>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="filtered"
      skeleton="cards"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          :title="search.trim() ? `No output styles match “${search.trim()}”.` : 'No output styles yet.'"
        >
          <button v-if="!search.trim() && !showForm" type="button" class="ccg-btn-primary" @click="showForm = true">
            + New
          </button>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li
            v-for="s in items"
            :key="`${s.scope}:${s.id}`"
            class="ccg-card flex min-w-0 flex-col gap-2.5 px-4 py-3.5"
          >
            <div class="flex items-center gap-2">
              <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">
                {{ s.frontmatter.name ?? s.id }}
              </span>
              <span
                class="ccg-badge"
                :style="s.scope === 'builtin' ? 'background: var(--ccg-warning-bg); color: var(--ccg-warning);' : undefined"
              >
                {{ s.scope === 'builtin' ? 'built-in' : s.scope }}
              </span>
            </div>
            <p
              class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
              style="color: var(--ccg-body); text-wrap: pretty;"
            >
              {{ describe(s.frontmatter.description, s.body) }}
            </p>
            <pre v-if="s.body" class="ccg-code-block max-h-32 text-[11px]">{{ s.body }}</pre>
            <div class="mt-auto flex items-center gap-1">
              <span v-if="s.frontmatter.keepCodingInstructions" class="ccg-chip">keep-coding-instructions</span>
              <span class="flex-1" />
              <button
                v-if="s.scope !== 'builtin'"
                type="button"
                class="ccg-btn-danger ccg-btn-sm"
                @click="confirmingDelete = { id: s.id, scope: s.scope }"
              >
                Delete
              </button>
            </div>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>

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
