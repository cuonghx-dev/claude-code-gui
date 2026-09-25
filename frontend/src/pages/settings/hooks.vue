<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import { toast } from 'vue-sonner'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import EmptyState from '@/components/EmptyState.vue'
import FormField from '@/components/forms/FormField.vue'
import SourceBadge from '@/components/settings/SourceBadge.vue'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import { useHookCreate, useHookDelete, useHookUpdate, useHooksList } from '@/composables/useHooks'
import type { HookEntry, HookGroup, HookInput } from '@/types/ipc'

const ctx = inject(SETTINGS_CONTEXT)!
const hooks = useHooksList(ctx.workingDir)
const create = useHookCreate()
const update = useHookUpdate()
const remove = useHookDelete()

const EVENTS = [
  'PreToolUse',
  'PostToolUse',
  'UserPromptSubmit',
  'SessionStart',
  'SessionEnd',
  'Notification',
  'Stop',
  'SubagentStop',
  'PreCompact',
]

const blank = (): HookInput => ({
  scope: ctx.scope.value,
  workingDir: ctx.workingDir.value ?? null,
  event: 'PreToolUse',
  matcher: '',
  entries: [{ type: 'command', command: '', timeout: null, statusMessage: null }],
  expectedMtimeMs: null,
})

const editing = ref<{ id: string | null; input: HookInput } | null>(null)
const confirming = ref<HookGroup | null>(null)

const startNew = () => {
  editing.value = { id: null, input: blank() }
}

const startEdit = (g: HookGroup) => {
  editing.value = {
    id: g.id,
    input: {
      scope: g.scope,
      workingDir: ctx.workingDir.value ?? null,
      event: g.event,
      matcher: g.matcher ?? '',
      entries: JSON.parse(JSON.stringify(g.entries)) as HookEntry[],
      expectedMtimeMs: null,
    },
  }
}

const addEntry = () => {
  editing.value?.input.entries.push({
    type: 'command',
    command: '',
    timeout: null,
    statusMessage: null,
  })
}

// Saving a hook means arbitrary shell will run on future tool calls, so the
// exact command is shown for confirmation and there is no "run now" anywhere.
const pendingSave = ref(false)
const commandSummary = computed(
  () => editing.value?.input.entries.map((e) => e.command).filter(Boolean).join('\n') ?? '',
)

const doSave = async () => {
  const e = editing.value
  if (!e) return
  try {
    if (e.id) await update.mutateAsync({ id: e.id, input: e.input })
    else await create.mutateAsync(e.input)
    toast.success('Hook saved')
    editing.value = null
  } catch (err) {
    toast.error((err as Error)?.message ?? 'Save failed')
  } finally {
    pendingSave.value = false
  }
}

const doDelete = async () => {
  const g = confirming.value
  if (!g) return
  try {
    await remove.mutateAsync({ id: g.id, workingDir: ctx.workingDir.value })
    toast.success('Hook deleted')
  } catch (err) {
    toast.error((err as Error)?.message ?? 'Delete failed')
  } finally {
    confirming.value = null
  }
}
</script>

