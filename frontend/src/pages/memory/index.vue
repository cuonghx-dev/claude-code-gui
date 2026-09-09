<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useAgentMemoryList, useMemoryList } from '@/composables/useMemory'
import { useProjectsList } from '@/composables/useProjects'

const workingDir = ref<string | undefined>(undefined)
const projects = useProjectsList()
const { isPending, isError, error, data } = useMemoryList(workingDir)
const agentMemory = useAgentMemoryList(workingDir)

const LABEL: Record<string, string> = {
  claudeMd: 'CLAUDE.md',
  claudeLocalMd: 'CLAUDE.local.md',
  rule: 'rule',
  agentMemory: 'agent memory',
}
</script>

<template>
  <PageHeader
    title="Memory"
    subtitle="CLAUDE.md and rules — the instructions Claude Code reads every session"
  >
    <template #actions>
      <select v-model="workingDir" class="ccg-input w-64">
        <option :value="undefined">User scope only</option>
        <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
          {{ p.workingDir }}
        </option>
      </select>
    </template>
  </PageHeader>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: files }">
      <section class="space-y-6 p-6">
        <ul class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 dark:divide-neutral-800 dark:border-neutral-800">
          <li v-for="f in files" :key="f.id">
            <RouterLink
              :to="{ path: `/memory/${encodeURIComponent(f.id)}`, query: { workingDir } }"
              class="block px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800"
            >
              <div class="flex items-baseline justify-between gap-3">
                <span class="truncate text-sm font-semibold">
                  {{ f.title ?? f.relPath }}
                </span>
                <span class="flex shrink-0 items-center gap-2 text-xs text-neutral-400">
                  <span class="rounded bg-neutral-500/10 px-1.5 py-0.5">{{ LABEL[f.kind] }}</span>
                  <span>{{ f.scope }}</span>
                  <span v-if="f.importCount">{{ f.importCount }} imports</span>
                  <span v-if="!f.exists" class="text-amber-600 dark:text-amber-400">
                    not created
                  </span>
                </span>
              </div>
              <p class="mt-0.5 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                {{ f.path }}
              </p>
            </RouterLink>
          </li>
        </ul>

        <div v-if="agentMemory.data.value?.length">
          <h3 class="mb-2 text-sm font-semibold">Agent memory</h3>
          <p class="mb-2 text-xs text-neutral-500 dark:text-neutral-400">
            Written by agents for themselves. Read-only here.
          </p>
          <ul class="space-y-1">
            <li
              v-for="f in agentMemory.data.value"
              :key="f.id"
              class="font-mono text-xs text-neutral-500 dark:text-neutral-400"
            >
              {{ f.relPath }}
            </li>
          </ul>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
