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
import { useClaudeCliInfo } from '@/composables/useSettings'

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
const cli = useClaudeCliInfo()

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
  <nav
    class="flex w-[220px] flex-none flex-col gap-[14px] overflow-y-auto border-r px-2.5 py-3"
    style="background: var(--ccg-sidebar); border-color: var(--ccg-chrome-border);"
  >
    <GlobalSearch />
    <div
      v-for="section in sections"
      :key="section.label"
      role="group"
      :aria-label="section.label"
      class="flex flex-col gap-px"
    >
      <h2
        class="px-2.5 pb-1 text-[10.5px] font-semibold uppercase tracking-[.08em]"
        style="color: var(--ccg-muted-soft);"
      >
        {{ section.label }}
      </h2>
      <SidebarLink v-for="item in section.items" :key="item.to" :item="item" />
    </div>
    <div class="flex-1" />
    <div class="flex flex-col gap-px">
      <SidebarLink v-for="item in bottomItems" :key="item.to" :item="item" />
    </div>
    <div
      class="-mt-2 flex items-center gap-2 border-t px-2.5 pt-2 font-mono text-[11px]"
      style="border-color: var(--ccg-chrome-border); color: var(--ccg-subtle);"
      :title="cli.data.value?.path ?? 'claude CLI not found on PATH'"
    >
      <span
        class="h-[7px] w-[7px] flex-none rounded-full"
        :style="{ background: cli.isPending.value ? 'var(--ccg-muted-soft)' : cli.data.value ? '#4E9A5B' : 'var(--ccg-error)' }"
      />
      <span class="truncate">{{
        cli.isPending.value ? 'claude …' : cli.data.value ? `claude ${cli.data.value.version.split(/\s/)[0]}` : 'claude not found'
      }}</span>
    </div>
  </nav>
</template>
