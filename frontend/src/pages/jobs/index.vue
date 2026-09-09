<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import JobStateBadge from '@/components/JobStateBadge.vue'
import { useJobsList } from '@/composables/useJobs'

const { isPending, isError, error, data } = useJobsList()

const relative = (iso: string | null) => {
  if (!iso) return ''
  const ms = Date.now() - new Date(iso).getTime()
  const mins = Math.round(ms / 60_000)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.round(mins / 60)
  if (hours < 24) return `${hours}h ago`
  return `${Math.round(hours / 24)}d ago`
}
</script>

<template>
  <PageHeader
    title="Jobs"
    :subtitle="`${data?.length ?? 0} background jobs in ~/.claude/jobs/`"
  />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState
          v-if="!items?.length"
          title="No background jobs"
          hint="Jobs appear here when the CLI runs work in the background."
        />
        <ul
          v-else
          class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <li v-for="j in items" :key="j.id">
            <RouterLink
              :to="`/jobs/${j.id}`"
              class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800"
            >
              <div class="flex items-baseline justify-between gap-3">
                <span class="flex min-w-0 items-center gap-2">
                  <span v-if="j.pinned" class="text-xs text-amber-500" title="Pinned">★</span>
                  <span class="truncate text-sm font-semibold">{{ j.name }}</span>
                </span>
                <span class="flex shrink-0 items-center gap-3">
                  <JobStateBadge :state="j.state" />
                  <span class="text-xs text-neutral-400">{{ relative(j.updatedAt) }}</span>
                </span>
              </div>
              <p v-if="j.detail" class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">
                {{ j.detail }}
              </p>
              <p class="mt-0.5 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                {{ j.cwd ?? j.id }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
