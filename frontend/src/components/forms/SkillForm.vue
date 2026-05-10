<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { flattenErrors, skillSchema } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { SkillInput } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<SkillInput>
  draftKey: string
  lockSlug?: boolean
  submitting?: boolean
  submitLabel?: string
}>()
const emit = defineEmits<{ submit: [SkillInput]; cancel: [] }>()

interface State {
  slug: string
  name: string
  description: string
  context: '' | 'when' | 'always'
  agent: string
  body: string
}

function fromInitial(): State {
  const fm = props.initial?.frontmatter
  return {
    slug: props.initial?.slug ?? '',
    name: fm?.name ?? '',
    description: fm?.description ?? '',
    context: (fm?.context ?? '') as State['context'],
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

function build(): SkillInput {
  return {
    slug: state.slug.trim(),
    frontmatter: {
      name: state.name.trim() || null,
      description: state.description.trim() || null,
      context: state.context || null,
      agent: state.agent.trim() || null,
      extra: {},
    } as SkillInput['frontmatter'],
    body: state.body,
  } as SkillInput
}

function onSubmit() {
  const input = build()
  const r = skillSchema.safeParse(input)
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
      <FormField label="Slug" required :error="errors.slug" hint="Directory name under skills/">
        <input v-model="state.slug" :readonly="lockSlug" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Name" :error="errors['frontmatter.name']">
        <input v-model="state.name" type="text" class="ccg-input" />
      </FormField>
      <FormField label="Description">
        <textarea v-model="state.description" rows="3" class="ccg-input" />
      </FormField>
      <FormField label="Context">
        <select v-model="state.context" class="ccg-input">
          <option value="">— inherit —</option>
          <option value="when">when (loaded only on trigger)</option>
          <option value="always">always (always loaded)</option>
        </select>
      </FormField>
      <FormField label="Agent" hint="Optional agent slug to bind">
        <input v-model="state.agent" type="text" class="ccg-input" />
      </FormField>
    </div>
    <div>
      <FormField label="SKILL.md body">
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
