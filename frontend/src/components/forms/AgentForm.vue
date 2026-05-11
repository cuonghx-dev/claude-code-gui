<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { agentSchema, flattenErrors } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import { useAsyncRequest } from '@/composables/useAsyncRequest'
import { agentsImproveInstructions } from '@/utils/ipc'
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

const improve = useAsyncRequest()
const improveError = ref('')
async function runImprove() {
  improveError.value = ''
  const body = state.body
  if (!body.trim()) {
    improveError.value = 'Body is empty.'
    return
  }
  try {
    const requestId = await agentsImproveInstructions({
      system:
        'You rewrite Claude Code agent instructions to be clearer, more specific, and more actionable. Preserve all technical content. Return the revised body only — no preamble.',
      prompt: body,
      model: state.model || null,
    } as Parameters<typeof agentsImproveInstructions>[0])
    // Replace body with streamed text as deltas arrive.
    state.body = ''
    improve.buffer.value = ''
    // Drain deltas into the editor by watching `improve.buffer`.
    const stop = watch(
      () => improve.buffer.value,
      (b) => {
        state.body = b
      },
    )
    try {
      await improve.start('claude:improve', String(requestId))
    } finally {
      stop()
    }
  } catch (e) {
    improveError.value = (e as { message?: string })?.message ?? String(e)
  }
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
          class="ccg-input"
        />
      </FormField>
      <FormField label="Directory" :error="errors.directory" hint="Subdir under agents/. Leave blank for top level.">
        <input
          v-model="state.directory"
          type="text"
          class="ccg-input"
        />
      </FormField>
      <FormField label="Name" :error="errors['frontmatter.name']">
        <input
          v-model="state.name"
          type="text"
          class="ccg-input"
        />
      </FormField>
      <FormField label="Description" :error="errors['frontmatter.description']">
        <textarea
          v-model="state.description"
          rows="2"
          class="ccg-input"
        />
      </FormField>
      <FormField label="Model" :error="errors['frontmatter.model']">
        <select
          v-model="state.model"
          class="ccg-input"
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
          class="ccg-input"
        />
      </FormField>
      <FormField label="Memory">
        <select
          v-model="state.memory"
          class="ccg-input"
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
          class="ccg-input"
        />
      </FormField>
      <FormField label="Tools" hint="Comma-separated">
        <input
          v-model="state.tools"
          type="text"
          class="ccg-input"
        />
      </FormField>
    </div>

    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-medium text-neutral-700 dark:text-neutral-300">Instructions</h3>
        <button
          type="button"
          class="ccg-btn-ghost text-xs"
          :disabled="improve.inFlight.value"
          @click="runImprove"
        >
          {{ improve.inFlight.value ? 'Improving…' : 'Improve with Claude' }}
        </button>
      </div>
      <p
        v-if="improveError"
        class="rounded-md border border-red-300 bg-red-50 p-2 text-xs text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ improveError }}
      </p>
      <MarkdownEditor v-model="state.body" min-height="420px" />
      <p
        v-if="improve.inFlight.value || improve.errorMessage.value"
        class="text-xs text-neutral-500 dark:text-neutral-400"
      >
        <span v-if="improve.inFlight.value">Streaming improvements from claude -p…</span>
        <span v-else-if="improve.errorMessage.value" class="text-red-600 dark:text-red-400">
          {{ improve.errorMessage.value }}
        </span>
      </p>
    </div>

    <div class="col-span-full flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
      <button
        type="button"
        class="ccg-btn-ghost"
        @click="emit('cancel')"
      >
        Cancel
      </button>
      <button
        type="submit"
        :disabled="submitting"
        class="ccg-btn-primary"
      >
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
