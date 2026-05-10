<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { flattenErrors, outputStyleSchema } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { OutputStyleInput, OutputStyleScope } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<OutputStyleInput>
  draftKey: string
  /** Pre-fill scope; if omitted, user picks. */
  defaultScope?: OutputStyleScope
  defaultWorkingDir?: string
  lockId?: boolean
  submitting?: boolean
  submitLabel?: string
}>()
const emit = defineEmits<{ submit: [OutputStyleInput]; cancel: [] }>()

interface State {
  id: string
  scope: OutputStyleScope
  workingDir: string
  name: string
  description: string
  keepCodingInstructions: boolean
  body: string
}

function fromInitial(): State {
  const fm = props.initial?.frontmatter
  return {
    id: props.initial?.id ?? '',
    scope: (props.initial?.scope ?? props.defaultScope ?? 'global') as OutputStyleScope,
    workingDir: props.initial?.workingDir ?? props.defaultWorkingDir ?? '',
    name: fm?.name ?? '',
    description: fm?.description ?? '',
    keepCodingInstructions: fm?.keepCodingInstructions ?? false,
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

function build(): OutputStyleInput {
  return {
    id: state.id.trim(),
    scope: state.scope,
    workingDir: state.scope === 'project' ? state.workingDir.trim() : null,
    frontmatter: {
      name: state.name.trim() || null,
      description: state.description.trim() || null,
      keepCodingInstructions: state.keepCodingInstructions || null,
      extra: {},
    } as OutputStyleInput['frontmatter'],
    body: state.body,
  } as OutputStyleInput
}

function onSubmit() {
  const input = build()
  const r = outputStyleSchema.safeParse(input)
  if (!r.success) {
    errors.value = flattenErrors(r.error)
    return
  }
  if (input.scope === 'project' && !input.workingDir) {
    errors.value = { workingDir: 'project scope requires working dir' }
    return
  }
  errors.value = {}
  emit('submit', input)
  draft.clear()
}
</script>

<template>
  <form class="grid grid-cols-1 gap-4 lg:grid-cols-2" @submit.prevent="onSubmit">
    <FormField label="ID" required :error="errors.id">
      <input v-model="state.id" :readonly="lockId" type="text" class="ccg-input" />
    </FormField>
    <FormField label="Scope">
      <select v-model="state.scope" class="ccg-input">
        <option value="global">global</option>
        <option value="project">project</option>
      </select>
    </FormField>
    <FormField
      v-if="state.scope === 'project'"
      label="Working dir"
      :error="errors.workingDir"
      class="lg:col-span-2"
    >
      <input v-model="state.workingDir" type="text" class="ccg-input" />
    </FormField>
    <FormField label="Name" :error="errors['frontmatter.name']">
      <input v-model="state.name" type="text" class="ccg-input" />
    </FormField>
    <FormField label="Keep coding instructions" class="flex-row items-center">
      <label class="inline-flex items-center gap-2">
        <input v-model="state.keepCodingInstructions" type="checkbox" />
        <span class="text-xs text-neutral-500 dark:text-neutral-400">
          Preserve coding sections from base style
        </span>
      </label>
    </FormField>
    <FormField label="Description" class="lg:col-span-2">
      <textarea v-model="state.description" rows="2" class="ccg-input" />
    </FormField>
    <FormField label="Body" class="lg:col-span-2">
      <MarkdownEditor v-model="state.body" min-height="320px" />
    </FormField>
    <div class="lg:col-span-2 flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
      <button type="button" class="ccg-btn-ghost" @click="emit('cancel')">Cancel</button>
      <button type="submit" :disabled="submitting" class="ccg-btn-primary">
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
