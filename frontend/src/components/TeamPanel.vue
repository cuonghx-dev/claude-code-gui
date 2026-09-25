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
  <div v-if="team" class="ccg-card p-4 text-left">
    <div class="flex items-baseline justify-between gap-4">
      <h3 class="text-[13px] font-semibold text-ink">Team {{ team.name }}</h3>
      <span class="text-[12px]" style="color: var(--ccg-subtle);">{{ fmt(team.createdAtMs) }}</span>
    </div>
    <ul class="mt-3 space-y-2">
      <li
        v-for="m in team.members"
        :key="m.agentId"
        class="flex items-center justify-between gap-4 text-[13px]"
      >
        <span class="truncate">
          <span class="font-mono font-medium text-ink">{{ m.name ?? m.agentId }}</span>
          <span v-if="m.agentType" class="ccg-badge ml-2">{{ m.agentType }}</span>
        </span>
        <span class="shrink-0 font-mono text-[11.5px]" style="color: var(--ccg-muted);">
          {{ m.backendType ?? '—' }}<template v-if="m.tmuxPaneId"> · {{ m.tmuxPaneId }}</template>
        </span>
      </li>
    </ul>
    <p v-if="!team.members.length" class="mt-2 text-[12px]" style="color: var(--ccg-subtle);">
      No members recorded.
    </p>
  </div>
</template>
