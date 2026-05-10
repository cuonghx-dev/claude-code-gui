<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useAgentsList } from '@/composables/useAgents'

const { isPending, isError, error, data } = useAgentsList()
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (a) =>
      a.slug.toLowerCase().includes(q) ||
      (a.frontmatter.name?.toLowerCase().includes(q) ?? false) ||
      (a.frontmatter.description?.toLowerCase().includes(q) ?? false),
  )
})
</script>

<template>
  <PageHeader title="Agents" :subtitle="`${data?.length ?? 0} agents in ~/.claude/agents/`">
    <template #actions>
      <input
        v-model="search"
        placeholder="Filter…"
        class="w-48 rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm placeholder:text-neutral-400 focus:border-violet-500 focus:outline-none dark:border-neutral-700 dark:bg-neutral-800"
      />
    </template>
  </PageHeader>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="filtered">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState
          v-if="!items?.length"
          title="No agents yet"
          hint="Create one in ~/.claude/agents/ or use the Phase 2 wizard."
        />
        <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
          <li v-for="a in items" :key="a.slug">
            <RouterLink
              :to="`/agents/${a.slug}`"
              class="block rounded-lg border border-neutral-200 bg-white p-4 shadow-sm transition-shadow hover:shadow-md dark:border-neutral-800 dark:bg-neutral-900"
            >
              <div class="flex items-center gap-2">
                <span
                  v-if="a.frontmatter.color"
                  class="h-3 w-3 rounded-full"
                  :style="{ background: a.frontmatter.color }"
                />
                <span class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
                  {{ a.frontmatter.name ?? a.slug }}
                </span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">
                {{ a.frontmatter.description ?? '—' }}
              </p>
              <div class="mt-2 flex flex-wrap gap-1">
                <span
                  v-if="a.frontmatter.model"
                  class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
                >
                  {{ a.frontmatter.model }}
                </span>
                <span
                  v-for="t in a.frontmatter.tools.slice(0, 3)"
                  :key="t"
                  class="rounded bg-violet-50 px-1.5 py-0.5 text-[10px] text-violet-700 dark:bg-violet-950/40 dark:text-violet-300"
                >
                  {{ t }}
                </span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
