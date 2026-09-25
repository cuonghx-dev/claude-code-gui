<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useAgentMemoryList, useMemoryList } from '@/composables/useMemory'
import { useProjectsList } from '@/composables/useProjects'

const workingDir = ref<string | undefined>(undefined)
const projects = useProjectsList()
const { isPending, isError, error, data } = useMemoryList(workingDir)
const agentMemory = useAgentMemoryList(workingDir)
const search = ref('')

const LABEL: Record<string, string> = {
  claudeMd: 'CLAUDE.md',
  claudeLocalMd: 'CLAUDE.local.md',
  rule: 'rule',
  agentMemory: 'agent memory',
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (f) =>
      f.relPath.toLowerCase().includes(q) ||
      f.path.toLowerCase().includes(q) ||
      (f.title?.toLowerCase().includes(q) ?? false),
  )
})
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Memory" subtitle="~/.claude/CLAUDE.md · ~/.claude/rules/**/*.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <select v-model="workingDir" class="ccg-input w-64 font-mono text-[12px]" aria-label="Project">
          <option :value="undefined">User scope only</option>
          <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
            {{ p.workingDir }}
          </option>
        </select>
      </template>
    </PageHeader>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="filtered"
      skeleton="cards"
    >
      <template #default="{ data: files }">
        <EmptyState
          v-if="!files?.length"
          :title="search.trim() ? `No memory files match “${search.trim()}”.` : 'No memory files.'"
        />
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="f in files" :key="f.id" class="min-w-0">
            <RouterLink
              :to="{ path: `/memory/${encodeURIComponent(f.id)}`, query: { workingDir } }"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">
                  {{ f.relPath }}
                </span>
                <span class="ccg-badge">{{ LABEL[f.kind] }}</span>
              </div>
              <p
                class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
                style="color: var(--ccg-body); text-wrap: pretty;"
              >
                {{ f.title ?? f.path }}
              </p>
              <div class="flex flex-wrap gap-1">
                <span class="ccg-chip">{{ f.scope }}</span>
                <span v-if="f.importCount" class="ccg-chip">{{ f.importCount }} imports</span>
                <span
                  v-if="!f.exists"
                  class="ccg-chip"
                  style="background: var(--ccg-warning-bg); border-color: transparent; color: var(--ccg-warning);"
                >not created</span>
              </div>
            </RouterLink>
          </li>
        </ul>

        <section v-if="agentMemory.data.value?.length" class="flex flex-col gap-2 px-7 pb-6">
          <div class="flex items-baseline gap-2">
            <span class="text-[13px] font-semibold text-ink">Agent memory</span>
            <span class="text-[12px]" style="color: var(--ccg-muted-soft);">
              Written by agents for themselves. Read-only here.
            </span>
          </div>
          <ul class="ccg-card overflow-hidden">
            <li
              v-for="f in agentMemory.data.value"
              :key="f.id"
              class="border-b px-3 py-2 font-mono text-[12.5px] text-ink last:border-b-0"
              style="border-color: var(--ccg-hairline-faint);"
            >
              {{ f.relPath }}
            </li>
          </ul>
        </section>
      </template>
    </QueryStateBoundary>
  </div>
</template>
