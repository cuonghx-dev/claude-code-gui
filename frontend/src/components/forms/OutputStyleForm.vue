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
const initialSnapshot = JSON.stringify(fromInitial())
const dirty = computed(() => JSON.stringify(state) !== initialSnapshot)
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
  <form class="flex max-w-[820px] flex-col gap-2" @submit.prevent="onSubmit">
    <div class="flex items-baseline gap-2">
      <span class="text-[13px] font-semibold text-ink">Output style</span>
      <span class="text-[12px]" style="color: var(--ccg-muted-soft);">Written as &lt;id&gt;.md in the selected scope</span>
    </div>
    <div class="flex flex-col gap-4 rounded-lg border bg-white p-4" style="border-color: var(--ccg-hairline);">
      <div class="grid grid-cols-2 gap-3">
        <FormField label="ID" required :error="errors.id">
          <input v-model="state.id" :readonly="lockId" type="text" class="ccg-input font-mono text-[12.5px]" />
        </FormField>
        <FormField label="Scope">
          <select v-model="state.scope" class="ccg-input font-mono text-[12.5px]">
            <option value="global">global</option>
            <option value="project">project</option>
          </select>
        </FormField>
        <FormField
          v-if="state.scope === 'project'"
          label="Working dir"
          :error="errors.workingDir"
          class="col-span-2"
        >
          <input v-model="state.workingDir" type="text" class="ccg-input font-mono text-[12.5px]" />
        </FormField>
        <FormField label="Name" :error="errors['frontmatter.name']">
          <input v-model="state.name" type="text" class="ccg-input" />
        </FormField>
        <div class="flex flex-col gap-1">
          <span class="text-xs font-medium text-neutral-700">Keep coding instructions</span>
          <div class="flex h-8 items-center gap-2">
            <button
              type="button"
              role="switch"
              class="ccg-switch"
              :aria-checked="state.keepCodingInstructions"
              aria-label="Keep coding instructions"
              @click="state.keepCodingInstructions = !state.keepCodingInstructions"
            />
            <span class="text-[12px]" style="color: var(--ccg-muted);">Preserve coding sections from base style</span>
          </div>
        </div>
        <FormField label="Description" class="col-span-2">
          <textarea v-model="state.description" rows="2" class="ccg-input" />
        </FormField>
      </div>
      <FormField label="Body">
        <MarkdownEditor v-model="state.body" min-height="320px" />
      </FormField>
      <div class="flex items-center justify-end gap-2">
        <button type="button" class="ccg-btn-ghost" @click="emit('cancel')">Cancel</button>
        <button type="submit" :disabled="submitting" class="ccg-btn-primary">
          {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
        </button>
      </div>
    </div>
  </form>
</template>
