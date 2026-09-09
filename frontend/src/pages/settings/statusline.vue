<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import {
  useStatusline,
  useStatuslineDelete,
  useStatuslinePreview,
  useStatuslinePut,
} from '@/composables/useStatusline'
import type { StatusLine } from '@/types/ipc'

const ctx = inject(SETTINGS_CONTEXT)!
const current = useStatusline(ctx.scope, ctx.workingDir)
const put = useStatuslinePut()
const del = useStatuslineDelete()
const preview = useStatuslinePreview()

const draft = ref<StatusLine>({ kind: 'command', command: null, padding: null })
watch(
  () => current.data.value,
  (s) => {
    if (s) draft.value = { ...s, kind: s.kind ?? 'command' }
  },
  { immediate: true },
)

const readOnly = computed(() => ctx.scope.value === 'managed')

const save = async () => {
  try {
    await put.mutateAsync({
      scope: ctx.scope.value,
      workingDir: ctx.workingDir.value,
      statusLine: draft.value,
    })
    toast.success('Status line saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}

const clear = async () => {
  try {
    await del.mutateAsync({ scope: ctx.scope.value, workingDir: ctx.workingDir.value })
    toast.success('Status line removed')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Failed')
  }
}

const testRun = async () => {
  try {
    await preview.mutateAsync({ statusLine: draft.value, workingDir: ctx.workingDir.value })
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Failed to run')
  }
}

// One line of output does not justify an xterm instance, so escapes are
// stripped for display.
const ANSI = /\u001b\[[0-9;?]*[ -/]*[@-~]/g
const stdout = computed(() => (preview.data.value?.stdout ?? '').replace(ANSI, ''))
</script>

<template>
  <section class="max-w-3xl space-y-4 p-6">
    <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
      Command
      <input
        v-model="draft.command"
        class="ccg-input font-mono"
        placeholder="~/bin/claude-statusline"
        :disabled="readOnly"
      />
    </label>
    <label class="flex w-40 flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
      Padding
      <input
        v-model.number="draft.padding"
        type="number"
        min="0"
        class="ccg-input"
        :disabled="readOnly"
      />
    </label>

    <div class="flex gap-2">
      <button
        type="button"
        class="ccg-btn-primary"
        :disabled="readOnly || put.isPending.value"
        @click="save"
      >
        Save
      </button>
      <button type="button" class="ccg-btn-ghost" :disabled="readOnly" @click="clear">Remove</button>
      <button
        type="button"
        class="ccg-btn-ghost ml-auto"
        :disabled="!draft.command || preview.isPending.value"
        @click="testRun"
      >
        {{ preview.isPending.value ? 'Running…' : 'Test run' }}
      </button>
    </div>
    <p class="text-xs text-amber-700 dark:text-amber-300">
      Test run executes this command on your machine, with the same JSON payload Claude Code sends
      on stdin. It is capped at 5 seconds.
    </p>

    <div v-if="preview.data.value" class="space-y-2">
      <h3 class="text-sm font-semibold">Output</h3>
      <pre
        class="overflow-x-auto rounded-lg bg-neutral-900 p-3 font-mono text-xs text-neutral-100"
      >{{ stdout || '(no output)' }}</pre>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        exit {{ preview.data.value.exitCode ?? '—' }} · {{ preview.data.value.durationMs }}ms
        <span v-if="preview.data.value.timedOut" class="text-red-600 dark:text-red-400">
          · timed out and was killed
        </span>
        <span v-if="preview.data.value.truncated"> · output truncated</span>
      </p>
      <pre
        v-if="preview.data.value.stderr"
        class="overflow-x-auto rounded-lg bg-red-500/10 p-3 font-mono text-xs text-red-700 dark:text-red-300"
      >{{ preview.data.value.stderr }}</pre>
    </div>
  </section>
</template>
