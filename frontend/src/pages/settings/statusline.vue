<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import FormField from '@/components/forms/FormField.vue'
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
  <section class="flex max-w-[820px] flex-col gap-[18px] px-7 py-[22px]">
    <div class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">Status line</span>
        <span class="text-[12px] text-[#A29E94]">A command whose first line of stdout Claude Code shows under the prompt</span>
      </div>
      <div class="flex flex-col gap-4 rounded-lg border border-[#E6E2DA] bg-white p-4">
        <div class="grid gap-3" style="grid-template-columns: minmax(0, 1fr) 120px">
          <FormField label="Command">
            <input
              v-model="draft.command"
              class="ccg-input font-mono text-[12.5px]"
              placeholder="~/bin/claude-statusline"
              :disabled="readOnly"
            />
          </FormField>
          <FormField label="Padding">
            <input
              v-model.number="draft.padding"
              type="number"
              min="0"
              class="ccg-input font-mono text-[12.5px]"
              :disabled="readOnly"
            />
          </FormField>
        </div>
        <p class="ccg-alert-warn px-3 py-2 text-[12px]">
          Test run executes this command on your machine, with the same JSON payload Claude Code
          sends on stdin. It is capped at 5 seconds.
        </p>
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="ccg-btn-ghost"
            :disabled="!draft.command || preview.isPending.value"
            @click="testRun"
          >
            {{ preview.isPending.value ? 'Running…' : 'Test run' }}
          </button>
          <span class="flex-1" />
          <button type="button" class="ccg-btn-danger" :disabled="readOnly" @click="clear">
            Remove
          </button>
          <button
            type="button"
            class="ccg-btn-primary"
            :disabled="readOnly || put.isPending.value"
            @click="save"
          >
            Save
          </button>
        </div>
      </div>
    </div>

    <div v-if="preview.data.value" class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">Output</span>
        <span class="font-mono text-[11.5px] text-[#A29E94]">
          exit {{ preview.data.value.exitCode ?? '—' }} · {{ preview.data.value.durationMs }}ms
          <span v-if="preview.data.value.truncated"> · output truncated</span>
        </span>
        <span v-if="preview.data.value.timedOut" class="text-[12px] text-[#B03A2E]">
          timed out and was killed
        </span>
      </div>
      <pre class="ccg-code-block">{{ stdout || '(no output)' }}</pre>
      <pre
        v-if="preview.data.value.stderr"
        class="ccg-code-block border-[#EBCFC3] bg-[#FBE9E6] text-[#B03A2E]"
      >{{ preview.data.value.stderr }}</pre>
    </div>
  </section>
</template>
