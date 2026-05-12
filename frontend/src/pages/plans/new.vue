<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { usePlanCreate } from '@/composables/usePlans'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const create = usePlanCreate()
const errorMessage = ref('')

const TEMPLATE = `# New plan

Describe the plan here.
`

const content = ref(TEMPLATE)
const initial = TEMPLATE
const dirty = computed(() => content.value !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<string>('plan:new', content)
const recovered = draft.load()
if (typeof recovered === 'string') content.value = recovered

function slugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64)
}

function onSubmit() {
  errorMessage.value = ''
  const m = content.value.match(/^#\s+(.+?)\s*$/m)
  if (!m) {
    errorMessage.value = 'Add a `# Heading`; it becomes the slug.'
    return
  }
  const slug = slugify(m[1])
  if (!/^[a-z0-9_]+(?:-[a-z0-9_]+)*$/.test(slug)) {
    errorMessage.value = `Derived slug "${slug}" is invalid. Use lowercase letters, digits, "-" or "_" in heading.`
    return
  }

  create
    .mutateAsync({ slug, body: content.value })
    .then((p) => {
      draft.clear()
      content.value = initial
      router.push(`/plans/${encodeURIComponent(p.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  content.value = initial
  router.push('/plans')
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <PageHeader title="New plan" subtitle="Plain markdown. Slug is derived from the first `# Heading`.">
      <template #actions>
        <button type="button" class="ccg-btn-ghost" @click="onCancel">Cancel</button>
        <button
          type="button"
          class="ccg-btn-primary"
          :disabled="create.isPending.value"
          @click="onSubmit"
        >
          {{ create.isPending.value ? 'Creating…' : 'Create' }}
        </button>
      </template>
    </PageHeader>
    <section class="flex min-h-0 flex-1 flex-col p-6">
      <p
        v-if="errorMessage"
        class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ errorMessage }}
      </p>
      <MarkdownEditor v-model="content" fill class="min-h-0 flex-1" />
    </section>
  </div>
</template>
