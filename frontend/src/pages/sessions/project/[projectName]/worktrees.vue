<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import { useProjectWorktreeInclude, useProjectWorktrees } from '@/composables/useProjects'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)

const worktrees = useProjectWorktrees(projectName)
const include = useProjectWorktreeInclude(projectName)
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-auto">
    <PageHeader title="Worktrees" subtitle="Read-only: this app never creates or removes worktrees" />

    <section class="flex flex-col gap-6 px-7 py-5">
      <div v-if="worktrees.isPending.value" class="flex flex-col gap-2">
        <div v-for="i in 2" :key="i" class="ccg-skeleton h-14" />
      </div>
      <p v-else-if="!worktrees.data.value?.length" class="text-[13px]" style="color: var(--ccg-subtle);">
        Not a git repository.
      </p>

      <ul v-else class="ccg-card divide-y overflow-hidden">
        <li
          v-for="w in worktrees.data.value"
          :key="w.path"
          class="flex flex-col gap-1 px-4 py-3"
          style="border-color: var(--ccg-hairline-soft);"
        >
          <div class="flex items-center gap-2">
            <span class="font-mono text-[13px] font-medium text-ink">{{ w.name }}</span>
            <span v-if="w.isMain" class="ccg-badge">main checkout</span>
            <span
              v-if="w.isCurrent"
              class="ccg-badge"
              style="background: var(--ccg-success-bg); color: var(--ccg-success);"
            >current</span>
            <span
              v-if="w.isLocked"
              class="ccg-badge"
              style="background: var(--ccg-warning-bg); color: var(--ccg-warning);"
            >locked{{ w.lockReason ? `: ${w.lockReason}` : '' }}</span>
            <span
              v-if="w.prunable"
              class="ccg-badge"
              style="background: var(--ccg-error-bg); color: var(--ccg-error);"
            >directory missing — prunable</span>
            <span class="ml-auto font-mono text-[12px]" style="color: var(--ccg-muted);">
              {{ w.branch ?? 'detached' }}<template v-if="w.head"> · {{ w.head }}</template>
            </span>
          </div>
          <p class="ccg-path">{{ w.path }}</p>
        </li>
      </ul>

      <div class="flex flex-col gap-2">
        <h3 class="ccg-section-label">.worktreeinclude</h3>
        <p class="text-[12.5px]" style="color: var(--ccg-muted);">
          Gitignored files a new worktree should still get. Parsed with gitignore semantics, so
          <span class="font-mono">!</span> negation works.
        </p>
        <p
          v-if="include.data.value && !include.data.value.exists"
          class="text-[13px]"
          style="color: var(--ccg-subtle);"
        >
          Not present in this project.
        </p>
        <div v-else-if="include.data.value" class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div class="flex flex-col gap-1.5">
            <p class="text-[12px] font-medium text-body">Patterns</p>
            <pre class="ccg-code-block">{{ include.data.value.patterns.join('\n') }}</pre>
          </div>
          <div class="flex flex-col gap-1.5">
            <p class="text-[12px] font-medium text-body">
              Matching files ({{ include.data.value.matchedFiles.length }})
            </p>
            <pre class="ccg-code-block max-h-80">{{ include.data.value.matchedFiles.join('\n') }}</pre>
            <p v-if="include.data.value.truncated" class="text-[12px]" style="color: var(--ccg-subtle);">
              Listing truncated.
            </p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