<template>
  <section class="flex max-w-[980px] flex-col gap-[18px] px-7 py-[22px]">
    <div class="flex flex-col gap-2">
      <div class="flex items-center gap-2">
        <span class="text-[13px] font-semibold text-ink">Hooks</span>
        <span class="min-w-0 flex-1 text-[12px] text-[#A29E94]">
          Shell commands around Claude Code's tool calls. This app edits the configuration and
          never runs one.
        </span>
        <button
          v-if="hooks.data.value?.length"
          type="button"
          class="ccg-btn-primary ccg-btn-sm"
          @click="startNew"
        >
          + New hook
        </button>
      </div>

      <div v-if="hooks.isPending.value" class="flex flex-col gap-2">
        <div v-for="n in 3" :key="n" class="ccg-skeleton h-[58px]" />
      </div>
      <div
        v-else-if="!hooks.data.value?.length"
        class="rounded-lg border border-[#E6E2DA] bg-white"
      >
        <EmptyState title="No hooks configured.">
          <button type="button" class="ccg-btn-primary" @click="startNew">+ New hook</button>
        </EmptyState>
      </div>

      <ul v-else class="rounded-lg border border-[#E6E2DA] bg-white">
        <li
          v-for="g in hooks.data.value"
          :key="g.id"
          class="flex flex-col gap-1.5 border-b border-[#F3F0EA] px-3 py-2.5 last:border-b-0"
        >
          <div class="flex items-center gap-2">
            <span class="text-[13px] font-semibold text-ink">{{ g.event }}</span>
            <span v-if="g.matcher" class="font-mono text-[12px] text-[#8A867C]">{{ g.matcher }}</span>
            <SourceBadge :scope="g.scope" />
            <span class="flex-1" />
            <button type="button" class="ccg-btn-ghost ccg-btn-sm" @click="startEdit(g)">Edit</button>
            <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirming = g">
              Delete
            </button>
          </div>
          <pre
            v-for="(e, i) in g.entries"
            :key="i"
            class="overflow-x-auto rounded-md bg-[#FAF9F6] px-2.5 py-1.5 font-mono text-[12px] text-[#5E5B54]"
          >{{ e.command }}</pre>
          <p v-if="!g.entries.length" class="text-[12px] text-[#A29E94]">
            No commands — this group does nothing.
          </p>
        </li>
      </ul>
    </div>

    <div v-if="editing" class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">
          {{ editing.id ? 'Edit hook' : 'New hook' }}
        </span>
        <span class="text-[12px] text-[#A29E94]">Saved to the selected scope's settings file</span>
      </div>
      <div class="flex flex-col gap-4 rounded-lg border border-[#E6E2DA] bg-white p-4">
        <div class="grid grid-cols-2 gap-3">
          <FormField label="Event">
            <input v-model="editing.input.event" list="hook-events" class="ccg-input" />
            <datalist id="hook-events">
              <option v-for="e in EVENTS" :key="e" :value="e" />
            </datalist>
          </FormField>
          <FormField label="Matcher" hint="Regular expression; blank matches everything">
            <input
              v-model="editing.input.matcher"
              class="ccg-input font-mono text-[12.5px]"
              placeholder="Bash|Edit"
            />
          </FormField>
        </div>

        <div
          v-for="(entry, i) in editing.input.entries"
          :key="i"
          class="flex flex-col gap-3 rounded-lg border border-[#ECE9E2] bg-[#FAF9F6] p-3"
        >
          <FormField label="Command">
            <textarea v-model="entry.command" rows="2" class="ccg-input font-mono text-[12.5px]" />
          </FormField>
          <div class="grid grid-cols-2 gap-3">
            <FormField label="Timeout (seconds)">
              <input
                v-model.number="entry.timeout"
                type="number"
                min="1"
                max="600"
                class="ccg-input font-mono text-[12.5px]"
              />
            </FormField>
            <FormField label="Status message">
              <input v-model="entry.statusMessage" class="ccg-input" />
            </FormField>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button type="button" class="ccg-btn-ghost" @click="addEntry">+ Command</button>
          <span class="flex-1" />
          <button type="button" class="ccg-btn-ghost" @click="editing = null">Cancel</button>
          <button type="button" class="ccg-btn-primary" @click="pendingSave = true">Save</button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :open="pendingSave"
      title="Save this hook?"
      :message="`Claude Code will run this on every matching tool call:\n\n${commandSummary}`"
      confirm-label="Save hook"
      danger
      @confirm="doSave"
      @update:open="(v: boolean) => { if (!v) pendingSave = false }"
    />

    <ConfirmDialog
      :open="!!confirming"
      title="Delete this hook?"
      :message="`Removes the ${confirming?.event ?? ''} hook from ${confirming?.filePath ?? ''}.`"
      confirm-label="Delete"
      danger
      @confirm="doDelete"
      @update:open="(v: boolean) => { if (!v) confirming = null }"
    />
  </section>
</template>
