<script setup lang="ts">
import { computed, ref } from 'vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import DiffView from '@/components/DiffView.vue'
import { useCheckpointDiff, useCheckpointRestore, useCheckpoints } from '@/composables/useCheckpoints'
import type { CheckpointFile, DiffSide } from '@/types/ipc'
import { toast } from 'vue-sonner'

const props = defineProps<{ projectName: string; sessionId: string }>()

const checkpoints = useCheckpoints(
  () => props.projectName,
  () => props.sessionId,
)

const selected = ref<CheckpointFile | null>(null)

// Left is the stored version, right is the file as it stands now: the useful
// question is "what changed since the checkpoint".
const left = computed<DiffSide | null>(() =>
  selected.value ? { kind: 'blob', backupFileName: selected.value.backupFileName } : null,
)
const right = computed<DiffSide | null>(() =>
  selected.value ? { kind: 'workingTree', path: selected.value.trackingPath } : null,
)

const diff = useCheckpointDiff(() => props.sessionId, left, right)

const restore = useCheckpointRestore()
const confirming = ref<CheckpointFile | null>(null)

const doRestore = async () => {
  const f = confirming.value
  if (!f) return
  try {
    await restore.mutateAsync({
      projectName: props.projectName,
      sessionId: props.sessionId,
      backupFileName: f.backupFileName,
      dest: f.trackingPath,
    })
    toast.success(`Restored ${basename(f.trackingPath)}`)
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Restore failed')
  } finally {
    confirming.value = null
  }
}

const basename = (p: string) => p.split('/').pop() ?? p
const time = (iso: string | null) =>
  iso ? new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(new Date(iso)) : ''
</script>

<template>
  <section class="flex min-h-0 flex-col gap-3">
    <p v-if="checkpoints.isPending.value" class="text-sm text-neutral-500">Loading checkpoints…</p>
    <p
      v-else-if="!checkpoints.data.value?.length"
      class="text-sm text-neutral-500 dark:text-neutral-400"
    >
      No file checkpoints in this session.
    </p>

    <ul v-else class="space-y-3 overflow-y-auto">
      <li v-for="c in checkpoints.data.value" :key="c.messageId">
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          {{ time(c.timestamp) }} · {{ c.files.length }} file{{ c.files.length === 1 ? '' : 's' }}
        </p>
        <ul class="mt-1 space-y-1">
          <li v-for="f in c.files" :key="f.backupFileName" class="flex items-center gap-2">
            <button
              type="button"
              class="min-w-0 flex-1 truncate text-left text-sm hover:underline"
              :class="{ 'font-semibold': selected?.backupFileName === f.backupFileName }"
              :disabled="!f.exists"
              @click="selected = f"
            >
              {{ basename(f.trackingPath) }}
              <span class="ml-1 text-xs text-neutral-400">v{{ f.version }}</span>
              <span v-if="!f.exists" class="ml-2 text-xs text-neutral-400">(blob missing)</span>
            </button>
            <button
              v-if="f.exists"
              type="button"
              class="ccg-btn-ghost text-xs"
              @click="confirming = f"
            >
              Restore
            </button>
          </li>
        </ul>
      </li>
    </ul>

    <div v-if="selected" class="min-h-0">
      <p v-if="diff.isPending.value" class="text-sm text-neutral-500">Diffing…</p>
      <DiffView v-else-if="diff.data.value" :diff="diff.data.value" />
    </div>

    <ConfirmDialog
      :open="!!confirming"
      title="Restore this version?"
      :message="`This overwrites ${confirming?.trackingPath ?? ''} with the version saved at the checkpoint. The current contents are not kept.`"
      confirm-label="Restore"
      danger
      @confirm="doRestore"
      @update:open="(v: boolean) => { if (!v) confirming = null }"
    />
  </section>
</template>
