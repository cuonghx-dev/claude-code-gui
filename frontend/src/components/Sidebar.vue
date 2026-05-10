<script setup lang="ts">
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

interface NavItem {
  to: string
  label: string
  icon: typeof Bot
}

// Phase 1 wires count badges from `useAgentsList().data.value?.length` etc.
const items: NavItem[] = [
  { to: '/agents',        label: 'Agents',        icon: Bot },
  { to: '/commands',      label: 'Commands',      icon: Slash },
  { to: '/skills',        label: 'Skills',        icon: Sparkles },
  { to: '/plans',         label: 'Plans',         icon: Map },
  { to: '/mcp',           label: 'MCP',           icon: Server },
  { to: '/output-styles', label: 'Output styles', icon: Palette },
  { to: '/plugins',       label: 'Plugins',       icon: Package },
  { to: '/sessions',      label: 'Sessions',      icon: History },
  { to: '/settings',      label: 'Settings',      icon: Settings },
]
</script>

<template>
  <nav class="flex w-56 shrink-0 flex-col gap-1 border-r border-neutral-200 bg-white px-2 py-4 dark:border-neutral-800 dark:bg-neutral-900">
    <div class="mb-3 px-3">
      <h1 class="text-sm font-semibold tracking-tight text-neutral-900 dark:text-neutral-100">
        Claude Code GUI
      </h1>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">v0.1.0</p>
    </div>
    <RouterLink
      v-for="item in items"
      :key="item.to"
      :to="item.to"
      class="flex items-center gap-2 rounded-md px-3 py-1.5 text-sm text-neutral-700 transition-colors hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
      active-class="bg-neutral-100 font-medium text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
    >
      <component :is="item.icon" class="h-4 w-4" />
      {{ item.label }}
    </RouterLink>
  </nav>
</template>
