<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useAgent } from '@/composables/useAgents'

const route = useRoute()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useAgent(slug)
</script>

<template>
  <PageHeader :title="data?.frontmatter?.name ?? slug" subtitle="Edit agent (Phase 2 enables write)" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: agent }">
      <section v-if="agent" class="grid grid-cols-1 gap-6 p-6 lg:grid-cols-2">
        <div class="space-y-3">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400">Frontmatter</h3>
          <dl class="grid grid-cols-3 gap-x-3 gap-y-2 rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">
            <dt class="text-neutral-500 dark:text-neutral-400">Slug</dt>
            <dd class="col-span-2 font-mono text-xs">{{ agent.slug }}</dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Name</dt>
            <dd class="col-span-2">{{ agent.frontmatter.name ?? '—' }}</dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Description</dt>
            <dd class="col-span-2">{{ agent.frontmatter.description ?? '—' }}</dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Model</dt>
            <dd class="col-span-2">{{ agent.frontmatter.model ?? '—' }}</dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Memory</dt>
            <dd class="col-span-2">{{ agent.frontmatter.memory ?? '—' }}</dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Skills</dt>
            <dd class="col-span-2">
              <span v-if="!agent.frontmatter.skills.length">—</span>
              <span v-for="s in agent.frontmatter.skills" :key="s" class="mr-1 rounded bg-violet-50 px-1.5 py-0.5 text-xs text-violet-700 dark:bg-violet-950/40 dark:text-violet-300">{{ s }}</span>
            </dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Tools</dt>
            <dd class="col-span-2">
              <span v-if="!agent.frontmatter.tools.length">—</span>
              <span v-for="t in agent.frontmatter.tools" :key="t" class="mr-1 rounded bg-neutral-100 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300">{{ t }}</span>
            </dd>
            <dt class="text-neutral-500 dark:text-neutral-400">Path</dt>
            <dd class="col-span-2 break-all font-mono text-xs">{{ agent.filePath }}</dd>
          </dl>
        </div>
        <div class="space-y-3">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400">Body</h3>
          <pre class="max-h-[70vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm leading-relaxed dark:border-neutral-800 dark:bg-neutral-900">{{ agent.body }}</pre>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
