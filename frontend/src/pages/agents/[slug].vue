<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useAgent,
  useAgentDelete,
  useAgentExport,
  useAgentUpdateRaw,
} from '@/composables/useAgents'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useAgent(slug)
const update = useAgentUpdateRaw()
const remove = useAgentDelete()
const exportMut = useAgentExport()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const content = ref('')
const initial = ref('')

const dirty = computed(() => content.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  async (a) => {
    if (!a) return
    try {
      const raw = await exportMut.mutateAsync(a.slug)
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
    router.push('/agents')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onExport() {
  errorMessage.value = ''
  try {
    const raw = await exportMut.mutateAsync(slug.value)
    const blob = new Blob([raw], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${slug.value}.md`
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
    :subtitle="data?.filePath ?? ''"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="onExport">Export</button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
      <button
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
    <template #default="{ data: agent }">
      <section v-if="agent" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <MarkdownEditor v-model="content" fill class="min-h-0 flex-1" />
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete agent?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
