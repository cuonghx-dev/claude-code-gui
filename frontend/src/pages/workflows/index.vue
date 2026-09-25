<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useWorkflowsList } from '@/composables/useWorkflows'

const { isPending, isError, error, data } = useWorkflowsList()
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = [...(data.value ?? [])].sort((a, b) =>
    (b.modifiedAt ?? '').localeCompare(a.modifiedAt ?? ''),
  )
  if (!q) return items
  return items.filter(
    (w) =>
      w.slug.toLowerCase().includes(q) ||
      w.name.toLowerCase().includes(q) ||
      w.description.toLowerCase().includes(q),
  )
})
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Workflows" subtitle="~/.claude/workflows/*.js">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <RouterLink to="/workflows/new" class="ccg-btn-primary">+ New</RouterLink>
      </template>
    </PageHeader>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="filtered"
      skeleton="cards"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          :title="search.trim() ? `No workflows match “${search.trim()}”.` : 'No workflows yet.'"
          :hint="search.trim() ? undefined : 'Save a run\'s script with `s` in /workflows, or write one here.'"
        >
          <RouterLink v-if="!search.trim()" to="/workflows/new" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="w in items" :key="w.slug" class="min-w-0">
            <RouterLink
              :to="`/workflows/${encodeURIComponent(w.slug)}`"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">/{{ w.name }}</span>
                <span v-if="w.modifiedAt" class="ccg-badge">{{ w.modifiedAt.slice(0, 10) }}</span>
              </div>
              <p
                class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
                style="color: var(--ccg-body); text-wrap: pretty;"
              >
                {{ w.description || w.filename }}
              </p>
              <div v-if="w.phases.length" class="flex flex-wrap gap-1">
                <span v-for="phase in w.phases.slice(0, 3)" :key="phase" class="ccg-chip">{{ phase }}</span>
                <span v-if="w.phases.length > 3" class="ccg-chip">+{{ w.phases.length - 3 }}</span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
