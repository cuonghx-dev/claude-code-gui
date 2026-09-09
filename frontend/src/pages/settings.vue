<script setup lang="ts">
import { computed, provide, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import ScopePicker from '@/components/settings/ScopePicker.vue'
import { useSettingsScopes } from '@/composables/useSettings'
import { useProjectsList } from '@/composables/useProjects'
import type { SettingsScope } from '@/types/ipc'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'

const route = useRoute()

// Scope and project are shared by every child tab: switching tabs should not
// lose which file you were editing.
const scope = ref<SettingsScope>('user')
const workingDir = ref<string | undefined>(undefined)

const projects = useProjectsList()
const scopes = useSettingsScopes(workingDir)

provide(SETTINGS_CONTEXT, { scope, workingDir })

const TABS = [
  { to: '/settings', label: 'General', exact: true },
  { to: '/settings/permissions', label: 'Permissions' },
  { to: '/settings/hooks', label: 'Hooks' },
  { to: '/settings/statusline', label: 'Status line' },
  { to: '/settings/keybindings', label: 'Keybindings' },
  { to: '/settings/effective', label: 'Effective' },
  { to: '/settings/raw', label: 'Raw JSON' },
]

const isActive = (tab: { to: string; exact?: boolean }) =>
  tab.exact ? route.path === tab.to : route.path.startsWith(tab.to)

const currentScopeInfo = computed(() =>
  scopes.data.value?.find((s) => s.scope === scope.value),
)
</script>

<template>
  <PageHeader title="Settings" subtitle="Claude Code settings files and app preferences" />

  <div class="flex flex-wrap items-end gap-4 border-b border-neutral-200 px-6 pb-3 dark:border-neutral-800">
    <ScopePicker
      v-model="scope"
      :scopes="scopes.data.value ?? []"
      :has-working-dir="!!workingDir"
    />
    <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
      Project
      <select v-model="workingDir" class="ccg-input w-64">
        <option :value="undefined">— none —</option>
        <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
          {{ p.workingDir }}
        </option>
      </select>
    </label>
  </div>

  <nav class="flex gap-1 overflow-x-auto border-b border-neutral-200 px-6 dark:border-neutral-800">
    <RouterLink
      v-for="t in TABS"
      :key="t.to"
      :to="t.to"
      class="shrink-0 border-b-2 px-3 py-2 text-sm"
      :class="
        isActive(t)
          ? 'border-neutral-900 font-medium dark:border-neutral-100'
          : 'border-transparent text-neutral-500 dark:text-neutral-400'
      "
    >
      {{ t.label }}
    </RouterLink>
  </nav>

  <p
    v-if="currentScopeInfo && !currentScopeInfo.writable"
    class="mx-6 mt-4 rounded-md bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300"
  >
    Managed settings are deployed by your organization and override everything else. This app reads
    them but never writes them.
  </p>

  <RouterView />
</template>
