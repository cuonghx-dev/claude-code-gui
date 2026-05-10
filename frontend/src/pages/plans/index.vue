<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { usePlansList } from '@/composables/usePlans'

const { isPending, isError, error, data } = usePlansList()
</script>

<template>
  <PageHeader title="Plans" :subtitle="`${data?.length ?? 0} markdown plans in ~/.claude/plans/`">
    <template #actions>
      <RouterLink to="/plans/new" class="ccg-btn-primary">+ New</RouterLink>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No plans" />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="p in items" :key="p.slug">
            <RouterLink :to="`/plans/${p.slug}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.title }}</span>
                <span class="text-xs text-neutral-400">{{ p.modifiedAt?.slice(0, 10) }}</span>
              </div>
              <p class="mt-0.5 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">{{ p.filename }}</p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
