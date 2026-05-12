<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { usePlan, usePlanDelete, usePlanUpdate } from '@/composables/usePlans'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = usePlan(slug)
const update = usePlanUpdate()
const remove = usePlanDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const body = ref('')
const initial = ref('')

const dirty = computed(() => body.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  (p) => {
    if (!p) return
    body.value = p.body
    initial.value = p.body
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
    router.push('/plans')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="data?.title ?? slug" :subtitle="data?.filename">
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
    <template #default="{ data: plan }">
      <section v-if="plan" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <MarkdownEditor v-model="body" fill class="min-h-0 flex-1" />
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete plan?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
