<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { usePlansList } from '@/composables/usePlans'

const { isPending, isError, error, data } = usePlansList()
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = [...(data.value ?? [])].sort((a, b) =>
    (b.modifiedAt ?? '').localeCompare(a.modifiedAt ?? ''),
  )
  if (!q) return items
  return items.filter(
    (p) => p.slug.toLowerCase().includes(q) || p.title.toLowerCase().includes(q),
  )
})
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Plans" subtitle="~/.claude/plans/*.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <RouterLink to="/plans/new" class="ccg-btn-primary">+ New</RouterLink>
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
          :title="search.trim() ? `No plans match “${search.trim()}”.` : 'No plans yet.'"
        >
          <RouterLink v-if="!search.trim()" to="/plans/new" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="p in items" :key="p.slug" class="min-w-0">
            <RouterLink
              :to="`/plans/${encodeURIComponent(p.slug)}`"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">{{ p.slug }}</span>
                <span v-if="p.modifiedAt" class="ccg-badge">{{ p.modifiedAt.slice(0, 10) }}</span>
              </div>
              <p
                class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
                style="color: var(--ccg-body); text-wrap: pretty;"
              >
                {{ p.title }}
              </p>
              <div class="flex flex-wrap gap-1">
                <span class="ccg-chip">{{ p.filename }}</span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
