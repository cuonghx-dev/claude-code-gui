<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useMcpList } from '@/composables/useMcp'

const { isPending, isError, error, data } = useMcpList('global')
</script>

<template>
  <PageHeader title="MCP servers" subtitle="From ~/.claude/.mcp.json (global scope)" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No MCP servers" hint="Add via .mcp.json" />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="s in items" :key="s.name">
            <RouterLink :to="`/mcp/${s.name}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ s.name }}</span>
                <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide dark:bg-neutral-800">
                  {{ s.transport.kind }}
                </span>
              </div>
              <p class="mt-0.5 line-clamp-1 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                {{ s.transport.kind === 'stdio' ? s.transport.command : s.transport.url }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
