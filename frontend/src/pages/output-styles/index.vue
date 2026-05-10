<script setup lang="ts">
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useOutputStylesList } from '@/composables/useOutputStyles'

const { isPending, isError, error, data } = useOutputStylesList()
</script>

<template>
  <PageHeader title="Output styles" subtitle="Global styles in ~/.claude/output-styles/" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No output styles" />
        <ul v-else class="space-y-3">
          <li v-for="s in items" :key="s.id" class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-sm font-semibold">{{ s.frontmatter.name ?? s.id }}</span>
              <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide dark:bg-neutral-800">{{ s.scope }}</span>
            </div>
            <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">{{ s.frontmatter.description ?? '—' }}</p>
            <pre class="mt-3 max-h-48 overflow-auto rounded border border-neutral-100 bg-neutral-50 p-3 text-xs dark:border-neutral-800 dark:bg-neutral-950">{{ s.body }}</pre>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
