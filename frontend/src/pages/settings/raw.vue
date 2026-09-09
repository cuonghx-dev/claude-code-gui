<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import JsonEditor from '@/components/JsonEditor.vue'
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
  <section class="flex h-[calc(100vh-13rem)] min-h-0 flex-col gap-3 p-6">
    <div class="flex items-center gap-2">
      <p class="font-mono text-xs text-neutral-500 dark:text-neutral-400">
        {{ doc.data.value?.path }}
      </p>
      <button
        type="button"
        class="ccg-btn-primary ml-auto"
        :disabled="readOnly || put.isPending.value"
        @click="save"
      >
        {{ put.isPending.value ? 'Saving…' : 'Save' }}
      </button>
    </div>
    <JsonEditor v-model="content" :disabled="readOnly" fill class="min-h-0 flex-1" />
  </section>
</template>
