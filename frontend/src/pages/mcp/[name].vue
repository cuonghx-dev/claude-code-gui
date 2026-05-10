<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useMcpServer } from '@/composables/useMcp'

const route = useRoute()
const name = computed(() => (route.params as { name: string }).name)
const { isPending, isError, error, data } = useMcpServer(name, 'global')
</script>

<template>
  <PageHeader :title="name" subtitle="MCP server detail (Phase 5 adds capability probe)" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: srv }">
      <section v-if="srv" class="p-6">
        <pre class="overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ JSON.stringify(srv, null, 2) }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
</template>
