<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  usePlugin,
  usePluginDelete,
  usePluginSetEnabled,
} from '@/composables/usePlugins'
import { describe } from '@/utils/description'

const route = useRoute()
const router = useRouter()
const id = computed(() => (route.params as { id: string }).id)
const { isPending, isError, error, data } = usePlugin(id)

const setEnabled = usePluginSetEnabled()
const remove = usePluginDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)

async function toggle(next: boolean) {
  errorMessage.value = ''
  try {
    await setEnabled.mutateAsync({ id: id.value, enabled: next })
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(id.value)
    router.push('/plugins')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="data?.name ?? id" :subtitle="data?.version ? `v${data.version}` : undefined">
    <template #actions>
      <button
        v-if="data"
        type="button"
        class="ccg-btn-ghost"
        @click="toggle(!data.enabled)"
      >
        {{ data.enabled ? 'Disable' : 'Enable' }}
      </button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Uninstall</button>
    </template>
  </PageHeader>
  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: detail }">
      <section v-if="detail" class="p-6 space-y-4">
        <dl class="grid grid-cols-3 gap-x-3 gap-y-2 rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">
          <dt class="text-neutral-500">Description</dt>
          <dd class="col-span-2">{{ describe(detail.description, detail.readme) }}</dd>
          <dt class="text-neutral-500">Skills</dt>
          <dd class="col-span-2">{{ detail.skills.join(', ') || '—' }}</dd>
          <dt class="text-neutral-500">Enabled</dt>
          <dd class="col-span-2">{{ detail.enabled ? 'yes' : 'no' }}</dd>
          <dt class="text-neutral-500">Path</dt>
          <dd class="col-span-2 break-all font-mono text-xs">{{ detail.dir }}</dd>
        </dl>
        <pre v-if="detail.readme" class="max-h-[60vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ detail.readme }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Uninstall plugin?"
    :message="`This permanently removes the plugin directory '${id}/' from disk.`"
    confirm-label="Uninstall"
    danger
    @confirm="onDelete"
  />
</template>
