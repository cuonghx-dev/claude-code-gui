<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FormField from './FormField.vue'
import { flattenErrors, mcpServerSchema } from '@/lib/schemas'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import type { McpServerInput } from '@/types/ipc'

const props = defineProps<{
  initial?: Partial<McpServerInput>
  draftKey: string
  lockName?: boolean
  submitting?: boolean
  submitLabel?: string
}>()
const emit = defineEmits<{ submit: [McpServerInput]; cancel: [] }>()

type Kind = 'stdio' | 'httpSse'

interface State {
  name: string
  kind: Kind
  command: string
  args: string
  envText: string
  url: string
  headersText: string
}

function parseRecord(text: string): Record<string, string> {
  const out: Record<string, string> = {}
  for (const line of text.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed) continue
    const idx = trimmed.indexOf('=')
    if (idx <= 0) continue
    const key = trimmed.slice(0, idx).trim()
    const value = trimmed.slice(idx + 1).trim()
    if (key) out[key] = value
  }
  return out
}

function dumpRecord(rec?: Record<string, string>): string {
  if (!rec) return ''
  return Object.entries(rec)
    .map(([k, v]) => `${k}=${v}`)
    .join('\n')
}

function fromInitial(): State {
  const t = props.initial?.transport
  if (t && t.kind === 'stdio') {
    return {
      name: props.initial?.name ?? '',
      kind: 'stdio',
      command: t.command,
      args: (t.args ?? []).join(' '),
      envText: dumpRecord(t.env),
      url: '',
      headersText: '',
    }
  }
  if (t && t.kind === 'httpSse') {
    return {
      name: props.initial?.name ?? '',
      kind: 'httpSse',
      command: '',
      args: '',
      envText: '',
      url: t.url,
      headersText: dumpRecord(t.headers),
    }
  }
  return {
    name: props.initial?.name ?? '',
    kind: 'stdio',
    command: '',
    args: '',
    envText: '',
    url: '',
    headersText: '',
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

function build(): McpServerInput {
  if (state.kind === 'stdio') {
    return {
      name: state.name.trim(),
      transport: {
        kind: 'stdio',
        command: state.command.trim(),
        args: state.args.split(/\s+/).filter(Boolean),
        env: parseRecord(state.envText),
      },
    }
  }
  return {
    name: state.name.trim(),
    transport: {
      kind: 'httpSse',
      url: state.url.trim(),
      headers: parseRecord(state.headersText),
    },
  }
}

function onSubmit() {
  const input = build()
  const r = mcpServerSchema.safeParse(input)
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
    <FormField label="Name" required :error="errors.name">
      <input v-model="state.name" :readonly="lockName" type="text" class="ccg-input max-w-md font-mono" />
    </FormField>
    <!-- Not a FormField: its <label> would forward clicks to the first button. -->
    <div class="flex flex-col gap-1">
      <span class="text-xs font-medium text-neutral-700">Transport</span>
      <div class="ccg-seg self-start" role="group" aria-label="Transport">
        <button type="button" :aria-pressed="state.kind === 'stdio'" @click="state.kind = 'stdio'">stdio</button>
        <button type="button" :aria-pressed="state.kind === 'httpSse'" @click="state.kind = 'httpSse'">HTTP/SSE</button>
      </div>
    </div>
    <template v-if="state.kind === 'stdio'">
      <FormField label="Command" required :error="errors['transport.command']">
        <input v-model="state.command" type="text" class="ccg-input font-mono text-[12.5px]" />
      </FormField>
      <FormField label="Args" hint="Space-separated">
        <input v-model="state.args" type="text" class="ccg-input font-mono text-[12.5px]" />
      </FormField>
      <FormField label="Env" hint="One KEY=VALUE per line">
        <textarea v-model="state.envText" rows="3" class="ccg-input font-mono text-[12.5px]" />
      </FormField>
    </template>
    <template v-else>
      <FormField label="URL" required :error="errors['transport.url']">
        <input v-model="state.url" type="text" class="ccg-input font-mono text-[12.5px]" />
      </FormField>
      <FormField label="Headers" hint="One KEY=VALUE per line">
        <textarea v-model="state.headersText" rows="3" class="ccg-input font-mono text-[12.5px]" />
      </FormField>
    </template>
    <div class="flex items-center justify-end gap-2 border-t border-hairline-soft pt-4">
      <button type="button" class="ccg-btn-ghost" @click="emit('cancel')">Cancel</button>
      <button type="submit" :disabled="submitting || !dirty" class="ccg-btn-primary">
        {{ submitting ? 'Saving…' : (submitLabel ?? 'Save') }}
      </button>
    </div>
  </form>
</template>
