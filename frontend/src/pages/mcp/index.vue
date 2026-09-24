<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useMcpList, useMcpScope } from '@/composables/useMcp'
import { useProjectsList } from '@/composables/useProjects'

const router = useRouter()
const { scope, projectName, workingDir, ready, query, fileLabel } = useMcpScope()
const { isPending, isError, error, data } = useMcpList(scope, workingDir, ready)
const projects = useProjectsList()

// Only projects with a known working dir can hold a .mcp.json.
const projectOptions = computed(() =>
  (projects.data.value ?? []).filter((p) => !!p.workingDir),
)

function setScope(next: 'global' | 'project') {
  if (next === 'global') {
    router.replace({ query: {} })
  } else {
    router.replace({
      query: { scope: 'project', project: projectName.value || projectOptions.value[0]?.name || '' },
    })
  }
}

function setProject(name: string) {
  router.replace({ query: { scope: 'project', project: name } })
}
</script>

<template>
  <PageHeader title="MCP servers" :subtitle="`From ${fileLabel}`">
    <template #actions>
      <RouterLink
        :to="{ path: '/mcp/new', query }"
        class="ccg-btn-primary"
        :class="!ready ? 'pointer-events-none opacity-50' : ''"
        :aria-disabled="!ready"
      >
        + New
      </RouterLink>
    </template>
  </PageHeader>
  <div class="flex flex-wrap items-center gap-2 px-6 pt-4 text-sm">
    <div role="radiogroup" aria-label="MCP scope" class="inline-flex rounded-md border border-neutral-200 p-0.5 dark:border-neutral-800">
      <button
        v-for="opt in [{ v: 'global', label: 'User' }, { v: 'project', label: 'Project' }] as const"
        :key="opt.v"
        type="button"
        role="radio"
        :aria-checked="scope === opt.v"
        class="rounded px-3 py-1 text-xs"
        :class="scope === opt.v
          ? 'bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900'
          : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800'"
        @click="setScope(opt.v)"
      >
        {{ opt.label }}
      </button>
    </div>
    <select
      v-if="scope === 'project'"
      :value="projectName"
      class="ccg-input max-w-md py-1 text-xs"
      aria-label="Project"
      @change="setProject(($event.target as HTMLSelectElement).value)"
    >
      <option value="" disabled>Choose a project…</option>
      <option v-for="p in projectOptions" :key="p.name" :value="p.name">
        {{ p.workingDir }}
      </option>
    </select>
  </div>
  <section v-if="!ready" class="p-6">
    <EmptyState title="Choose a project" hint="Project-scoped servers live in the project's .mcp.json." />
  </section>
  <QueryStateBoundary v-else :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No MCP servers" :hint="`Add one here or in ${fileLabel}`" />
        <ul v-else class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
          <li v-for="s in items" :key="s.name">
            <RouterLink :to="{ path: `/mcp/${encodeURIComponent(s.name)}`, query }" class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ s.name }}</span>
                <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide dark:bg-neutral-800">
                  {{ s.transport.kind }}
                </span>
              </div>
              <p class="mt-0.5 line-clamp-1 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                {{ s.transport.kind === 'stdio' ? s.transport.command : s.transport.url }}
              </p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
