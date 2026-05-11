<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import {
  Bot,
  Slash,
  Sparkles,
  Map,
  Server,
  Palette,
  Package,
  History,
  Settings,
} from 'lucide-vue-next'
import logoUrl from '@/assets/logo.png'
import { useAgentsList } from '@/composables/useAgents'
import { useCommandsList } from '@/composables/useCommands'
import { useSkillsList } from '@/composables/useSkills'
import { usePlansList } from '@/composables/usePlans'
import { useMcpList } from '@/composables/useMcp'
import { useOutputStylesList } from '@/composables/useOutputStyles'
import { usePluginsList } from '@/composables/usePlugins'
import { useProjectsList } from '@/composables/useProjects'

const agents = useAgentsList()
const commands = useCommandsList()
const skills = useSkillsList()
const plans = usePlansList()
const mcp = useMcpList('global')
const outputStyles = useOutputStylesList()
const plugins = usePluginsList()
const projects = useProjectsList()

interface NavItem {
  to: string
  label: string
  icon: typeof Bot
  count: () => number | undefined
}

const items = computed<NavItem[]>(() => [
  { to: '/agents',        label: 'Agents',        icon: Bot,      count: () => agents.data.value?.length },
  { to: '/commands',      label: 'Commands',      icon: Slash,    count: () => commands.data.value?.length },
  { to: '/skills',        label: 'Skills',        icon: Sparkles, count: () => skills.data.value?.length },
  { to: '/plans',         label: 'Plans',         icon: Map,      count: () => plans.data.value?.length },
  { to: '/mcp',           label: 'MCP',           icon: Server,   count: () => mcp.data.value?.length },
  { to: '/output-styles', label: 'Output styles', icon: Palette,  count: () => outputStyles.data.value?.length },
  { to: '/plugins',       label: 'Plugins',       icon: Package,  count: () => plugins.data.value?.length },
  { to: '/sessions',      label: 'Sessions',      icon: History,  count: () => projects.data.value?.length },
  { to: '/settings',      label: 'Settings',      icon: Settings, count: () => undefined },
])
</script>

<template>
  <nav class="flex w-56 shrink-0 flex-col gap-1 border-r border-neutral-200 bg-white px-2 py-4 dark:border-neutral-800 dark:bg-neutral-900">
    <div class="mb-3 flex items-center gap-2 px-3">
      <img
        :src="logoUrl"
        alt=""
        aria-hidden="true"
        class="h-8 w-8 shrink-0 rounded-md"
        style="image-rendering: pixelated;"
      />
      <h1 class="min-w-0 truncate text-sm font-semibold tracking-tight text-neutral-900 dark:text-neutral-100">
        Claude Code GUI
      </h1>
    </div>
    <RouterLink
      v-for="item in items"
      :key="item.to"
      :to="item.to"
      class="flex items-center gap-2 rounded-md px-3 py-1.5 text-sm text-neutral-700 transition-colors hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
      active-class="bg-neutral-100 font-medium text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
    >
      <component :is="item.icon" class="h-4 w-4" />
      <span class="flex-1">{{ item.label }}</span>
      <span
        v-if="item.count() !== undefined"
        class="rounded bg-neutral-200 px-1.5 py-0.5 text-[10px] tabular-nums text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100"
        :aria-label="`${item.count()} ${item.label.toLowerCase()}`"
      >
        {{ item.count() }}
      </span>
    </RouterLink>
  </nav>
</template>
