<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import EditorPage from '@/components/EditorPage.vue'
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
  <EditorPage
    v-model="content"
    section="Plans"
    section-to="/plans"
    name="new"
    :dirty="dirty"
    file-path="~/.claude/plans/<heading>.md"
    :error="errorMessage"
    :frontmatter="false"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost ccg-btn-sm" @click="onCancel">Cancel</button>
      <button
        type="button"
        class="ccg-btn-primary ccg-btn-sm"
        :disabled="create.isPending.value"
        @click="onSubmit"
      >
        {{ create.isPending.value ? 'Creating…' : 'Create' }}
      </button>
    </template>
    <template #status>
      <span class="truncate font-sans">Slug is derived from the first <span class="font-mono"># Heading</span></span>
    </template>
  </EditorPage>
</template>
