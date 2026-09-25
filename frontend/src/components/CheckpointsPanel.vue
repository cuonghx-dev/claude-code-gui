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
    <div v-if="checkpoints.isPending.value" class="flex flex-col gap-2">
      <div v-for="i in 3" :key="i" class="ccg-skeleton h-10" />
    </div>
    <p v-else-if="!checkpoints.data.value?.length" class="text-[13px]" style="color: var(--ccg-subtle);">
      No file checkpoints in this session.
    </p>

    <ul v-else class="flex flex-col gap-3 overflow-y-auto">
      <li v-for="c in checkpoints.data.value" :key="c.messageId" class="flex flex-col gap-1">
        <p class="text-[12px]" style="color: var(--ccg-subtle);">
          {{ time(c.timestamp) }} · {{ c.files.length }} file{{ c.files.length === 1 ? '' : 's' }}
        </p>
        <ul class="ccg-card overflow-hidden">
          <li
            v-for="(f, fi) in c.files"
            :key="f.backupFileName"
            class="flex items-center gap-2 px-2.5 py-1.5"
            :class="{ 'border-t': fi > 0 }"
            :style="{
              borderColor: 'var(--ccg-hairline-soft)',
              background: selected?.backupFileName === f.backupFileName ? 'var(--ccg-surface-strong)' : undefined,
            }"
          >
            <button
              type="button"
              class="min-w-0 flex-1 truncate text-left font-mono text-[12px] text-ink hover:underline disabled:cursor-not-allowed disabled:no-underline"
              :class="{ 'font-semibold': selected?.backupFileName === f.backupFileName }"
              :title="f.trackingPath"
              :disabled="!f.exists"
              @click="selected = f"
            >
              {{ basename(f.trackingPath) }}
              <span class="ml-1" style="color: var(--ccg-muted-soft);">v{{ f.version }}</span>
              <span v-if="!f.exists" class="ml-2" style="color: var(--ccg-muted-soft);">(blob missing)</span>
            </button>
            <button
              v-if="f.exists"
              type="button"
              class="ccg-btn-ghost ccg-btn-sm"
              @click="confirming = f"
            >
              Restore
            </button>
          </li>
        </ul>
      </li>
    </ul>

    <div v-if="selected" class="min-h-0">
      <p v-if="diff.isPending.value" class="text-[12.5px]" style="color: var(--ccg-subtle);">Diffing…</p>
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
