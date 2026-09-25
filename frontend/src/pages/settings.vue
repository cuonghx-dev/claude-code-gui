<script setup lang="ts">
import { computed, provide } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import ScopePicker from '@/components/settings/ScopePicker.vue'
import { useSettingsScopes } from '@/composables/useSettings'
import { useProjectsList } from '@/composables/useProjects'
import { useKeybindings } from '@/composables/useKeybindings'
import type { SettingsScope } from '@/types/ipc'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'

const route = useRoute()
const router = useRouter()

const projects = useProjectsList()

// Scope and project live in the route query (`?scope=project&project=<name>`)
// so every tab link, reload and deep link keeps which file you were editing.
const SCOPES: SettingsScope[] = ['user', 'project', 'local', 'managed']
const projectName = computed(() =>
  typeof route.query.project === 'string' ? route.query.project : '',
)
const setQuery = (patch: Record<string, string | undefined>) => {
  const next = { ...route.query, ...patch }
  for (const k of Object.keys(next)) if (next[k] === undefined) delete next[k]
  void router.replace({ query: next })
}

const workingDir = computed<string | undefined>({
  get: () => projects.data.value?.find((p) => p.name === projectName.value)?.workingDir,
  set: (dir) => {
    const name = projects.data.value?.find((p) => p.workingDir === dir)?.name
    // Project-bound scopes have no file without a project, so fall back to user.
    const dropScope = !name && (scope.value === 'project' || scope.value === 'local')
    setQuery({ project: name, scope: dropScope ? undefined : route.query.scope as string })
  },
})

const scope = computed<SettingsScope>({
  get: () => {
    const q = route.query.scope
    return SCOPES.includes(q as SettingsScope) ? (q as SettingsScope) : 'user'
  },
  set: (s) => setQuery({ scope: s === 'user' ? undefined : s }),
})

const scopes = useSettingsScopes(workingDir)

provide(SETTINGS_CONTEXT, { scope, workingDir })

const TABS = [
  { to: '/settings', label: 'General', exact: true },
  { to: '/settings/permissions', label: 'Permissions' },
  { to: '/settings/statusline', label: 'Status line' },
  { to: '/settings/hooks', label: 'Hooks' },
  { to: '/settings/keybindings', label: 'Keybindings' },
  { to: '/settings/effective', label: 'Effective' },
  { to: '/settings/raw', label: 'Raw JSON' },
]

const isActive = (tab: { to: string; exact?: boolean }) =>
  tab.exact
    ? route.path === tab.to || route.path === `${tab.to}/`
    : route.path.startsWith(tab.to)

const currentScopeInfo = computed(() =>
  scopes.data.value?.find((s) => s.scope === scope.value),
)

// Keybindings are one global file regardless of scope, so the path shown in
// the tab row follows the tab rather than the scope there.
const keybindings = useKeybindings()
const onKeybindings = computed(() => route.path.startsWith('/settings/keybindings'))
const filePath = computed(() =>
  onKeybindings.value ? keybindings.data.value?.path : currentScopeInfo.value?.path,
)
const fileMissing = computed(() =>
  onKeybindings.value
    ? keybindings.data.value && !keybindings.data.value.exists
    : currentScopeInfo.value && !currentScopeInfo.value.exists,
)
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <header class="flex shrink-0 flex-col gap-3.5 border-b border-[#ECE9E2] px-7 pt-5">
      <div class="flex items-center gap-3">
        <h1 class="ccg-page-title flex-1">Settings</h1>
        <ScopePicker
          v-model="scope"
          :scopes="scopes.data.value ?? []"
          :has-working-dir="!!workingDir"
        />
        <select
          v-model="workingDir"
          class="settings-project"
          aria-label="Project"
          :title="workingDir ?? 'No project selected'"
        >
          <option :value="undefined">No project</option>
          <option v-for="p in projects.data.value ?? []" :key="p.name" :value="p.workingDir">
            {{ p.name }}
          </option>
        </select>
      </div>

      <nav class="ccg-tabs overflow-x-auto">
        <RouterLink
          v-for="t in TABS"
          :key="t.to"
          :to="{ path: t.to, query: route.query }"
          class="ccg-tab"
          :class="{ 'ccg-tab-active': isActive(t) }"
        >
          {{ t.label }}
        </RouterLink>
        <span class="flex-1" />
        <span
          v-if="filePath"
          class="ccg-tab min-w-0 truncate font-mono text-[11.5px] text-[#A29E94] hover:text-[#A29E94]"
          :title="filePath"
        >
          {{ filePath }}<template v-if="fileMissing"> · not created yet</template>
        </span>
      </nav>
    </header>

    <div class="flex min-h-0 flex-1 flex-col overflow-auto">
      <p
        v-if="currentScopeInfo && !currentScopeInfo.writable && !onKeybindings"
        class="ccg-alert-warn mx-7 mt-5 px-3 py-2 text-[12.5px]"
      >
        Managed settings are deployed by your organization and override everything else. This app
        reads them but never writes them.
      </p>
      <RouterView />
    </div>
  </div>
</template>

<style scoped>
.settings-project {
  height: 30px;
  max-width: 220px;
  padding: 0 26px 0 10px;
  border: 1px solid var(--ccg-hairline-strong);
  border-radius: 7px;
  background: #fff
    url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 10 10'%3E%3Cpath d='M2 3.5 5 6.5 8 3.5' fill='none' stroke='%238A867C' stroke-width='1.3'/%3E%3C/svg%3E")
    no-repeat right 9px center;
  appearance: none;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--ccg-ink);
  text-overflow: ellipsis;
  transition: border-color 120ms ease-out;
}
.settings-project:hover {
  border-color: #c9c3b7;
}
.settings-project:focus-visible {
  outline: none;
  border-color: #c9c3b7;
  box-shadow: 0 0 0 3px rgba(31, 30, 27, 0.08);
}
</style>
