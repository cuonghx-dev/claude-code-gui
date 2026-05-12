<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import FormField from '@/components/forms/FormField.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { useAgentImport } from '@/composables/useAgents'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const importMut = useAgentImport()
const errorMessage = ref('')

const TEMPLATE = `---
name: my-agent
description: "Describe when this agent should be used."
tools: Read, Write, Edit, Bash, Glob, Grep
model: sonnet
---

You are ...
`

interface FormState {
  slug: string
  directory: string
  content: string
}

const state = reactive<FormState>({ slug: '', directory: '', content: TEMPLATE })
const initial = JSON.stringify(state)
const dirty = computed(() => JSON.stringify(state) !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<FormState>('agent:new', () => ({ ...state }))
const recovered = draft.load()
if (recovered) Object.assign(state, recovered)

const slugErr = ref('')
const contentErr = ref('')

/** Sniff `name:` from frontmatter to suggest a slug. Cheap regex; not a YAML parser. */
function sniffName(content: string): string | undefined {
  const fmMatch = content.match(/^---\r?\n([\s\S]*?)\r?\n---/)
  if (!fmMatch) return undefined
  const nameMatch = fmMatch[1].match(/^name:\s*["']?([^\r\n"']+)["']?\s*$/m)
  return nameMatch?.[1]?.trim()
}

function slugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64)
}

const suggestedSlug = computed(() => {
  const name = sniffName(state.content)
  return name ? slugify(name) : ''
})

function onSubmit() {
  slugErr.value = ''
  contentErr.value = ''
  errorMessage.value = ''

  const slug = (state.slug.trim() || suggestedSlug.value).trim()
  if (!slug) {
    slugErr.value = 'required (or add `name:` to frontmatter)'
    return
  }
  if (!/^[a-z0-9_]+(?:-[a-z0-9_]+)*$/.test(slug)) {
    slugErr.value = 'lowercase letters, digits, "-" or "_"'
    return
  }
  if (!state.content.trim()) {
    contentErr.value = 'required'
    return
  }
  if (!/^---\r?\n/.test(state.content)) {
    contentErr.value = 'must start with YAML frontmatter (---)'
    return
  }

  importMut
    .mutateAsync({ slug, directory: state.directory.trim(), content: state.content })
    .then((agent) => {
      draft.clear()
      Object.assign(state, JSON.parse(initial))
      router.push(`/agents/${encodeURIComponent(agent.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  Object.assign(state, JSON.parse(initial))
  router.push('/agents')
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <PageHeader title="New agent" subtitle="Paste a markdown agent definition. Backend parses the frontmatter." />
    <section class="flex min-h-0 flex-1 flex-col p-6">
      <p
        v-if="errorMessage"
        class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ errorMessage }}
      </p>
      <form class="flex h-full min-h-0 flex-col gap-4" @submit.prevent="onSubmit">
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <FormField
            label="Slug"
            :error="slugErr"
            :hint="suggestedSlug && !state.slug ? `auto: ${suggestedSlug}` : 'Filename without .md. Auto-derived from `name:` if blank.'"
          >
            <input
              v-model="state.slug"
              type="text"
              class="ccg-input"
              :placeholder="suggestedSlug || 'my-agent'"
            />
          </FormField>
          <FormField label="Directory" hint="Subdir under agents/. Leave blank for top level.">
            <input v-model="state.directory" type="text" class="ccg-input" />
          </FormField>
        </div>

        <div class="flex min-h-0 flex-1 flex-col gap-1">
          <span class="text-xs font-medium text-neutral-700 dark:text-neutral-300">Markdown</span>
          <MarkdownEditor v-model="state.content" fill class="min-h-[420px] flex-1" />
          <span
            v-if="contentErr"
            class="text-xs text-red-600 dark:text-red-400"
          >
            {{ contentErr }}
          </span>
          <span v-else class="text-xs text-neutral-500 dark:text-neutral-400">
            YAML frontmatter (---) then body.
          </span>
        </div>

        <div class="flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
          <button type="button" class="ccg-btn-ghost" @click="onCancel">Cancel</button>
          <button type="submit" :disabled="importMut.isPending.value" class="ccg-btn-primary">
            {{ importMut.isPending.value ? 'Creating…' : 'Create' }}
          </button>
        </div>
      </form>
    </section>
  </div>
</template>
