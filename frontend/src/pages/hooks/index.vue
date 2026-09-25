<script setup lang="ts">
import { computed } from 'vue'
import { Terminal, FileCode } from 'lucide-vue-next'
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
  <div class="flex h-full flex-col">
    <PageHeader title="Hooks" subtitle="settings.json → hooks" />

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="data"
      skeleton="rows"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          title="No hooks configured."
          hint="Add them under “hooks” in a settings.json file."
        />
        <div v-else class="flex flex-col gap-6 px-7 py-5">
          <section v-for="bucket in grouped" :key="bucket.event" class="flex flex-col gap-2">
            <div class="flex items-baseline gap-2">
              <span class="text-[13px] font-semibold text-ink">{{ bucket.event }}</span>
              <span
                v-if="bucket.label !== bucket.event"
                class="text-[12px]"
                style="color: var(--ccg-muted-soft);"
              >{{ bucket.label }}</span>
              <span class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">
                {{ bucket.groups.length }}
              </span>
            </div>
            <div
              v-for="(g, gi) in bucket.groups"
              :key="`${bucket.event}:${gi}`"
              class="ccg-card overflow-hidden"
              style="border-radius: 8px;"
            >
              <div
                class="flex items-center gap-2 border-b px-3 py-2"
                style="border-color: var(--ccg-hairline-faint); background: var(--ccg-canvas-soft);"
              >
                <span class="text-[11.5px]" style="color: var(--ccg-subtle);">matcher</span>
                <span class="font-mono text-[12.5px] text-ink">{{ g.matcher || '*' }}</span>
                <span class="flex-1" />
                <span class="ccg-badge">{{ g.scope }}</span>
                <span class="ccg-path max-w-[320px] truncate text-[11px]" :title="g.filePath">{{ g.filePath }}</span>
              </div>
              <div
                v-for="(entry, ei) in g.entries"
                :key="ei"
                class="flex flex-col gap-1 px-3 py-2"
                :class="ei > 0 ? 'border-t' : ''"
                style="border-color: var(--ccg-hairline-faint);"
              >
                <div class="flex items-center gap-2">
                  <FileCode
                    v-if="isScript(entry.command)"
                    :size="14"
                    :stroke-width="1.5"
                    class="flex-none"
                    style="color: var(--ccg-subtle);"
                  />
                  <Terminal
                    v-else
                    :size="14"
                    :stroke-width="1.5"
                    class="flex-none"
                    style="color: var(--ccg-subtle);"
                  />
                  <span class="font-mono text-[12.5px] font-medium text-ink">{{ deriveName(entry.command) }}</span>
                  <span
                    v-if="entry.statusMessage"
                    class="min-w-0 flex-1 truncate text-[12px]"
                    style="color: var(--ccg-muted);"
                  >{{ entry.statusMessage }}</span>
                  <span v-else class="flex-1" />
                  <span v-if="entry.timeout != null" class="ccg-chip">timeout {{ entry.timeout }}s</span>
                </div>
                <pre
                  v-if="entry.command"
                  class="whitespace-pre-wrap break-all font-mono text-[12px] leading-[1.6]"
                  style="color: var(--ccg-body);"
                >{{ entry.command }}</pre>
              </div>
            </div>
          </section>
        </div>
      </template>
    </QueryStateBoundary>
  </div>
</template>
