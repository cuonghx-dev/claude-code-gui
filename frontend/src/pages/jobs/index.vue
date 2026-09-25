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
  <div class="flex h-full flex-col">
    <PageHeader
      title="Jobs"
      :subtitle="`~/.claude/jobs/ · ${data?.length ?? 0} background job${data?.length === 1 ? '' : 's'}`"
    />
    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="data"
      skeleton="rows"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          title="No background jobs. They appear here when the CLI runs work in the background."
        />
        <ul v-else class="flex flex-col gap-2 px-7 py-5">
          <li v-for="j in items" :key="j.id">
            <RouterLink
              :to="`/jobs/${j.id}`"
              class="ccg-card ccg-card-hover flex flex-col gap-1.5 px-4 py-3"
            >
              <div class="flex items-center gap-2">
                <span v-if="j.pinned" class="text-[12px]" style="color: var(--ccg-accent);" title="Pinned">★</span>
                <span class="min-w-0 flex-1 truncate text-[13.5px] font-medium text-ink">{{ j.name }}</span>
                <JobStateBadge :state="j.state" />
                <span class="w-16 text-right font-mono text-[11px]" style="color: var(--ccg-muted-soft);">
                  {{ relative(j.updatedAt) }}
                </span>
              </div>
              <p
                v-if="j.detail"
                class="line-clamp-2 text-[13px] leading-[1.45]"
                style="color: var(--ccg-body);"
              >
                {{ j.detail }}
              </p>
              <p class="truncate font-mono text-[11.5px]" style="color: var(--ccg-subtle);">
                {{ j.cwd ?? j.id }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
