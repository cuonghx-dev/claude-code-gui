<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useCommandsList } from '@/composables/useCommands'

const { isPending, isError, error, data } = useCommandsList()
</script>

<template>
  <PageHeader title="Commands" :subtitle="`${data?.length ?? 0} slash commands`">
    <template #actions>
      <RouterLink to="/commands/new" class="ccg-btn-primary">+ New</RouterLink>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No commands" />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="c in items" :key="c.slug">
            <RouterLink :to="`/commands/${c.slug}`" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="font-mono text-sm">/{{ c.slug }}</span>
                <span v-if="c.frontmatter.argumentHint" class="text-xs text-neutral-400">{{ c.frontmatter.argumentHint }}</span>
              </div>
              <p class="mt-0.5 line-clamp-1 text-xs text-neutral-500 dark:text-neutral-400">{{ c.frontmatter.description ?? '—' }}</p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
