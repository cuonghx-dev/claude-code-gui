<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { toast } from 'vue-sonner'
import PageHeader from '@/components/PageHeader.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
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
  <PageHeader
    :title="doc.data.value?.file.relPath ?? id"
    :subtitle="doc.data.value?.file.path ?? ''"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="showPreview = !showPreview">
        {{ showPreview ? 'Hide flattened' : 'Preview flattened' }}
      </button>
      <button type="button" class="ccg-btn-primary" :disabled="!dirty || put.isPending.value" @click="save">
        {{ put.isPending.value ? 'Saving…' : 'Save' }}
      </button>
      <RouterLink to="/memory" class="ccg-btn-ghost">Back</RouterLink>
    </template>
  </PageHeader>

  <QueryStateBoundary
    :is-pending="doc.isPending.value"
    :is-error="doc.isError.value"
    :error="doc.error.value"
    :data="doc.data.value"
  >
    <template #default="{ data: d }">
      <section v-if="d" class="flex h-[calc(100vh-65px)] min-h-0 gap-4 p-6">
        <MarkdownEditor v-model="body" fill class="min-h-0 flex-1" />

        <aside class="w-80 shrink-0 space-y-4 overflow-y-auto">
          <div v-if="d.imports.length">
            <h3 class="text-sm font-semibold">Imports</h3>
            <ul class="mt-1 space-y-1 text-xs">
              <li v-for="(imp, i) in d.imports" :key="i" class="break-all">
                <span class="font-mono">@{{ imp.raw }}</span>
                <span v-if="!imp.exists" class="ml-1 text-red-600 dark:text-red-400">missing</span>
                <span v-else-if="imp.cyclic" class="ml-1 text-amber-600 dark:text-amber-400">
                  cycle
                </span>
                <span v-if="imp.depth > 1" class="ml-1 text-neutral-400">depth {{ imp.depth }}</span>
              </li>
            </ul>
          </div>
          <p v-else class="text-xs text-neutral-500 dark:text-neutral-400">No imports.</p>

          <div v-if="showPreview && preview.data.value">
            <h3 class="text-sm font-semibold">Flattened</h3>
            <p class="text-xs text-neutral-500 dark:text-neutral-400">
              {{ preview.data.value.sources.length }} files
              <span v-if="preview.data.value.truncated"> · truncated</span>
            </p>
            <pre class="mt-1 max-h-96 overflow-auto rounded bg-neutral-100 p-2 font-mono text-[11px] dark:bg-neutral-900">{{ preview.data.value.flattened }}</pre>
          </div>
        </aside>
      </section>
    </template>
  </QueryStateBoundary>
</template>
