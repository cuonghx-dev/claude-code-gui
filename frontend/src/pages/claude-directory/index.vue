<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ExternalLink, X } from 'lucide-vue-next'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ClaudeDirNode from '@/components/ClaudeDirNode.vue'
import { useClaudeDirectoryTree } from '@/composables/useClaudeDirectory'
import { useProjectsList } from '@/composables/useProjects'
import { filesRead } from '@/utils/ipc'
import type { ClaudeDirEntry, ClaudeDirTree } from '@/types/ipc'

const projects = useProjectsList()
const projectPath = ref<string>('')
const { isPending, isError, error, data } = useClaudeDirectoryTree(projectPath)

const selected = ref<ClaudeDirEntry | null>(null)
const preview = ref<string>('')
const previewError = ref<string>('')
const previewLoading = ref(false)

async function select(entry: ClaudeDirEntry) {
  if (entry.kind === 'dir' || !entry.exists) return
  selected.value = entry
  preview.value = ''
  previewError.value = ''
  previewLoading.value = true
  try {
    preview.value = await filesRead(entry.path)
  } catch (e) {
    previewError.value = (e as { message?: string })?.message ?? String(e)
  } finally {
    previewLoading.value = false
  }
}

// A different project means the old selection points at a stale path.
watch(projectPath, () => {
  selected.value = null
  preview.value = ''
  previewError.value = ''
})

const scopeLabel = (t: ClaudeDirTree) => (t.scope === 'global' ? 'Global' : 'Project')

const presentCount = (t: ClaudeDirTree) => t.entries.filter((e: ClaudeDirEntry) => e.exists).length

const relPath = (t: ClaudeDirTree, e: ClaudeDirEntry) =>
  e.path.startsWith(`${t.root}/`) ? e.path.slice(t.root.length + 1) : e.path

// Files this app has a dedicated editor for get a jump link rather than only a
// read-only preview.
const editorRoute = computed(() => {
  const f = selected.value
  if (!f) return null
  if (f.label === 'settings.json' || f.label === 'settings.local.json') return '/settings/raw'
  if (f.label === 'keybindings.json') return '/settings/keybindings'
  if (f.label === 'CLAUDE.md' || f.label === 'CLAUDE.local.md') return '/memory'
  if (f.path.includes('/.claude/rules/') || f.path.includes('/rules/')) return '/memory'
  if (f.label === '.mcp.json') return '/mcp'
  if (f.label === '.worktreeinclude' && projectPath.value) {
    const p = projects.data.value?.find((x) => x.workingDir === projectPath.value)
    if (p) return `/sessions/project/${encodeURIComponent(p.name)}/worktrees`
  }
  return null
})

const subtitle = computed(() =>
  projectPath.value
    ? 'What Claude Code reads from ~/.claude and this project’s .claude'
    : 'What Claude Code reads from ~/.claude — pick a project to add its .claude',
)
</script>

<template>
  <PageHeader title="Claude directory" :subtitle="subtitle">
    <template #actions>
      <select
        v-model="projectPath"
        aria-label="Project scope"
        class="rounded-md border border-neutral-300 bg-white px-2 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-900"
      >
        <option value="">No project</option>
        <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
          {{ p.workingDir }}
        </option>
      </select>
      <button
        class="ccg-btn-ghost inline-flex items-center gap-1"
        @click="openUrl('https://code.claude.com/docs/en/claude-directory')"
      >
        <ExternalLink class="h-3.5 w-3.5" /> Docs
      </button>
    </template>
  </PageHeader>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: trees }">
      <section class="flex h-full min-h-0 gap-6 p-6">
        <div class="min-w-0 flex-1 space-y-8 overflow-auto">
          <div v-for="tree in trees ?? []" :key="tree.scope">
            <div class="mb-3 flex flex-wrap items-baseline gap-2">
              <span class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
                {{ scopeLabel(tree) }}
              </span>
              <span class="font-mono text-[11px] text-neutral-500 dark:text-neutral-400">{{ tree.root }}</span>
              <span class="text-xs text-neutral-500 dark:text-neutral-400">
                {{ presentCount(tree) }} / {{ tree.entries.length }} present
              </span>
            </div>

            <ul class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
              <ClaudeDirNode
                v-for="entry in tree.entries"
                :key="entry.id"
                :entry="entry"
                :label="relPath(tree, entry)"
                :project-path="projectPath || undefined"
                @select="select"
              />
            </ul>
          </div>
        </div>

        <aside
          v-if="selected"
          class="flex w-[36rem] max-w-[45%] shrink-0 flex-col rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900"
        >
          <div class="flex items-start gap-2 border-b border-neutral-200 px-4 py-3 dark:border-neutral-800">
            <div class="min-w-0 flex-1">
              <p class="truncate font-mono text-sm font-medium text-neutral-900 dark:text-neutral-100">{{ selected.label }}</p>
              <p class="truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">{{ selected.path }}</p>
            </div>
            <RouterLink
              v-if="editorRoute"
              :to="editorRoute"
              class="ccg-btn-ghost whitespace-nowrap"
            >
              Edit
            </RouterLink>
            <button
              v-if="selected.docsUrl"
              class="ccg-btn-ghost inline-flex items-center gap-1"
              @click="openUrl(selected.docsUrl)"
            >
              <ExternalLink class="h-3.5 w-3.5" /> Docs
            </button>
            <button
              type="button"
              aria-label="Close preview"
              class="rounded p-1 text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
              @click="selected = null"
            >
              <X class="h-4 w-4" />
            </button>
          </div>
          <div class="min-h-0 flex-1 overflow-auto p-4">
            <p v-if="previewLoading" class="text-sm text-neutral-500 dark:text-neutral-400">Loading…</p>
            <p v-else-if="previewError" class="text-sm text-red-700 dark:text-red-300">{{ previewError }}</p>
            <pre v-else class="whitespace-pre-wrap break-words font-mono text-xs text-neutral-700 dark:text-neutral-300">{{ preview }}</pre>
          </div>
        </aside>
      </section>
    </template>
  </QueryStateBoundary>
</template>
