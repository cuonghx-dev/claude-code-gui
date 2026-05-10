<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { usePlugin } from '@/composables/usePlugins'

const route = useRoute()
const id = computed(() => (route.params as { id: string }).id)
const { isPending, isError, error, data } = usePlugin(id)
</script>

<template>
  <PageHeader :title="data?.name ?? id" :subtitle="data?.version ? `v${data.version}` : undefined" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: detail }">
      <section v-if="detail" class="p-6 space-y-4">
        <dl class="grid grid-cols-3 gap-x-3 gap-y-2 rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">
          <dt class="text-neutral-500">Description</dt>
          <dd class="col-span-2">{{ detail.description ?? '—' }}</dd>
          <dt class="text-neutral-500">Skills</dt>
          <dd class="col-span-2">{{ detail.skills.join(', ') || '—' }}</dd>
          <dt class="text-neutral-500">Path</dt>
          <dd class="col-span-2 break-all font-mono text-xs">{{ detail.dir }}</dd>
        </dl>
        <pre v-if="detail.readme" class="max-h-[60vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ detail.readme }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
</template>
