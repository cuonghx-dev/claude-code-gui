<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useCommand } from '@/composables/useCommands'

const route = useRoute()
const slug = computed(() => (route.params as { slug: string }).slug)
const { isPending, isError, error, data } = useCommand(slug)
</script>

<template>
  <PageHeader :title="`/${slug}`" :subtitle="data?.frontmatter?.description ?? undefined" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: cmd }">
      <section v-if="cmd" class="grid grid-cols-1 gap-6 p-6 lg:grid-cols-2">
        <dl class="grid grid-cols-3 gap-x-3 gap-y-2 rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">
          <dt class="text-neutral-500">Argument hint</dt>
          <dd class="col-span-2">{{ cmd.frontmatter.argumentHint ?? '—' }}</dd>
          <dt class="text-neutral-500">Allowed tools</dt>
          <dd class="col-span-2">
            <span v-if="!cmd.frontmatter.allowedTools.length">—</span>
            <span v-for="t in cmd.frontmatter.allowedTools" :key="t" class="mr-1 rounded bg-neutral-100 px-1.5 py-0.5 text-xs">{{ t }}</span>
          </dd>
          <dt class="text-neutral-500">Agent</dt>
          <dd class="col-span-2">{{ cmd.frontmatter.agent ?? '—' }}</dd>
          <dt class="text-neutral-500">Path</dt>
          <dd class="col-span-2 break-all font-mono text-xs">{{ cmd.filePath }}</dd>
        </dl>
        <pre class="max-h-[70vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm leading-relaxed dark:border-neutral-800 dark:bg-neutral-900">{{ cmd.body }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
</template>
