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
  <section class="flex max-w-[980px] flex-col gap-[18px] px-7 py-[22px]">
    <div v-if="!exists" class="rounded-lg border border-[#E6E2DA] bg-white">
      <EmptyState
        title="No keybindings file"
        hint="Claude Code creates this the first time you run /keybindings."
      >
        <button type="button" class="ccg-btn-primary" @click="create.mutate()">
          Create keybindings.json
        </button>
      </EmptyState>
    </div>

    <template v-else>
      <div class="flex flex-col gap-2">
        <div class="flex items-baseline gap-2">
          <span class="text-[13px] font-semibold text-ink">Bindings</span>
          <span class="text-[12px] text-[#A29E94]">
            Global to every project; a blank action unbinds the default
          </span>
        </div>
        <div class="overflow-hidden rounded-lg border border-[#E6E2DA] bg-white">
          <div
            class="kb-grid border-b border-[#ECE9E2] bg-[#FAF9F6] px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[.08em] text-[#A29E94]"
          >
            <span>Context</span><span>Keys</span><span>Action</span><span />
          </div>
          <div
            v-for="(b, i) in draft"
            :key="i"
            class="kb-grid items-center border-b border-[#F3F0EA] px-3 py-1.5 last:border-b-0"
          >
            <input v-model="b.context" list="kb-contexts" class="ccg-input h-[30px] text-[12.5px]" />
            <input v-model="b.key" class="ccg-input h-[30px] font-mono text-[12.5px]" />
            <input
              v-model="b.action"
              class="ccg-input h-[30px] font-mono text-[12.5px]"
              placeholder="chat:submit (blank unbinds)"
            />
            <button
              type="button"
              class="kb-remove"
              :aria-label="`Remove ${b.key}`"
              @click="draft.splice(i, 1)"
            >
              ×
            </button>
          </div>
          <div v-if="!draft.length" class="px-3 py-2 text-[12.5px] text-[#8A867C]">
            No custom bindings yet.
          </div>
        </div>
        <datalist id="kb-contexts">
          <option v-for="c in CONTEXTS" :key="c" :value="c" />
        </datalist>
      </div>

      <div class="flex flex-col gap-2">
        <div class="flex items-baseline gap-2">
          <span class="text-[13px] font-semibold text-ink">Add binding</span>
          <span class="text-[12px] text-[#A29E94]">
            Ctrl+C, Ctrl+D, Ctrl+M, Ctrl+I and Ctrl+H are reserved by Claude Code
          </span>
        </div>
        <div class="flex items-center gap-2">
          <input
            :value="capture"
            readonly
            class="ccg-input w-56 font-mono text-[12.5px]"
            placeholder="Press a shortcut…"
            @keydown="onCapture"
          />
          <button type="button" class="ccg-btn-ghost" :disabled="!capture" @click="addBinding">
            Add binding
          </button>
          <span
            v-if="captureIssues.length"
            class="rounded bg-[#FBE9E6] px-[7px] py-0.5 text-[11.5px] text-[#B03A2E]"
          >
            {{ captureIssues.join('; ') }}
          </span>
        </div>
      </div>

      <div class="flex justify-end">
        <button
          type="button"
          class="ccg-btn-primary"
          :disabled="put.isPending.value"
          @click="save"
        >
          {{ put.isPending.value ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </template>
  </section>
</template>

<style scoped>
.kb-grid {
  display: grid;
  grid-template-columns: 160px 180px minmax(0, 1fr) 20px;
  gap: 10px;
}
.kb-remove {
  color: #c2bdb2;
  font-size: 14px;
  line-height: 1;
  transition: color 120ms ease-out;
}
.kb-remove:hover {
  color: var(--ccg-error);
}
</style>
