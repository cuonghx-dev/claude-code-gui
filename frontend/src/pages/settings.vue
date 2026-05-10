<script setup lang="ts">
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { useClaudeCliInfo, useSettings } from '@/composables/useSettings'

const { isPending, isError, error, data: settings } = useSettings()
const { data: cli } = useClaudeCliInfo()
</script>

<template>
  <PageHeader title="Settings" subtitle="Read-only Phase 1 view; Phase 2 enables write" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="settings">
    <template #default="{ data: s }">
      <section class="p-6 space-y-6">
        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Claude CLI</h3>
          <dl class="mt-2 grid grid-cols-3 gap-x-3 gap-y-2 text-sm">
            <dt class="text-neutral-500">Path</dt>
            <dd class="col-span-2 break-all font-mono text-xs">{{ cli?.path ?? 'not found' }}</dd>
            <dt class="text-neutral-500">Version</dt>
            <dd class="col-span-2">{{ cli?.version ?? '—' }}</dd>
          </dl>
        </div>
        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">~/.claude/settings.json</h3>
          <dl class="mt-2 grid grid-cols-3 gap-x-3 gap-y-2 text-sm">
            <dt class="text-neutral-500">Default model</dt>
            <dd class="col-span-2">{{ s?.defaultModel ?? '—' }}</dd>
            <dt class="text-neutral-500">Default permission mode</dt>
            <dd class="col-span-2">{{ s?.defaultPermissionMode ?? '—' }}</dd>
            <dt class="text-neutral-500">Onboarding completed</dt>
            <dd class="col-span-2">{{ s?.onboardingCompleted ?? '—' }}</dd>
          </dl>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
