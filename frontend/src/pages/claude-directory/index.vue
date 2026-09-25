<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ExternalLink, X } from 'lucide-vue-next'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ClaudeDirNode from '@/components/ClaudeDirNode.vue'
import EmptyState from '@/components/EmptyState.vue'
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
  <div class="flex h-full flex-col">
    <PageHeader title="Claude directory" :subtitle="subtitle">
      <template #actions>
        <select
          v-model="projectPath"
          aria-label="Project scope"
          class="ccg-input max-w-[280px] font-mono text-[12px]"
        >
          <option value="">No project</option>
          <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
            {{ p.workingDir }}
          </option>
        </select>
        <button
          type="button"
          class="ccg-btn-ghost"
          @click="openUrl('https://code.claude.com/docs/en/claude-directory')"
        >
          <ExternalLink :size="14" :stroke-width="1.5" /> Docs
        </button>
      </template>
    </PageHeader>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="data"
      skeleton="rows"
    >
      <template #default="{ data: trees }">
        <div class="flex min-h-0 flex-1">
          <nav
            class="flex w-[300px] flex-none flex-col gap-4 overflow-auto border-r p-3"
            style="border-color: var(--ccg-hairline-soft);"
            aria-label=".claude tree"
          >
            <div v-for="tree in trees ?? []" :key="tree.scope" class="flex flex-col gap-1">
              <div class="flex flex-col gap-0.5 px-2 pb-1">
                <div class="flex items-baseline gap-2">
                  <span class="ccg-section-label">{{ scopeLabel(tree) }}</span>
                  <span class="flex-1" />
                  <span class="font-mono text-[10.5px]" style="color: var(--ccg-muted-soft);">
                    {{ presentCount(tree) }}/{{ tree.entries.length }}
                  </span>
                </div>
                <span class="truncate font-mono text-[11px]" style="color: var(--ccg-subtle);" :title="tree.root">
                  {{ tree.root }}
                </span>
              </div>
              <ul class="flex flex-col gap-px">
                <ClaudeDirNode
                  v-for="entry in tree.entries"
                  :key="entry.id"
                  :entry="entry"
                  :label="relPath(tree, entry)"
                  :project-path="projectPath || undefined"
                  :selected-path="selected?.path"
                  @select="select"
                />
              </ul>
            </div>
          </nav>

          <section class="flex min-w-0 flex-1 flex-col gap-[18px] overflow-auto px-7 py-[22px]">
            <EmptyState v-if="!selected" title="Select a file in the tree to preview it." />
            <template v-else>
              <div class="flex flex-col gap-1">
                <div class="flex items-center gap-2.5">
                  <h3 class="min-w-0 flex-1 truncate font-mono text-[17px] font-semibold text-ink">
                    {{ selected.label }}
                  </h3>
                  <span v-if="selected.badge" class="ccg-badge">{{ selected.badge }}</span>
                  <RouterLink v-if="editorRoute" :to="editorRoute" class="ccg-btn-primary ccg-btn-sm">Edit</RouterLink>
                  <button
                    v-if="selected.docsUrl"
                    type="button"
                    class="ccg-btn-ghost ccg-btn-sm"
                    @click="openUrl(selected.docsUrl)"
                  >
                    <ExternalLink :size="14" :stroke-width="1.5" /> Docs
                  </button>
                  <button
                    type="button"
                    aria-label="Close preview"
                    class="ccg-btn-ghost ccg-btn-sm px-2"
                    @click="selected = null"
                  >
                    <X :size="16" :stroke-width="1.5" />
                  </button>
                </div>
                <p class="ccg-path truncate" :title="selected.path">{{ selected.path }}</p>
              </div>
              <p
                v-if="selected.oneLiner"
                class="text-[13px] leading-[1.5]"
                style="color: var(--ccg-body);"
              >
                {{ selected.oneLiner }}
              </p>
              <div class="flex min-h-0 flex-col gap-1.5">
                <span class="ccg-section-label">Contents</span>
                <div v-if="previewLoading" class="ccg-skeleton h-40" style="border-radius: 8px;" />
                <p v-else-if="previewError" class="ccg-alert-error px-3 py-2 text-[12.5px]">{{ previewError }}</p>
                <pre v-else class="ccg-code-block" style="white-space: pre-wrap; word-break: break-word;">{{ preview }}</pre>
              </div>
            </template>
          </section>
        </div>
      </template>
    </QueryStateBoundary>
  </div>
</template>
