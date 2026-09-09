<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import { toast } from 'vue-sonner'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
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
  <section class="space-y-4 p-6">
    <div class="flex items-center gap-2">
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        Hooks run shell commands around Claude Code's tool calls. This app edits the configuration
        and never runs one.
      </p>
      <button type="button" class="ccg-btn-primary ml-auto" @click="startNew">+ New hook</button>
    </div>

    <p v-if="hooks.isPending.value" class="text-sm text-neutral-500">Loading…</p>
    <p
      v-else-if="!hooks.data.value?.length"
      class="text-sm text-neutral-500 dark:text-neutral-400"
    >
      No hooks configured.
    </p>

    <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 dark:divide-neutral-800 dark:border-neutral-800">
      <li v-for="g in hooks.data.value" :key="g.id" class="p-3">
        <div class="flex items-baseline gap-2">
          <span class="text-sm font-semibold">{{ g.event }}</span>
          <span v-if="g.matcher" class="font-mono text-xs text-neutral-500">{{ g.matcher }}</span>
          <SourceBadge :scope="g.scope" />
          <span class="ml-auto flex gap-2">
            <button type="button" class="ccg-btn-ghost text-xs" @click="startEdit(g)">Edit</button>
            <button type="button" class="ccg-btn-ghost text-xs" @click="confirming = g">
              Delete
            </button>
          </span>
        </div>
        <pre
          v-for="(e, i) in g.entries"
          :key="i"
          class="mt-1 overflow-x-auto rounded bg-neutral-100 p-2 font-mono text-xs dark:bg-neutral-900"
        >{{ e.command }}</pre>
        <p v-if="!g.entries.length" class="mt-1 text-xs text-neutral-400">
          No commands — this group does nothing.
        </p>
      </li>
    </ul>

    <div
      v-if="editing"
      class="space-y-3 rounded-lg border border-neutral-200 p-4 dark:border-neutral-800"
    >
      <h3 class="text-sm font-semibold">{{ editing.id ? 'Edit hook' : 'New hook' }}</h3>
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
          Event
          <input v-model="editing.input.event" list="hook-events" class="ccg-input" />
          <datalist id="hook-events">
            <option v-for="e in EVENTS" :key="e" :value="e" />
          </datalist>
        </label>
        <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
          Matcher (regular expression, blank matches everything)
          <input v-model="editing.input.matcher" class="ccg-input font-mono" placeholder="Bash|Edit" />
        </label>
      </div>

      <div v-for="(entry, i) in editing.input.entries" :key="i" class="space-y-2 rounded border border-neutral-200 p-3 dark:border-neutral-800">
        <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
          Command
          <textarea v-model="entry.command" rows="2" class="ccg-input font-mono text-xs" />
        </label>
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
            Timeout (seconds)
            <input v-model.number="entry.timeout" type="number" min="1" max="600" class="ccg-input" />
          </label>
          <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
            Status message
            <input v-model="entry.statusMessage" class="ccg-input" />
          </label>
        </div>
      </div>

      <div class="flex gap-2">
        <button type="button" class="ccg-btn-ghost" @click="addEntry">+ Command</button>
        <button type="button" class="ccg-btn-primary ml-auto" @click="pendingSave = true">
          Save
        </button>
        <button type="button" class="ccg-btn-ghost" @click="editing = null">Cancel</button>
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
