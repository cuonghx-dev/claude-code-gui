<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useAgentImport, useAgentsList } from '@/composables/useAgents'
import { describe } from '@/utils/description'
import { agentColor } from '@/utils/agentColor'

const { isPending, isError, error, data } = useAgentsList()
const search = ref('')
const router = useRouter()
const importMut = useAgentImport()
const fileInput = ref<HTMLInputElement>()
const importError = ref('')

async function onFileChosen(e: Event) {
  importError.value = ''
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const slug = file.name.replace(/\.md$/i, '')
  const content = await file.text()
  try {
    await importMut.mutateAsync({ slug, directory: '', content })
    if (fileInput.value) fileInput.value.value = ''
    router.push(`/agents/${encodeURIComponent(slug)}`)
  } catch (err) {
    importError.value = (err as { message?: string })?.message ?? String(err)
    if (fileInput.value) fileInput.value.value = ''
  }
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (a) =>
      a.slug.toLowerCase().includes(q) ||
      (a.frontmatter.name?.toLowerCase().includes(q) ?? false) ||
      (a.frontmatter.description?.toLowerCase().includes(q) ?? false),
  )
})
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Agents" subtitle="~/.claude/agents/**/*.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <input
          ref="fileInput"
          type="file"
          accept=".md,text/markdown"
          class="hidden"
          @change="onFileChosen"
        />
        <button type="button" class="ccg-btn-ghost" @click="fileInput?.click()">Import</button>
        <RouterLink to="/agents/new" class="ccg-btn-primary">+ New</RouterLink>
      </template>
    </PageHeader>
    <p v-if="importError" class="ccg-alert-error mx-7 mt-4 px-3 py-2 text-[12.5px]" role="alert">
      Import failed: {{ importError }}
    </p>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="filtered"
      skeleton="cards"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          :title="search.trim() ? `No agents match “${search.trim()}”.` : 'No agents yet.'"
        >
          <RouterLink v-if="!search.trim()" to="/agents/new" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="a in items" :key="a.slug" class="min-w-0">
            <RouterLink
              :to="`/agents/${encodeURIComponent(a.slug)}`"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span
                  class="h-[9px] w-[9px] flex-none rounded-full"
                  :style="{ background: agentColor(a.frontmatter.color) }"
                />
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">
                  {{ a.frontmatter.name ?? a.slug }}
                </span>
                <span v-if="a.frontmatter.model" class="ccg-badge">{{ a.frontmatter.model }}</span>
              </div>
              <p class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]" style="color: var(--ccg-body); text-wrap: pretty;">
                {{ describe(a.frontmatter.description, a.body) }}
              </p>
              <div v-if="a.frontmatter.tools.length" class="flex flex-wrap gap-1">
                <span v-for="t in a.frontmatter.tools.slice(0, 3)" :key="t" class="ccg-chip">{{ t }}</span>
                <span v-if="a.frontmatter.tools.length > 3" class="ccg-chip">+{{ a.frontmatter.tools.length - 3 }}</span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
