<script setup lang="ts">
import { computed } from 'vue'
import {
  Bot,
  Slash,
  Sparkles,
  Map,
  Workflow,
  Server,
  Palette,
  Package,
  History,
  Activity,
  BarChart3,
  Webhook,
  FolderTree,
  BookText,
  Settings,
} from 'lucide-vue-next'
import logoUrl from '@/assets/logo.png'
import GlobalSearch from '@/components/GlobalSearch.vue'
import SidebarLink from '@/components/SidebarLink.vue'
import type { NavItem } from '@/components/SidebarLink.vue'
import { useAgentsList } from '@/composables/useAgents'
import { useCommandsList } from '@/composables/useCommands'
import { useSkillsList } from '@/composables/useSkills'
import { useJobsList } from '@/composables/useJobs'
import { useMemoryList } from '@/composables/useMemory'
import { usePlansList } from '@/composables/usePlans'
import { useWorkflowsList } from '@/composables/useWorkflows'
import { useMcpList } from '@/composables/useMcp'
import { useOutputStylesList } from '@/composables/useOutputStyles'
import { useHooksList } from '@/composables/useHooks'
import { usePluginsList } from '@/composables/usePlugins'
import { useProjectsList } from '@/composables/useProjects'

const agents = useAgentsList()
const commands = useCommandsList()
const skills = useSkillsList()
const plans = usePlansList()
const jobs = useJobsList()
const memory = useMemoryList()
const workflows = useWorkflowsList()
const mcp = useMcpList('global')
const outputStyles = useOutputStylesList()
const hooks = useHooksList()
const plugins = usePluginsList()
const projects = useProjectsList()

interface NavSection {
  label: string
  items: NavItem[]
}

const sections = computed<NavSection[]>(() => [
  {
    label: 'Authoring',
    items: [
      { to: '/agents',        label: 'Agents',        icon: Bot,      count: () => agents.data.value?.length },
      { to: '/commands',      label: 'Commands',      icon: Slash,    count: () => commands.data.value?.length },
      { to: '/skills',        label: 'Skills',        icon: Sparkles, count: () => skills.data.value?.length },
      { to: '/plans',         label: 'Plans',         icon: Map,      count: () => plans.data.value?.length },
      { to: '/workflows',     label: 'Workflows',     icon: Workflow, count: () => workflows.data.value?.length },
      { to: '/output-styles', label: 'Output styles', icon: Palette,  count: () => outputStyles.data.value?.length },
    ],
  },
  {
    label: 'Extend',
    items: [
      { to: '/mcp',           label: 'MCP',           icon: Server,   count: () => mcp.data.value?.length },
      { to: '/hooks',         label: 'Hooks',         icon: Webhook,  count: () => hooks.data.value?.length },
      { to: '/plugins',       label: 'Plugins',       icon: Package,  count: () => plugins.data.value?.length },
    ],
  },
  {
    label: 'Activity',
    items: [
      { to: '/sessions',      label: 'Sessions',      icon: History,  count: () => projects.data.value?.length },
      { to: '/jobs',          label: 'Jobs',          icon: Activity, count: () => jobs.data.value?.length },
      { to: '/usage',         label: 'Usage',         icon: BarChart3, count: () => undefined },
    ],
  },
  {
    label: 'Config',
    items: [
      { to: '/memory',        label: 'Memory',        icon: BookText, count: () => memory.data.value?.length },
      { to: '/claude-directory', label: '.claude',    icon: FolderTree, count: () => undefined },
    ],
  },
])

const bottomItems = computed<NavItem[]>(() => [
  { to: '/settings',      label: 'Settings',      icon: Settings, count: () => undefined },
])
</script>

<template>
  <nav class="flex w-56 shrink-0 flex-col gap-1 border-r bg-canvas px-2 py-4" style="border-color: var(--ccg-hairline);">
    <div class="mb-6 mt-4 flex items-center gap-2 px-3">
      <img
        :src="logoUrl"
        alt=""
        aria-hidden="true"
        class="h-7 w-7 shrink-0 rounded-md"
        style="image-rendering: pixelated;"
      />
      <h1
        class="flex items-baseline gap-1.5 text-sm font-medium leading-none text-ink"
        style="letter-spacing: -0.015em;"
      >
        <span>Claude Code</span>
        <span
          class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider"
          style="background-color: var(--ccg-primary); color: var(--ccg-on-primary);"
        >GUI</span>
      </h1>
    </div>
    <GlobalSearch />
    <div
      v-for="(section, i) in sections"
      :key="section.label"
      role="group"
      :aria-label="section.label"
      class="flex flex-col gap-1"
      :class="i === 0 ? '' : 'mt-3'"
    >
      <h2 class="mb-0.5 px-3 text-[10px] font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
        {{ section.label }}
      </h2>
      <SidebarLink v-for="item in section.items" :key="item.to" :item="item" />
    </div>
    <div class="mt-auto flex flex-col gap-1 border-t border-neutral-200 pt-2 dark:border-neutral-800">
      <SidebarLink v-for="item in bottomItems" :key="item.to" :item="item" />
    </div>
  </nav>
</template>
