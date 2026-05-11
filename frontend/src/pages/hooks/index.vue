<script setup lang="ts">
import { computed } from 'vue'
import { Webhook, Terminal, FileCode } from 'lucide-vue-next'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useHooksList } from '@/composables/useHooks'
import type { HookGroup } from '@/types/ipc'

const { isPending, isError, error, data } = useHooksList()

const eventLabels: Record<string, string> = {
  PreToolUse: 'Before Claude uses a tool',
  PostToolUse: 'After Claude uses a tool',
  UserPromptSubmit: 'UserPromptSubmit',
  SessionStart: 'SessionStart',
  SessionEnd: 'SessionEnd',
  Notification: 'Notification',
  Stop: 'Stop',
  SubagentStop: 'SubagentStop',
  PreCompact: 'PreCompact',
}

interface GroupedEvent {
  event: string
  label: string
  groups: HookGroup[]
}

const grouped = computed<GroupedEvent[]>(() => {
  const out = new Map<string, HookGroup[]>()
  for (const g of data.value ?? []) {
    if (!out.has(g.event)) out.set(g.event, [])
    out.get(g.event)!.push(g)
  }
  return Array.from(out.entries()).map(([event, groups]) => ({
    event,
    label: eventLabels[event] ?? event,
    groups,
  }))
})

function deriveName(command: string | null): string {
  if (!command) return ''
  const dirMatch = command.match(/\/\.claude\/hooks\/([^/]+)\//)
  if (dirMatch) return dirMatch[1]
  const quoted = command.match(/["']([^"']+)["']/)
  if (quoted) {
    const base = quoted[1].split('/').pop() ?? ''
    return base.replace(/\.(c?js|mjs|ts|sh|py)$/, '')
  }
  const first = command.split(/\s+/)[0] ?? command
  return first.split('/').pop() ?? first
}

function isScript(command: string | null): boolean {
  return !!command && /\.(c?js|mjs|ts|sh|py)/.test(command)
}
</script>

<template>
  <PageHeader
    title="Hooks"
    subtitle="Run shell commands automatically when certain events happen in Claude Code."
  />

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No hooks" />
        <div v-else class="space-y-8">
          <div v-for="bucket in grouped" :key="bucket.event">
            <div class="mb-3 flex items-center gap-2">
              <Webhook class="h-4 w-4 text-neutral-500 dark:text-neutral-400" />
              <span class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">{{ bucket.label }}</span>
              <span class="text-xs text-neutral-500 dark:text-neutral-400">{{ bucket.groups.length }}</span>
            </div>
            <div class="space-y-3">
              <div
                v-for="(g, gi) in bucket.groups"
                :key="`${bucket.event}:${gi}`"
                class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900"
              >
                <div
                  v-for="(entry, ei) in g.entries"
                  :key="ei"
                  :class="ei > 0 ? 'mt-4 border-t border-neutral-100 pt-4 dark:border-neutral-800' : ''"
                >
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="inline-flex items-center gap-1 rounded bg-neutral-100 px-2 py-0.5 text-xs font-mono text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200">
                      <FileCode v-if="isScript(entry.command)" class="h-3 w-3" />
                      <Terminal v-else class="h-3 w-3" />
                      {{ deriveName(entry.command) }}
                    </span>
                    <span
                      v-if="g.matcher"
                      class="rounded bg-neutral-100 px-2 py-0.5 text-xs font-mono text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200"
                    >
                      {{ g.matcher }}
                    </span>
                    <span
                      v-if="entry.timeout != null"
                      class="text-[11px] text-neutral-500 dark:text-neutral-400"
                    >
                      timeout {{ entry.timeout }}s
                    </span>
                  </div>
                  <p
                    v-if="entry.statusMessage"
                    class="mt-2 text-xs text-neutral-500 dark:text-neutral-400"
                  >
                    {{ entry.statusMessage }}
                  </p>
                  <pre
                    v-if="entry.command"
                    class="mt-2 overflow-auto whitespace-pre-wrap break-all rounded bg-neutral-50 p-3 font-mono text-xs text-neutral-700 dark:bg-neutral-950 dark:text-neutral-300"
                  >▸ {{ entry.command }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
