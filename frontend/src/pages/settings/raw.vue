<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import JsonEditor from '@/components/JsonEditor.vue'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import { useSettingsRaw, useSettingsRawPut } from '@/composables/useSettings'

const ctx = inject(SETTINGS_CONTEXT)!
const doc = useSettingsRaw(ctx.scope, ctx.workingDir)
const put = useSettingsRawPut()

const content = ref('')
watch(
  () => doc.data.value,
  (d) => {
    content.value = d?.content || '{\n}\n'
  },
  { immediate: true },
)

const dirty = computed(() => content.value !== (doc.data.value?.content || '{\n}\n'))
useUnsavedChanges(dirty)

const readOnly = computed(() => ctx.scope.value === 'managed')

const save = async () => {
  try {
    await put.mutateAsync({
      scope: ctx.scope.value,
      workingDir: ctx.workingDir.value,
      content: content.value,
      // The CLI writes these files too, so a save refuses to clobber a version
      // that changed on disk since it was read.
      expectedMtimeMs: doc.data.value?.mtimeMs ?? undefined,
    })
    toast.success('Saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}
</script>

<template>
  <section class="flex min-h-[360px] flex-1 flex-col px-7 py-[22px]">
    <JsonEditor v-model="content" :disabled="readOnly" fill min-height="0">
      <template #actions>
        <button
          type="button"
          class="ccg-btn-primary ccg-btn-sm"
          :disabled="readOnly || !dirty || put.isPending.value"
          @click="save"
        >
          {{ put.isPending.value ? 'Saving…' : 'Save' }}
        </button>
      </template>
    </JsonEditor>
  </section>
</template>
