<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useSessionMessages } from '@/composables/useSessions'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const { isPending, isError, error, data } = useSessionMessages(projectName, sessionId, undefined, 200)
</script>

<template>
  <PageHeader :title="`Session ${sessionId.slice(0, 12)}…`" :subtitle="`${data?.total ?? 0} messages (showing first 200)`" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: page }">
      <section v-if="page" class="p-6 space-y-3">
        <div v-for="m in page.items" :key="m.id || Math.random()" class="rounded-md border border-neutral-200 bg-white p-3 text-sm dark:border-neutral-800 dark:bg-neutral-900">
          <div class="mb-1 flex items-baseline gap-2 text-[11px] text-neutral-400">
            <span class="rounded bg-neutral-100 px-1.5 py-0.5 uppercase tracking-wide dark:bg-neutral-800">{{ m.kind }}</span>
            <span v-if="m.role">{{ m.role }}</span>
            <span v-if="m.timestamp" class="ml-auto">{{ m.timestamp }}</span>
          </div>
          <pre v-if="m.content" class="whitespace-pre-wrap break-words text-sm text-neutral-800 dark:text-neutral-200">{{ m.content }}</pre>
          <pre v-else-if="m.toolInput" class="whitespace-pre-wrap break-words font-mono text-xs text-neutral-600 dark:text-neutral-400">tool_input: {{ JSON.stringify(m.toolInput, null, 2) }}</pre>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
