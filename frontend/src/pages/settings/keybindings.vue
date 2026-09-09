<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import EmptyState from '@/components/EmptyState.vue'
import {
  useKeybindings,
  useKeybindingsCreate,
  useKeybindingsPut,
  useValidateChord,
} from '@/composables/useKeybindings'
import type { Keybinding } from '@/types/ipc'

const doc = useKeybindings()
const put = useKeybindingsPut()
const create = useKeybindingsCreate()
const validate = useValidateChord()

const draft = ref<Keybinding[]>([])
watch(
  () => doc.data.value,
  (d) => {
    if (d) draft.value = JSON.parse(JSON.stringify(d.bindings))
  },
  { immediate: true },
)

const CONTEXTS = ['Global', 'Chat', 'Autocomplete', 'Confirmation', 'Transcript', 'Task', 'Select']

const capture = ref('')
const captureIssues = ref<string[]>([])

// Reading the chord from a real keypress beats asking someone to spell it.
const onCapture = async (e: KeyboardEvent) => {
  e.preventDefault()
  const parts: string[] = []
  if (e.ctrlKey) parts.push('ctrl')
  if (e.altKey) parts.push('alt')
  if (e.shiftKey) parts.push('shift')
  if (e.metaKey) parts.push('cmd')
  const key = e.key.toLowerCase()
  if (!['control', 'alt', 'shift', 'meta'].includes(key)) parts.push(key)
  const chord = parts.join('+')
  const result = await validate.mutateAsync(chord)
  capture.value = result.normalized ?? chord
  captureIssues.value = result.issues
}

const addBinding = () => {
  if (!capture.value) return
  draft.value.push({ context: 'Chat', key: capture.value, action: '' })
  capture.value = ''
  captureIssues.value = []
}

const save = async () => {
  try {
    await put.mutateAsync({
      // A blank action means "unbind the default", which the file spells null.
      bindings: draft.value.map((b) => ({ ...b, action: b.action || null })),
      expectedMtimeMs: doc.data.value?.mtimeMs ?? undefined,
    })
    toast.success('Keybindings saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}

const exists = computed(() => doc.data.value?.exists ?? false)
</script>

<template>
  <section class="space-y-4 p-6">
    <p class="font-mono text-xs text-neutral-500 dark:text-neutral-400">
      {{ doc.data.value?.path }}
    </p>

    <EmptyState
      v-if="!exists"
      title="No keybindings file"
      hint="Claude Code creates this the first time you run /keybindings."
    />
    <button v-if="!exists" type="button" class="ccg-btn-primary" @click="create.mutate()">
      Create keybindings.json
    </button>

    <template v-else>
      <table class="w-full text-sm">
        <thead class="text-left text-xs text-neutral-500 dark:text-neutral-400">
          <tr class="border-b border-neutral-200 dark:border-neutral-800">
            <th class="py-2 pr-4 font-medium">Context</th>
            <th class="py-2 pr-4 font-medium">Keys</th>
            <th class="py-2 pr-4 font-medium">Action</th>
            <th class="py-2"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(b, i) in draft"
            :key="i"
            class="border-b border-neutral-100 dark:border-neutral-900"
          >
            <td class="py-1 pr-4">
              <input v-model="b.context" list="kb-contexts" class="ccg-input text-xs" />
            </td>
            <td class="py-1 pr-4"><input v-model="b.key" class="ccg-input font-mono text-xs" /></td>
            <td class="py-1 pr-4">
              <input
                v-model="b.action"
                class="ccg-input font-mono text-xs"
                placeholder="chat:submit (blank unbinds)"
              />
            </td>
            <td class="py-1 text-right">
              <button type="button" class="ccg-btn-ghost text-xs" @click="draft.splice(i, 1)">
                Remove
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <datalist id="kb-contexts">
        <option v-for="c in CONTEXTS" :key="c" :value="c" />
      </datalist>

      <div class="flex items-center gap-2">
        <input
          :value="capture"
          readonly
          class="ccg-input w-56 font-mono text-xs"
          placeholder="Press a shortcut…"
          @keydown="onCapture"
        />
        <button type="button" class="ccg-btn-ghost" :disabled="!capture" @click="addBinding">
          Add binding
        </button>
        <span v-if="captureIssues.length" class="text-xs text-red-600 dark:text-red-400">
          {{ captureIssues.join('; ') }}
        </span>
        <button
          type="button"
          class="ccg-btn-primary ml-auto"
          :disabled="put.isPending.value"
          @click="save"
        >
          {{ put.isPending.value ? 'Saving…' : 'Save' }}
        </button>
      </div>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        Ctrl+C, Ctrl+D, Ctrl+M, Ctrl+I and Ctrl+H are reserved by Claude Code and cannot be
        rebound.
      </p>
    </template>
  </section>
</template>
