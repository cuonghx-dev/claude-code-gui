<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useSkill } from '@/composables/useSkills'

const route = useRoute()
const slug = computed(() => (route.params as { slug: string }).slug)
const { isPending, isError, error, data } = useSkill(slug)
</script>

<template>
  <PageHeader :title="data?.frontmatter?.name ?? slug" :subtitle="data?.source.kind === 'plugin' ? `From plugin ${data.source.id}` : 'Local skill'" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: skill }">
      <section v-if="skill" class="grid grid-cols-1 gap-6 p-6 lg:grid-cols-2">
        <dl class="grid grid-cols-3 gap-x-3 gap-y-2 rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">
          <dt class="text-neutral-500">Description</dt>
          <dd class="col-span-2">{{ skill.frontmatter.description ?? '—' }}</dd>
          <dt class="text-neutral-500">Context</dt>
          <dd class="col-span-2">{{ skill.frontmatter.context ?? '—' }}</dd>
          <dt class="text-neutral-500">Agent</dt>
          <dd class="col-span-2">{{ skill.frontmatter.agent ?? '—' }}</dd>
          <dt class="text-neutral-500">Directory</dt>
          <dd class="col-span-2 break-all font-mono text-xs">{{ skill.directory }}</dd>
        </dl>
        <pre class="max-h-[70vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ skill.body }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
</template>
