<script setup lang="ts">
import RelationshipsPanel from '@/components/RelationshipsPanel.vue'
import EditorPage from '@/components/EditorPage.vue'
import { agentColor } from '@/utils/agentColor'
import { useRelatedCount } from '@/composables/useRelationships'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
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
const relatedCount = useRelatedCount('agent', slug)
const showRelated = ref(false)

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
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: agent }">
      <EditorPage
        v-if="agent"
        v-model="content"
        section="Agents"
        section-to="/agents"
        :name="slug"
        :color="agentColor(agent.frontmatter.color)"
        :dirty="dirty"
        :file-path="agent.filePath"
        :error="errorMessage"
      >
        <template #actions>
          <button
            type="button"
            class="ccg-btn-ghost ccg-btn-sm"
            :aria-pressed="showRelated"
            :disabled="relatedCount === 0"
            @click="showRelated = !showRelated"
          >
            Relationships ({{ relatedCount }})
          </button>
          <button type="button" class="ccg-btn-ghost ccg-btn-sm" @click="onExport">Export</button>
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
        <template #panel>
          <RelationshipsPanel v-if="showRelated && relatedCount > 0" :agent-slug="slug" />
        </template>
      </EditorPage>
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
