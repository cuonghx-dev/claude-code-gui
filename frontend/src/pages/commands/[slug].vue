<script setup lang="ts">
import RelationshipGraph from '@/components/RelationshipGraph.vue'
import { useRelatedCount } from '@/composables/useRelationships'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useCommand,
  useCommandDelete,
  useCommandExport,
  useCommandUpdateRaw,
} from '@/composables/useCommands'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)
const relatedCount = useRelatedCount('command', slug)
const showRelated = ref(false)

const { isPending, isError, error, data } = useCommand(slug)
const update = useCommandUpdateRaw()
const remove = useCommandDelete()
const exportMut = useCommandExport()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const content = ref('')
const initial = ref('')

const dirty = computed(() => content.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  async (c) => {
    if (!c) return
    try {
      const raw = await exportMut.mutateAsync(c.slug)
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
    router.push('/commands')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="`/${slug}`" :subtitle="data?.frontmatter?.description ?? undefined">
    <template #actions>
      <button
        v-if="relatedCount > 0"
        type="button"
        class="ccg-btn-ghost"
        :aria-pressed="showRelated"
        @click="showRelated = !showRelated"
      >
        Relationships ({{ relatedCount }})
      </button>
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
    <template #default="{ data: cmd }">
      <section v-if="cmd" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <div class="flex min-h-0 flex-1 gap-4">
          <MarkdownEditor v-model="content" fill class="min-h-0 min-w-0 flex-1" />
          <aside
            v-if="showRelated && relatedCount > 0"
            class="w-80 shrink-0 overflow-y-auto rounded-lg border border-neutral-200 p-3 dark:border-neutral-800"
            aria-label="Relationships"
          >
            <RelationshipGraph :command-slug="slug" />
          </aside>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete command?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
