<script setup lang="ts">
import { computed } from 'vue'
import { useTeamsList } from '@/composables/useTeams'

const props = defineProps<{ sessionId: string }>()

const teams = useTeamsList()

// A team is written under ~/.claude/teams/<short-id>/ but records the lead's
// full session uuid, which is what routes here.
const team = computed(() =>
  teams.data.value?.find((t) => t.leadSessionId === props.sessionId) ?? null,
)

const fmt = (ms: number | null) =>
  ms ? new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(ms) : '—'
</script>

<template>
  <div
    v-if="team"
    class="rounded-lg border border-neutral-200 p-4 text-left dark:border-neutral-800"
  >
    <div class="flex items-baseline justify-between gap-4">
      <h3 class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
        Team {{ team.name }}
      </h3>
      <span class="text-xs text-neutral-500 dark:text-neutral-400">
        {{ fmt(team.createdAtMs) }}
      </span>
    </div>
    <ul class="mt-3 space-y-2">
      <li
        v-for="m in team.members"
        :key="m.agentId"
        class="flex items-center justify-between gap-4 text-sm"
      >
        <span class="truncate">
          <span class="font-medium text-neutral-900 dark:text-neutral-100">
            {{ m.name ?? m.agentId }}
          </span>
          <span v-if="m.agentType" class="ml-2 text-xs text-neutral-500 dark:text-neutral-400">
            {{ m.agentType }}
          </span>
        </span>
        <span class="shrink-0 font-mono text-xs text-neutral-500 dark:text-neutral-400">
          {{ m.backendType ?? '—' }}<template v-if="m.tmuxPaneId"> · {{ m.tmuxPaneId }}</template>
        </span>
      </li>
    </ul>
    <p
      v-if="!team.members.length"
      class="mt-2 text-xs text-neutral-500 dark:text-neutral-400"
    >
      No members recorded.
    </p>
  </div>
</template>
