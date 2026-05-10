<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import MarkdownEditor from '../MarkdownEditor.vue'
import { flattenErrors, planSchema } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { PlanInput } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<PlanInput>
  draftKey: string
  lockSlug?: boolean
  submitting?: boolean
  submitLabel?: string
}>()
const emit = defineEmits<{ submit: [PlanInput]; cancel: [] }>()

const state = reactive({
  slug: props.initial?.slug ?? '',
  body: props.initial?.body ?? '',
})
const errors = ref<Record<string, string>>({})
const initial = JSON.stringify(state)
const dirty = computed(() => JSON.stringify(state) !== initial)
useUnsavedChanges(dirty)
const draft = useDraftRecovery(props.draftKey, () => ({ ...state }))
const recovered = draft.load() as typeof state | undefined
if (recovered) Object.assign(state, recovered)
watch(
  () => props.initial,
  (i) => {
    state.slug = i?.slug ?? ''
    state.body = i?.body ?? ''
  },
)

function onSubmit() {
  const input: PlanInput = { slug: state.slug.trim(), body: state.body }
  const r = planSchema.safeParse(input)
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
  <form class="space-y-4" @submit.prevent="onSubmit">
    <FormField label="Slug" required :error="errors.slug" hint="Filename without .md">
      <input v-model="state.slug" :readonly="lockSlug" type="text" class="ccg-input max-w-md" />
    </FormField>
    <FormField label="Body">
      <MarkdownEditor v-model="state.body" min-height="520px" />
    </FormField>
    <div class="flex items-center justify-end gap-2 border-t border-neutral-200 pt-4 dark:border-neutral-800">
      <button type="button" class="ccg-btn-ghost" @click="emit('cancel')">Cancel</button>
      <button type="submit" :disabled="submitting" class="ccg-btn-primary">
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
