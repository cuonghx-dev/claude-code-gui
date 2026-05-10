<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { usePluginsList } from '@/composables/usePlugins'

const { isPending, isError, error, data } = usePluginsList()
</script>

<template>
  <PageHeader title="Plugins" subtitle="Installed plugins under ~/.claude/plugins/" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No plugins installed" hint="Phase 3 adds the marketplace Discover tab." />
        <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <li v-for="p in items" :key="p.id">
            <RouterLink :to="`/plugins/${p.id}`" class="block rounded-lg border border-neutral-200 bg-white p-4 hover:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.name }}</span>
                <span v-if="p.version" class="text-xs text-neutral-400">v{{ p.version }}</span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">{{ p.description ?? '—' }}</p>
              <p class="mt-2 text-[11px] text-neutral-400">
                {{ p.skills.length }} skill{{ p.skills.length === 1 ? '' : 's' }} ·
                <span :class="p.enabled ? 'text-emerald-600 dark:text-emerald-400' : 'text-neutral-500'">
                  {{ p.enabled ? 'enabled' : 'disabled' }}
                </span>
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
