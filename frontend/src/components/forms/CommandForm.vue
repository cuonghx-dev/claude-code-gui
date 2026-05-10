<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { commandSchema, flattenErrors } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { CommandInput } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<CommandInput>
  draftKey: string
  lockSlug?: boolean
  submitting?: boolean
  submitLabel?: string
}>()
const emit = defineEmits<{ submit: [CommandInput]; cancel: [] }>()

interface State {
  slug: string
  directory: string
  name: string
  description: string
  argumentHint: string
  allowedTools: string
  agent: string
  body: string
}

function fromInitial(): State {
  const fm = props.initial?.frontmatter
  return {
    slug: props.initial?.slug ?? '',
    directory: props.initial?.directory ?? '',
    name: fm?.name ?? '',
    description: fm?.description ?? '',
    argumentHint: fm?.argumentHint ?? '',
    allowedTools: (fm?.allowedTools ?? []).join(', '),
    agent: fm?.agent ?? '',
    body: props.initial?.body ?? '',
  }
}

const state = reactive(fromInitial())
const errors = ref<Record<string, string>>({})
const initial = JSON.stringify(fromInitial())
const dirty = computed(() => JSON.stringify(state) !== initial)
useUnsavedChanges(dirty)
const draft = useDraftRecovery<State>(props.draftKey, () => ({ ...state }))
const recovered = draft.load()
if (recovered) Object.assign(state, recovered)
watch(() => props.initial, () => Object.assign(state, fromInitial()))

function build(): CommandInput {
  return {
    slug: state.slug.trim(),
    directory: state.directory.trim(),
    frontmatter: {
      name: state.name.trim() || null,
      description: state.description.trim() || null,
      argumentHint: state.argumentHint.trim() || null,
      allowedTools: state.allowedTools.split(',').map((s) => s.trim()).filter(Boolean),
      agent: state.agent.trim() || null,
      extra: {},
    } as CommandInput['frontmatter'],
    body: state.body,
  } as CommandInput
}

function onSubmit() {
  const input = build()
  const r = commandSchema.safeParse(input)
  if (!r.success) {
    errors.value = flattenErrors(r.error)
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
        <input v-model="state.slug" :readonly="lockSlug" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Directory" :error="errors.directory">
        <input v-model="state.directory" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Name" :error="errors['frontmatter.name']">
        <input v-model="state.name" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Description">
        <textarea v-model="state.description" rows="2" class="ccg-input" />
      </FormField>
      <FormField label="Argument hint" hint="Shown after slash, e.g. [pr-number]">
        <input v-model="state.argumentHint" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Allowed tools" hint="Comma-separated">
        <input v-model="state.allowedTools" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Agent" hint="Optional agent slug to bind">
        <input v-model="state.agent" type="text" class="ccg-input" />
      </FormField>
    </div>
    <div class="flex flex-col gap-3">
      <FormField label="Body">
        <MarkdownEditor v-model="state.body" min-height="420px" />
      </FormField>
    </div>
    <div class="col-span-full flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
      <button type="button" class="ccg-btn-ghost" @click="emit('cancel')">Cancel</button>
      <button type="submit" :disabled="submitting" class="ccg-btn-primary">
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
