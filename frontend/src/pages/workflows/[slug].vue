<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useWorkflow, useWorkflowDelete, useWorkflowUpdate } from '@/composables/useWorkflows'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useWorkflow(slug)
const update = useWorkflowUpdate()
const remove = useWorkflowDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const body = ref('')
const initial = ref('')

const dirty = computed(() => body.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  (w) => {
    if (!w) return
    body.value = w.body
    initial.value = w.body
  },
  { immediate: true },
)

async function onSave() {
  errorMessage.value = ''
  try {
    await update.mutateAsync({
      slug: slug.value,
      input: { slug: slug.value, body: body.value },
    })
    initial.value = body.value
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    initial.value = body.value
    router.push('/workflows')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="data ? `/${data.name}` : slug" :subtitle="data?.filename">
    <template #actions>
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
    <template #default="{ data: workflow }">
      <section v-if="workflow" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <p v-if="workflow.description" class="mb-3 text-sm text-neutral-600 dark:text-neutral-300">
          {{ workflow.description }}
        </p>
        <div v-if="workflow.phases.length" class="mb-3 flex flex-wrap gap-1.5">
          <span
            v-for="phase in workflow.phases"
            :key="phase"
            class="rounded bg-neutral-200 px-1.5 py-0.5 text-[10px] text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100"
          >
            {{ phase }}
          </span>
        </div>
        <MarkdownEditor v-model="body" language="javascript" fill class="min-h-0 flex-1" />
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete workflow?"
    :message="`This will permanently remove '${slug}.js' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
