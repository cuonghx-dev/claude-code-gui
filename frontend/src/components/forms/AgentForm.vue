<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { agentSchema, flattenErrors } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { AgentInput } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<AgentInput>
  /** Stable identifier for draft recovery. e.g. `agent:new` or `agent:my-slug`. */
  draftKey: string
  /** When true, slug field becomes read-only (rename via update is allowed
   *  but here we keep it simple). */
  lockSlug?: boolean
  submitting?: boolean
  submitLabel?: string
}>()

const emit = defineEmits<{
  submit: [AgentInput]
  cancel: []
}>()

interface FormState {
  slug: string
  directory: string
  name: string
  description: string
  model: '' | 'opus' | 'sonnet' | 'haiku'
  color: string
  memory: '' | 'user' | 'project' | 'local' | 'none'
  skills: string
  tools: string
  body: string
}

function fromInitial(): FormState {
  const fm = props.initial?.frontmatter
  return {
    slug: props.initial?.slug ?? '',
    directory: props.initial?.directory ?? '',
    name: fm?.name ?? '',
    description: fm?.description ?? '',
    model: (fm?.model ?? '') as FormState['model'],
    color: fm?.color ?? '',
    memory: (fm?.memory ?? '') as FormState['memory'],
    skills: (fm?.skills ?? []).join(', '),
    tools: (fm?.tools ?? []).join(', '),
    body: props.initial?.body ?? '',
  }
}

const state = reactive(fromInitial())
const errors = ref<Record<string, string>>({})
const initial = JSON.stringify(fromInitial())

const dirty = computed(() => JSON.stringify(state) !== initial)
useUnsavedChanges(dirty)
const draft = useDraftRecovery<FormState>(props.draftKey, () => ({ ...state }))

// Hydrate draft if present and we're not editing an existing record.
const recovered = draft.load()
if (recovered) {
  Object.assign(state, recovered)
}

watch(() => props.initial, () => Object.assign(state, fromInitial()))

function buildInput(): AgentInput {
  return {
    slug: state.slug.trim(),
    directory: state.directory.trim(),
    frontmatter: {
      name: state.name.trim() || null,
      description: state.description.trim() || null,
      model: state.model || null,
      color: state.color.trim() || null,
      memory: state.memory || null,
      skills: state.skills.split(',').map((s) => s.trim()).filter(Boolean),
      tools: state.tools.split(',').map((s) => s.trim()).filter(Boolean),
      // `extra` is a server-private bag; the frontend always sends an empty
      // object and the backend preserves whatever was on disk.
      extra: {},
    } as AgentInput['frontmatter'],
    body: state.body,
  } as AgentInput
}

function onSubmit() {
  const input = buildInput()
  const result = agentSchema.safeParse(input)
  if (!result.success) {
    errors.value = flattenErrors(result.error)
    return
  }
  errors.value = {}
  emit('submit', input)
  draft.clear()
}
</script>

<template>
  <form class="grid grid-cols-1 gap-6 lg:grid-cols-2" @submit.prevent="onSubmit">
    <div class="space-y-4">
      <FormField label="Slug" required :error="errors.slug" hint="Filename without .md">
        <input
          v-model="state.slug"
          :readonly="lockSlug"
          type="text"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Directory" :error="errors.directory" hint="Subdir under agents/. Leave blank for top level.">
        <input
          v-model="state.directory"
          type="text"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Name" :error="errors['frontmatter.name']">
        <input
          v-model="state.name"
          type="text"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Description" :error="errors['frontmatter.description']">
        <textarea
          v-model="state.description"
          rows="2"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Model" :error="errors['frontmatter.model']">
        <select
          v-model="state.model"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        >
          <option value="">— inherit —</option>
          <option value="opus">Opus</option>
          <option value="sonnet">Sonnet</option>
          <option value="haiku">Haiku</option>
        </select>
      </FormField>
      <FormField label="Color" :error="errors['frontmatter.color']" hint="Hex, e.g. #7c3aed">
        <input
          v-model="state.color"
          type="text"
          placeholder="#7c3aed"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Memory">
        <select
          v-model="state.memory"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        >
          <option value="">— inherit —</option>
          <option value="user">user</option>
          <option value="project">project</option>
          <option value="local">local</option>
          <option value="none">none</option>
        </select>
      </FormField>
      <FormField label="Skills" hint="Comma-separated slugs">
        <input
          v-model="state.skills"
          type="text"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
      <FormField label="Tools" hint="Comma-separated">
        <input
          v-model="state.tools"
          type="text"
          class="rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-800"
        />
      </FormField>
    </div>

    <div class="flex flex-col gap-3">
      <FormField label="Instructions" hint="Markdown body of the agent system prompt">
        <MarkdownEditor v-model="state.body" min-height="420px" />
      </FormField>
    </div>

    <div class="col-span-full flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
      <button
        type="button"
        class="rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:hover:bg-neutral-700"
        @click="emit('cancel')"
      >
        Cancel
      </button>
      <button
        type="submit"
        :disabled="submitting"
        class="rounded-md bg-violet-600 px-3 py-1.5 text-sm text-white hover:bg-violet-700 disabled:opacity-50"
      >
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
