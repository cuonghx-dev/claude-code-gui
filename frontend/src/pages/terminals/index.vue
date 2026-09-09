<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useTerminalsList } from '@/composables/useCliHistory'

const { isPending, isError, error, data } = useTerminalsList()

const fmt = (iso: string | null) =>
  iso ? new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'short' }).format(new Date(iso)) : '—'
</script>

<template>
  <PageHeader
    title="Terminals"
    :subtitle="`${data?.length ?? 0} recorded sessions in ~/.claude/cli-history/`"
  />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState
          v-if="!items?.length"
          title="No recorded terminals"
          hint="Sessions started from this app are saved here when they exit."
        />
        <ul
          v-else
          class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <li v-for="t in items" :key="t.id">
            <RouterLink
              :to="`/terminals/${t.id}`"
              class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800"
            >
              <div class="flex items-baseline justify-between gap-3">
                <span class="truncate text-sm font-semibold">
                  {{ t.preview || `Session ${t.id.slice(0, 8)}…` }}
                </span>
                <span class="flex shrink-0 items-center gap-3 text-xs text-neutral-400">
                  <span
                    v-if="t.exitCode !== null && t.exitCode !== 0"
                    class="rounded bg-red-500/10 px-1.5 py-0.5 text-red-600 dark:text-red-400"
                  >
                    exit {{ t.exitCode }}
                  </span>
                  <span>{{ t.lineCount.toLocaleString() }} lines</span>
                  <span>{{ fmt(t.endedAt) }}</span>
                </span>
              </div>
              <p class="mt-0.5 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                {{ t.workingDir ?? t.id }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
