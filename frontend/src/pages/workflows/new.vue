<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import EditorPage from '@/components/EditorPage.vue'
import { useWorkflowCreate } from '@/composables/useWorkflows'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const create = useWorkflowCreate()
const errorMessage = ref('')

const TEMPLATE = `export const meta = {
  name: 'new-workflow',
  description: 'What this workflow orchestrates',
  phases: [{ title: 'Review' }],
}

phase('Review')

const found = await agent('List every .ts file under src/.', {
  schema: {
    type: 'object',
    required: ['files'],
    properties: { files: { type: 'array', items: { type: 'string' } } },
  },
})

const results = await pipeline(found.files, (file) =>
  agent(\`Review \${file}.\`, { label: file, phase: 'Review' }),
)

return results.filter(Boolean)
`

const content = ref(TEMPLATE)
const initial = TEMPLATE
const dirty = computed(() => content.value !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<string>('workflow:new', content)
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
  if (!/export\s+const\s+meta\s*=\s*\{/.test(content.value)) {
    errorMessage.value = 'Script must start with an `export const meta = { … }` block.'
    return
  }
  const m = content.value.match(/\bname\s*:\s*(?:'([^']+)'|"([^"]+)"|`([^`]+)`)/)
  const name = m?.[1] ?? m?.[2] ?? m?.[3]
  if (!name) {
    errorMessage.value = 'Add a string `name` to the meta block; it becomes the filename and `/command`.'
    return
  }
  const slug = slugify(name)
  if (!/^[a-z0-9_]+(?:-[a-z0-9_]+)*$/.test(slug)) {
    errorMessage.value = `Derived slug "${slug}" is invalid. Use lowercase letters, digits, "-" or "_" in the name.`
    return
  }

  create
    .mutateAsync({ slug, body: content.value })
    .then((w) => {
      draft.clear()
      content.value = initial
      router.push(`/workflows/${encodeURIComponent(w.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  content.value = initial
  router.push('/workflows')
}
</script>

<template>
  <EditorPage
    v-model="content"
    section="Workflows"
    section-to="/workflows"
    name="new"
    :dirty="dirty"
    file-path="~/.claude/workflows/<meta.name>.js"
    :error="errorMessage"
    language="javascript"
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
      <span class="truncate font-sans">Filename is derived from <span class="font-mono">meta.name</span></span>
    </template>
  </EditorPage>
</template>
