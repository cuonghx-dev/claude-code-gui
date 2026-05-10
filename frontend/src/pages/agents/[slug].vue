<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import AgentForm from '@/components/forms/AgentForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useAgent,
  useAgentDelete,
  useAgentExport,
  useAgentUpdate,
} from '@/composables/useAgents'
import type { AgentInput } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useAgent(slug)
const update = useAgentUpdate()
const remove = useAgentDelete()
const exportMut = useAgentExport()

const errorMessage = ref('')
const confirmingDelete = ref(false)

async function onSubmit(input: AgentInput) {
  errorMessage.value = ''
  try {
    const next = await update.mutateAsync({ slug: slug.value, input })
    if (next.slug !== slug.value) {
      router.replace(`/agents/${encodeURIComponent(next.slug)}`)
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
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
  <PageHeader :title="data?.frontmatter?.name ?? slug" :subtitle="data?.filePath">
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="onExport">Export</button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: agent }">
      <section v-if="agent" class="p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <AgentForm
          :draft-key="`agent:${agent.slug}`"
          :initial="{
            slug: agent.slug,
            directory: agent.directory,
            frontmatter: agent.frontmatter,
            body: agent.body,
          }"
          :submitting="update.isPending.value"
          submit-label="Save changes"
          @submit="onSubmit"
          @cancel="router.push('/agents')"
        />
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
