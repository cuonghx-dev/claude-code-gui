<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useWorkflowsList } from '@/composables/useWorkflows'

const { isPending, isError, error, data } = useWorkflowsList()
</script>

<template>
  <PageHeader
    title="Workflows"
    :subtitle="`${data?.length ?? 0} dynamic-workflow scripts in ~/.claude/workflows/`"
  >
    <template #actions>
      <RouterLink to="/workflows/new" class="ccg-btn-primary">+ New</RouterLink>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState
          v-if="!items?.length"
          title="No workflows"
          hint="Save a run's script with `s` in /workflows, or write one here."
        />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="w in [...items].sort((a, b) => (b.modifiedAt ?? '').localeCompare(a.modifiedAt ?? ''))" :key="w.slug">
            <RouterLink :to="`/workflows/${w.slug}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">/{{ w.name }}</span>
                <span class="text-xs text-neutral-400">{{ w.modifiedAt?.slice(0, 10) }}</span>
              </div>
              <p v-if="w.description" class="mt-0.5 text-xs text-neutral-600 dark:text-neutral-300">
                {{ w.description }}
              </p>
              <div class="mt-1 flex flex-wrap items-center gap-1.5">
                <span class="font-mono text-[11px] text-neutral-500 dark:text-neutral-400">{{ w.filename }}</span>
                <span
                  v-for="phase in w.phases"
                  :key="phase"
                  class="rounded bg-neutral-200 px-1.5 py-0.5 text-[10px] text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100"
                >
                  {{ phase }}
                </span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
