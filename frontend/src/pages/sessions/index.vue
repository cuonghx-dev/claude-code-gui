<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useProjectsList } from '@/composables/useProjects'

const { isPending, isError, error, data } = useProjectsList()
</script>

<template>
  <PageHeader title="Sessions" subtitle="Pick a project to view past Claude Code sessions" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No projects" hint="Open `claude` in any directory to create one." />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="p in items" :key="p.name">
            <RouterLink :to="`/sessions/project/${encodeURIComponent(p.name)}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.workingDir }}</span>
                <span class="text-xs text-neutral-400">{{ p.sessionCount }} session{{ p.sessionCount === 1 ? '' : 's' }}</span>
              </div>
              <p class="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">
                Last active: {{ p.lastActive?.slice(0, 16) ?? 'never' }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
