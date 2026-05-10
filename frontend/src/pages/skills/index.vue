<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useSkillsList } from '@/composables/useSkills'

const { isPending, isError, error, data } = useSkillsList()
</script>

<template>
  <PageHeader title="Skills" :subtitle="`${data?.length ?? 0} skills (local + plugin)`" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No skills" />
        <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <li v-for="s in items" :key="s.slug">
            <RouterLink :to="`/skills/${s.slug}`" class="block rounded-lg border border-neutral-200 bg-white p-4 hover:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
              <div class="flex items-center gap-2">
                <span class="text-sm font-semibold">{{ s.frontmatter.name ?? s.slug }}</span>
                <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400">
                  {{ s.source.kind === 'plugin' ? `plugin: ${s.source.id}` : 'local' }}
                </span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">{{ s.frontmatter.description ?? '—' }}</p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
