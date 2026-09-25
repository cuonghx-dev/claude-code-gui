<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EditorPage from '@/components/EditorPage.vue'
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
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: workflow }">
      <EditorPage
        v-if="workflow"
        v-model="body"
        section="Workflows"
        section-to="/workflows"
        :name="`/${workflow.name}`"
        :dirty="dirty"
        :file-path="workflow.filePath"
        :error="errorMessage"
        language="javascript"
        :frontmatter="false"
      >
        <template #actions>
          <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirmingDelete = true">Delete</button>
          <button
            type="button"
            class="ccg-btn-primary ccg-btn-sm"
            :disabled="!dirty || update.isPending.value"
            @click="onSave"
          >
            {{ update.isPending.value ? 'Saving…' : 'Save' }}
          </button>
        </template>
        <template #banner>
          <div
            v-if="workflow.description || workflow.phases.length"
            class="flex flex-none flex-wrap items-center gap-1.5 border-b px-5 py-2.5"
            style="border-color: var(--ccg-hairline-soft);"
          >
            <p v-if="workflow.description" class="mr-2 text-[13px]" style="color: var(--ccg-body);">
              {{ workflow.description }}
            </p>
            <span v-for="phase in workflow.phases" :key="phase" class="ccg-chip">{{ phase }}</span>
          </div>
        </template>
      </EditorPage>
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
