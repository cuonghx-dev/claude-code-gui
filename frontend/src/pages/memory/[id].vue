<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { toast } from 'vue-sonner'
import EditorPage from '@/components/EditorPage.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useMemoryDoc, useMemoryPreview, useMemoryPut } from '@/composables/useMemory'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const id = computed(() => decodeURIComponent((route.params as { id: string }).id))
const workingDir = computed(() => (route.query.workingDir as string) || undefined)

const doc = useMemoryDoc(id, workingDir)
const put = useMemoryPut()

const body = ref('')
const initial = ref('')
watch(
  () => doc.data.value,
  (d) => {
    if (d) {
      body.value = d.content
      initial.value = d.content
    }
  },
  { immediate: true },
)

const dirty = computed(() => body.value !== initial.value)
useUnsavedChanges(dirty)

const showPreview = ref(false)
const preview = useMemoryPreview(id, workingDir, showPreview)

const save = async () => {
  try {
    await put.mutateAsync({
      id: id.value,
      content: body.value,
      workingDir: workingDir.value,
      expectedMtimeMs: doc.data.value?.file.mtimeMs ?? undefined,
    })
    initial.value = body.value
    toast.success('Saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}
</script>

<template>
  <QueryStateBoundary
    :is-pending="doc.isPending.value"
    :is-error="doc.isError.value"
    :error="doc.error.value"
    :data="doc.data.value"
  >
    <template #default="{ data: d }">
      <EditorPage
        v-if="d"
        v-model="body"
        section="Memory"
        section-to="/memory"
        :name="d.file.relPath"
        :dirty="dirty"
        :file-path="d.file.path"
      >
        <template #actions>
          <button
            type="button"
            class="ccg-btn-ghost ccg-btn-sm"
            :aria-pressed="showPreview"
            @click="showPreview = !showPreview"
          >
            Preview flattened
          </button>
          <button
            type="button"
            class="ccg-btn-primary ccg-btn-sm"
            :disabled="!dirty || put.isPending.value"
            @click="save"
          >
            {{ put.isPending.value ? 'Saving…' : 'Save' }}
          </button>
        </template>
        <template #panel>
          <aside
            class="flex w-80 flex-none flex-col overflow-y-auto border-l"
            style="border-color: var(--ccg-hairline-soft); background: #F7F5F0;"
            aria-label="Imports"
          >
            <div
              class="border-b px-4 py-3.5 text-[13px] font-semibold text-ink"
              style="border-color: var(--ccg-hairline-soft);"
            >
              Imports
            </div>
            <ul v-if="d.imports.length" class="flex flex-col gap-1.5 px-4 py-3">
              <li
                v-for="(imp, i) in d.imports"
                :key="i"
                class="flex flex-wrap items-center gap-1.5 break-all font-mono text-[11.5px] text-ink"
              >
                <span>@{{ imp.raw }}</span>
                <span
                  v-if="!imp.exists"
                  class="rounded-[3px] px-[5px] py-px font-sans text-[10.5px]"
                  style="background: var(--ccg-error-bg); color: var(--ccg-error);"
                >missing</span>
                <span
                  v-else-if="imp.cyclic"
                  class="rounded-[3px] px-[5px] py-px font-sans text-[10.5px]"
                  style="background: var(--ccg-warning-bg); color: var(--ccg-warning);"
                >cycle</span>
                <span v-if="imp.depth > 1" class="ccg-badge">depth {{ imp.depth }}</span>
              </li>
            </ul>
            <p v-else class="px-4 py-3 text-[12px]" style="color: var(--ccg-subtle);">No imports.</p>

            <template v-if="showPreview && preview.data.value">
              <div
                class="flex items-baseline gap-2 border-y px-4 py-3.5"
                style="border-color: var(--ccg-hairline-soft);"
              >
                <span class="text-[13px] font-semibold text-ink">Flattened</span>
                <span class="text-[12px]" style="color: var(--ccg-muted-soft);">
                  {{ preview.data.value.sources.length }} files<span v-if="preview.data.value.truncated"> · truncated</span>
                </span>
              </div>
              <pre class="ccg-code-block m-3 max-h-96 overflow-auto whitespace-pre-wrap break-words text-[11px]">{{ preview.data.value.flattened }}</pre>
            </template>
          </aside>
        </template>
      </EditorPage>
    </template>
  </QueryStateBoundary>
</template>
