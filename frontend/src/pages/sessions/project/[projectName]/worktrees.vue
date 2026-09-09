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
  <PageHeader title="Worktrees" subtitle="Read-only: this app never creates or removes worktrees" />

  <section class="space-y-6 p-6">
    <p v-if="worktrees.isPending.value" class="text-sm text-neutral-500">Loading…</p>
    <p
      v-else-if="!worktrees.data.value?.length"
      class="text-sm text-neutral-500 dark:text-neutral-400"
    >
      Not a git repository.
    </p>

    <ul
      v-else
      class="divide-y divide-neutral-200 rounded-lg border border-neutral-200 dark:divide-neutral-800 dark:border-neutral-800"
    >
      <li v-for="w in worktrees.data.value" :key="w.path" class="px-4 py-3">
        <div class="flex items-baseline gap-2">
          <span class="text-sm font-semibold">{{ w.name }}</span>
          <span v-if="w.isMain" class="rounded bg-blue-500/10 px-1.5 py-0.5 text-[11px] text-blue-600 dark:text-blue-400">
            main checkout
          </span>
          <span v-if="w.isCurrent" class="rounded bg-emerald-500/10 px-1.5 py-0.5 text-[11px] text-emerald-600 dark:text-emerald-400">
            current
          </span>
          <span v-if="w.isLocked" class="rounded bg-amber-500/10 px-1.5 py-0.5 text-[11px] text-amber-700 dark:text-amber-300">
            locked{{ w.lockReason ? `: ${w.lockReason}` : '' }}
          </span>
          <span v-if="w.prunable" class="rounded bg-red-500/10 px-1.5 py-0.5 text-[11px] text-red-600 dark:text-red-400">
            directory missing — prunable
          </span>
          <span class="ml-auto font-mono text-xs text-neutral-500 dark:text-neutral-400">
            {{ w.branch ?? 'detached' }}<template v-if="w.head"> · {{ w.head }}</template>
          </span>
        </div>
        <p class="mt-0.5 font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
          {{ w.path }}
        </p>
      </li>
    </ul>

    <div>
      <h3 class="text-sm font-semibold">.worktreeinclude</h3>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        Gitignored files a new worktree should still get. Parsed with gitignore semantics, so
        <span class="font-mono">!</span> negation works.
      </p>
      <p
        v-if="include.data.value && !include.data.value.exists"
        class="mt-2 text-sm text-neutral-500 dark:text-neutral-400"
      >
        Not present in this project.
      </p>
      <div v-else-if="include.data.value" class="mt-2 grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <p class="text-xs font-medium text-neutral-700 dark:text-neutral-300">Patterns</p>
          <ul class="mt-1 space-y-0.5 font-mono text-xs">
            <li v-for="p in include.data.value.patterns" :key="p">{{ p }}</li>
          </ul>
        </div>
        <div>
          <p class="text-xs font-medium text-neutral-700 dark:text-neutral-300">
            Matching files ({{ include.data.value.matchedFiles.length }})
          </p>
          <ul class="mt-1 space-y-0.5 font-mono text-xs text-neutral-600 dark:text-neutral-300">
            <li v-for="f in include.data.value.matchedFiles" :key="f">{{ f }}</li>
          </ul>
          <p
            v-if="include.data.value.truncated"
            class="mt-1 text-xs text-neutral-500 dark:text-neutral-400"
          >
            Listing truncated.
          </p>
        </div>
      </div>
    </div>
  </section>
</template>
