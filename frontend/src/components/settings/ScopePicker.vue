<script setup lang="ts">
import type { ScopeInfo, SettingsScope } from '@/types/ipc'

const props = defineProps<{
  modelValue: SettingsScope
  scopes: ScopeInfo[]
  /** Project and local scopes need a project selected. */
  hasWorkingDir: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [SettingsScope] }>()

// Fixed order from the least to the most specific file, with Managed last
// since it is read-only.
const ORDER: SettingsScope[] = ['user', 'project', 'local', 'managed']
const LABEL: Record<SettingsScope, string> = {
  managed: 'Managed',
  local: 'Project local',
  project: 'Project',
  user: 'User',
}

const needsProject = (s: SettingsScope) => (s === 'project' || s === 'local') && !props.hasWorkingDir
const info = (s: SettingsScope) => props.scopes.find((i) => i.scope === s)
const readOnly = (s: SettingsScope) => s === 'managed' || info(s)?.writable === false
</script>

<template>
  <div class="ccg-seg" role="group" aria-label="Settings scope">
    <button
      v-for="s in ORDER"
      :key="s"
      type="button"
      :aria-pressed="s === modelValue"
      :disabled="needsProject(s) && s !== modelValue"
      :class="{ 'scope-readonly': readOnly(s) && s !== modelValue }"
      :title="
        needsProject(s)
          ? 'Select a project first'
          : readOnly(s)
            ? `${info(s)?.path ?? LABEL[s]} (read-only)`
            : info(s)?.path
      "
      @click="emit('update:modelValue', s)"
    >
      {{ LABEL[s] }}
    </button>
  </div>
</template>

<style scoped>
.ccg-seg > button.scope-readonly {
  color: var(--ccg-muted-soft);
}
</style>
