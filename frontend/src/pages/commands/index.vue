<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useCommandsList } from '@/composables/useCommands'
import { describe } from '@/utils/description'

const { isPending, isError, error, data } = useCommandsList()
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (c) =>
      c.slug.toLowerCase().includes(q) ||
      (c.frontmatter.description?.toLowerCase().includes(q) ?? false),
  )
})
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Commands" subtitle="~/.claude/commands/**/*.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <RouterLink to="/commands/new" class="ccg-btn-primary">+ New</RouterLink>
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
          :title="search.trim() ? `No commands match “${search.trim()}”.` : 'No commands yet.'"
        >
          <RouterLink v-if="!search.trim()" to="/commands/new" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="c in items" :key="c.slug" class="min-w-0">
            <RouterLink
              :to="`/commands/${encodeURIComponent(c.slug)}`"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">/{{ c.slug }}</span>
                <span v-if="c.frontmatter.agent" class="ccg-badge">{{ c.frontmatter.agent }}</span>
              </div>
              <p
                class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
                style="color: var(--ccg-body); text-wrap: pretty;"
              >
                {{ describe(c.frontmatter.description, c.body) }}
              </p>
              <div
                v-if="c.frontmatter.argumentHint || c.frontmatter.allowedTools.length"
                class="flex flex-wrap gap-1"
              >
                <span v-if="c.frontmatter.argumentHint" class="ccg-chip">{{ c.frontmatter.argumentHint }}</span>
                <span v-for="t in c.frontmatter.allowedTools.slice(0, 3)" :key="t" class="ccg-chip">{{ t }}</span>
                <span v-if="c.frontmatter.allowedTools.length > 3" class="ccg-chip">
                  +{{ c.frontmatter.allowedTools.length - 3 }}
                </span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
