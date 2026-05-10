<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useSessionsForProject } from '@/composables/useSessions'
import { useProject } from '@/composables/useProjects'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const { data: project } = useProject(projectName)
const { isPending, isError, error, data } = useSessionsForProject(projectName)
</script>

<template>
  <PageHeader :title="project?.workingDir ?? projectName" subtitle="Past Claude Code sessions in this project" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No sessions" />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="s in items" :key="s.sessionId">
            <RouterLink :to="`/sessions/project/${encodeURIComponent(projectName)}/session/${s.sessionId}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="font-mono text-xs">{{ s.sessionId.slice(0, 12) }}…</span>
                <span class="text-xs text-neutral-400">{{ s.messageCount }} msg · {{ Math.round(Number(s.sizeBytes) / 1024) }} KB</span>
              </div>
              <p class="mt-0.5 line-clamp-1 text-xs text-neutral-500 dark:text-neutral-400">{{ s.preview ?? '—' }}</p>
              <p class="mt-1 text-[11px] text-neutral-400">{{ s.lastMessageAt?.slice(0, 16) ?? '—' }}</p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
