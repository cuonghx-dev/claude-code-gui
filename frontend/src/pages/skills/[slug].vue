<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useSkill,
  useSkillDelete,
  useSkillExport,
  useSkillReadRaw,
  useSkillUpdateRaw,
} from '@/composables/useSkills'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useSkill(slug)
const update = useSkillUpdateRaw()
const remove = useSkillDelete()
const exportMut = useSkillExport()
const readRaw = useSkillReadRaw()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const content = ref('')
const initial = ref('')

const isLocal = computed(() => data.value?.source.kind === 'local')
const dirty = computed(() => content.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  async (s) => {
    if (!s) return
    try {
      const raw = await readRaw.mutateAsync(s.slug)
      content.value = raw
      initial.value = raw
    } catch (e) {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    }
  },
  { immediate: true },
)

async function onSave() {
  errorMessage.value = ''
  try {
    await update.mutateAsync({ slug: slug.value, content: content.value })
    initial.value = content.value
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    initial.value = content.value
    router.push('/skills')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onExport() {
  errorMessage.value = ''
  try {
    const bytes = await exportMut.mutateAsync(slug.value)
    const blob = new Blob([new Uint8Array(bytes)], { type: 'application/x-tar' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${slug.value}.tar`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader
    :title="data?.frontmatter?.name ?? slug"
    :subtitle="data?.source.kind === 'plugin' ? `From plugin ${data.source.id} (read-only)` : data?.directory"
  >
    <template #actions>
      <button v-if="isLocal" type="button" class="ccg-btn-ghost" @click="onExport">Export</button>
      <button
        v-if="isLocal"
        type="button"
        class="ccg-btn-danger"
        @click="confirmingDelete = true"
      >
        Delete
      </button>
      <button
        v-if="isLocal"
        type="button"
        class="ccg-btn-primary"
        :disabled="!dirty || update.isPending.value"
        @click="onSave"
      >
        {{ update.isPending.value ? 'Saving…' : 'Save' }}
      </button>
    </template>
  </PageHeader>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: skill }">
      <section v-if="skill" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <p
          v-if="!isLocal"
          class="mb-4 rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200"
        >
          Plugin-bundled skills are read-only. Edit the source plugin to change them.
        </p>
        <MarkdownEditor v-model="content" fill class="min-h-0 flex-1" />
      </section>
    </template>
  </QueryStateBoundary>

  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete skill?"
    :message="`This permanently removes the skill directory '${slug}/' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
